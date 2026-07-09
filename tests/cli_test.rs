use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_herdr-focus-notify"))
}

#[test]
fn help_and_version_print_to_stdout() {
    let help = binary().arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--focus-latest"));
    assert!(help.stderr.is_empty());

    let version = binary().arg("--version").output().unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert!(version.stderr.is_empty());
}

#[test]
fn no_event_is_quiet_even_when_notifier_config_is_bad() {
    let output = binary()
        .env("HERDR_FOCUS_NOTIFY_NOTIFIER", "/definitely/missing")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn test_mode_reports_bad_notifier() {
    let output = binary()
        .arg("--test")
        .env("HERDR_FOCUS_NOTIFY_NOTIFIER", "/definitely/missing")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configured notifier"));
}

#[test]
fn pane_focused_event_removes_matching_notification_group() {
    let dir = temp_test_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let fake_notifier = dir.join("alerter");
    let log = dir.join("args.log");
    std::fs::write(
        &fake_notifier,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\n",
            shell_quote(log.to_string_lossy().as_ref())
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&fake_notifier).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&fake_notifier, permissions).unwrap();
    }

    let output = binary()
        .env("HERDR_FOCUS_NOTIFY_NOTIFIER", &fake_notifier)
        .env("HERDR_PLUGIN_STATE_DIR", &dir)
        .env(
            "HERDR_PLUGIN_EVENT_JSON",
            r#"{"event":"pane_focused","data":{"type":"pane_focused","pane_id":"w1:p3"}}"#,
        )
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        "--remove\nherdr-w1-p3\n"
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pane_focused_event_replays_other_active_notifications() {
    let dir = temp_test_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let fake_notifier = dir.join("alerter");
    let log = dir.join("args.log");
    std::fs::write(
        &fake_notifier,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> {}\n",
            shell_quote(log.to_string_lossy().as_ref())
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&fake_notifier).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&fake_notifier, permissions).unwrap();
    }

    for pane_id in ["p1", "p2"] {
        let event_json = format!(
            r#"{{"event":"pane_agent_status_changed","data":{{"type":"pane_agent_status_changed","pane_id":"{pane_id}","workspace_id":"work","agent_status":"blocked","agent":"Codex","title":"Task {pane_id}"}}}}"#
        );
        let output = binary()
            .env("HERDR_FOCUS_NOTIFY_NOTIFIER", &fake_notifier)
            .env("HERDR_PLUGIN_STATE_DIR", &dir)
            .env("HERDR_PLUGIN_EVENT_JSON", event_json)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    wait_for_log_contains(&log, "herdr-p2");
    std::fs::write(&log, "").unwrap();

    let output = binary()
        .env("HERDR_FOCUS_NOTIFY_NOTIFIER", &fake_notifier)
        .env("HERDR_PLUGIN_STATE_DIR", &dir)
        .env(
            "HERDR_PLUGIN_EVENT_JSON",
            r#"{"event":"pane_focused","data":{"type":"pane_focused","pane_id":"p1"}}"#,
        )
        .output()
        .unwrap();

    assert!(output.status.success());
    let content = wait_for_log_contains(&log, "herdr-p2");
    assert!(content.contains("--remove\nherdr-p1"));
    assert!(content.contains("--group\nherdr-p2"));
    let active = std::fs::read_to_string(dir.join("active-notifications.json")).unwrap();
    assert!(!active.contains("\"pane_id\": \"p1\""));
    assert!(active.contains("\"pane_id\": \"p2\""));

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn focus_latest_focuses_newest_active_notification_and_removes_it() {
    let dir = temp_test_dir();
    std::fs::create_dir_all(&dir).unwrap();

    let fake_herdr = dir.join("herdr");
    let focus_log = dir.join("focus.log");
    std::fs::write(
        &fake_herdr,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"agent\" ] && [ \"$2\" = \"focus\" ]; then\n  printf '%s\\n' \"$3\" >> {}\n  exit 0\nfi\nif [ \"$1\" = \"agent\" ] && [ \"$2\" = \"list\" ]; then\n  printf '%s' '{{\"result\":{{\"agents\":[]}}}}'\n  exit 0\nfi\nexit 1\n",
            shell_quote(focus_log.to_string_lossy().as_ref())
        ),
    )
    .unwrap();

    let fake_notifier = dir.join("alerter");
    let notifier_log = dir.join("notifier.log");
    std::fs::write(
        &fake_notifier,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> {}\nexit 0\n",
            shell_quote(notifier_log.to_string_lossy().as_ref())
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for path in [&fake_herdr, &fake_notifier] {
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o700);
            std::fs::set_permissions(path, permissions).unwrap();
        }
    }

    let active = r#"[
  {
    "pane_id": "p1",
    "status": "blocked",
    "title": "First",
    "body": "Body 1",
    "group": "herdr-p1",
    "app_icon": null
  },
  {
    "pane_id": "p2",
    "status": "done",
    "title": "Second",
    "body": "Body 2",
    "group": "herdr-p2",
    "app_icon": null
  }
]"#;
    std::fs::write(dir.join("active-notifications.json"), active).unwrap();

    let output = binary()
        .arg("--focus-latest")
        .env("HERDR_BIN_PATH", &fake_herdr)
        .env("HERDR_FOCUS_NOTIFY_NOTIFIER", &fake_notifier)
        .env("HERDR_PLUGIN_STATE_DIR", &dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(&focus_log).unwrap(), "p2\n");

    let notifier_content = wait_for_log_contains(&notifier_log, "herdr-p1");
    assert!(notifier_content.contains("--remove\nherdr-p2\n"));
    assert!(notifier_content.contains("--group\nherdr-p1\n"));

    let active_after = std::fs::read_to_string(dir.join("active-notifications.json")).unwrap();
    assert!(active_after.contains("\"pane_id\": \"p1\""));
    assert!(!active_after.contains("\"pane_id\": \"p2\""));

    std::fs::remove_dir_all(dir).unwrap();
}

fn temp_test_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "herdr-focus-notify-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn wait_for_log_contains(path: &std::path::Path, needle: &str) -> String {
    for _ in 0..100 {
        if let Ok(content) = std::fs::read_to_string(path) {
            if content.contains(needle) {
                return content;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {needle} in {}", path.display());
}

fn shell_quote(value: &str) -> String {
    let mut quoted = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}
