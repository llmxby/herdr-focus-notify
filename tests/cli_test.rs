use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_herdr-focus-notify"))
}

#[test]
fn help_and_version_print_to_stdout() {
    let help = binary().arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Usage:"));
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
    let dir = std::env::temp_dir().join(format!(
        "herdr-focus-notify-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
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
