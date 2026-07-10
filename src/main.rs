mod activate;
mod away;
mod cli;
mod config;
mod event;
mod executable;
mod focus;
mod icons;
mod notification;
mod notifier;
mod presence;
mod script;
mod state;
mod util;

use std::env;
use std::process::Command;
use std::process::ExitCode;

use activate::activate_configured_app;
use cli::{parse_cli_args, print_usage, CliAction};
use config::{away_command, away_mode, away_recipients, is_enabled, status_is_enabled};
use event::{event_action_from_event_json, PluginEventAction};
use executable::{resolve_herdr_bin, resolve_plugin_bin};
use focus::{should_skip_notification, test_notification};
use notification::notification_group_for_pane;
use notifier::{resolve_notifier_bin, send_notification};
use presence::should_use_away_delivery;
use script::write_focus_script;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("herdr-focus-notify: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let action = parse_cli_args(env::args().skip(1))?;

    match action {
        CliAction::Help => {
            print_usage();
            return Ok(());
        }
        CliAction::Version => {
            println!("herdr-focus-notify {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        CliAction::Event | CliAction::Test | CliAction::FocusLatest | CliAction::OnClick(_) => {}
    }

    if !is_enabled() {
        return Ok(());
    }

    let herdr_bin = resolve_herdr_bin();
    let plugin_bin = resolve_plugin_bin();

    if let CliAction::OnClick(pane_id) = action {
        let notifier_bin = resolve_notifier_bin()?;
        dismiss_notification(&pane_id, true, &herdr_bin, &plugin_bin, &notifier_bin)?;
        return Ok(());
    }

    if action == CliAction::FocusLatest {
        let notifier_bin = resolve_notifier_bin()?;
        focus_latest_active_notification(&herdr_bin, &plugin_bin, &notifier_bin)?;
        return Ok(());
    }

    let notification = match action {
        CliAction::Test => test_notification(&herdr_bin),
        CliAction::Event => {
            let event_json = match env::var("HERDR_PLUGIN_EVENT_JSON") {
                Ok(value) => value,
                Err(_) => return Ok(()),
            };

            match event_action_from_event_json(&event_json)? {
                Some(PluginEventAction::Notify(notification)) => notification,
                Some(PluginEventAction::DismissPane {
                    pane_id,
                    replay_remaining,
                }) => {
                    let notifier_bin = resolve_notifier_bin()?;
                    dismiss_notification(
                        &pane_id,
                        replay_remaining,
                        &herdr_bin,
                        &plugin_bin,
                        &notifier_bin,
                    )?;
                    return Ok(());
                }
                None => return Ok(()),
            }
        }
        CliAction::Help | CliAction::Version | CliAction::FocusLatest | CliAction::OnClick(_) => {
            unreachable!("handled before notification setup")
        }
    };

    if !status_is_enabled(&notification.status) {
        return Ok(());
    }

    if should_skip_notification(&notification.pane_id, &herdr_bin) {
        return Ok(());
    }

    if try_deliver_away_notification(&notification)? {
        if action == CliAction::Event {
            state::save_active_notification(&notification)
                .map_err(|err| format!("failed to save active notification: {err}"))?;
        }
        if away_mode() == "replace" {
            return Ok(());
        }
    }

    let notifier_bin = resolve_notifier_bin()?;
    deliver_notification(
        &notification,
        &herdr_bin,
        &plugin_bin,
        &notifier_bin,
        action == CliAction::Test,
    )?;
    if action == CliAction::Event {
        state::save_active_notification(&notification)
            .map_err(|err| format!("failed to save active notification: {err}"))?;
    }

    Ok(())
}

fn try_deliver_away_notification(
    notification: &notification::FocusNotification,
) -> Result<bool, String> {
    let mode = away_mode();
    if mode == "off" {
        return Ok(false);
    }

    if !should_use_away_delivery() {
        return Ok(false);
    }

    let command = away_command().ok_or_else(|| {
        "HERDR_FOCUS_NOTIFY_AWAY_COMMAND is required when away mode is enabled".to_string()
    })?;
    let recipients = away_recipients();
    if recipients.is_empty() {
        return Err(
            "HERDR_FOCUS_NOTIFY_AWAY_RECIPIENTS is required when away mode is enabled".to_string(),
        );
    }

    away::run_away_command(&command, &recipients, notification)?;
    Ok(true)
}

fn focus_latest_active_notification(
    herdr_bin: &str,
    plugin_bin: &str,
    notifier_bin: &str,
) -> Result<(), String> {
    let Some(notification) = state::latest_active_notification()
        .map_err(|err| format!("failed to load active notifications: {err}"))?
    else {
        return Ok(());
    };

    activate_configured_app()?;
    focus_pane(herdr_bin, &notification.pane_id)?;

    let remaining = state::dismiss_active_notification(&notification.pane_id)
        .map_err(|err| format!("failed to update active notifications: {err}"))?;
    notifier::remove_notification(notifier_bin, &notification.group)
        .map_err(|err| format!("failed to remove notification: {err}"))?;

    for notification in remaining {
        if !status_is_enabled(&notification.status) {
            continue;
        }
        deliver_notification(&notification, herdr_bin, plugin_bin, notifier_bin, false)?;
    }

    Ok(())
}

fn focus_pane(herdr_bin: &str, pane_id: &str) -> Result<(), String> {
    let status = Command::new(herdr_bin)
        .arg("agent")
        .arg("focus")
        .arg(pane_id)
        .status()
        .map_err(|err| format!("failed to run herdr agent focus: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("herdr agent focus exited with {status}"))
    }
}

fn dismiss_notification(
    pane_id: &str,
    replay_remaining: bool,
    herdr_bin: &str,
    plugin_bin: &str,
    notifier_bin: &str,
) -> Result<(), String> {
    let remaining = state::dismiss_active_notification(pane_id)
        .map_err(|err| format!("failed to update active notifications: {err}"))?;
    let group = notification_group_for_pane(pane_id);
    notifier::remove_notification(notifier_bin, &group)
        .map_err(|err| format!("failed to remove notification: {err}"))?;

    if replay_remaining {
        for notification in remaining {
            if !status_is_enabled(&notification.status) {
                continue;
            }
            deliver_notification(&notification, herdr_bin, plugin_bin, notifier_bin, false)?;
        }
    }

    Ok(())
}

fn deliver_notification(
    notification: &notification::FocusNotification,
    herdr_bin: &str,
    plugin_bin: &str,
    notifier_bin: &str,
    foreground: bool,
) -> Result<(), String> {
    let script_path = write_focus_script(notification, herdr_bin, plugin_bin, notifier_bin)
        .map_err(|err| format!("failed to write focus script: {err}"))?;

    send_notification(&script_path, foreground)
        .map_err(|err| format!("failed to send notification: {err}"))
}
