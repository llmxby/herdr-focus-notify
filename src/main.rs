mod cli;
mod config;
mod event;
mod executable;
mod focus;
mod icons;
mod notification;
mod notifier;
mod script;
mod state;
mod util;

use std::env;
use std::process::ExitCode;

use cli::{parse_cli_args, print_usage, CliAction};
use config::{is_enabled, status_is_enabled};
use event::{event_action_from_event_json, PluginEventAction};
use executable::resolve_herdr_bin;
use focus::{should_skip_notification, test_notification};
use notification::notification_group_for_pane;
use notifier::{resolve_notifier_bin, send_notification};
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
        CliAction::Event | CliAction::Test => {}
    }

    if !is_enabled() {
        return Ok(());
    }

    let herdr_bin = resolve_herdr_bin();

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
                    dismiss_notification(&pane_id, replay_remaining, &herdr_bin, &notifier_bin)?;
                    return Ok(());
                }
                None => return Ok(()),
            }
        }
        CliAction::Help | CliAction::Version => unreachable!("handled before notification setup"),
    };

    if !status_is_enabled(&notification.status) {
        return Ok(());
    }

    if should_skip_notification(&notification.pane_id, &herdr_bin) {
        return Ok(());
    }

    let notifier_bin = resolve_notifier_bin()?;
    deliver_notification(
        &notification,
        &herdr_bin,
        &notifier_bin,
        action == CliAction::Test,
    )?;
    if action == CliAction::Event {
        state::save_active_notification(&notification)
            .map_err(|err| format!("failed to save active notification: {err}"))?;
    }

    Ok(())
}

fn dismiss_notification(
    pane_id: &str,
    replay_remaining: bool,
    herdr_bin: &str,
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
            deliver_notification(&notification, herdr_bin, notifier_bin, false)?;
        }
    }

    Ok(())
}

fn deliver_notification(
    notification: &notification::FocusNotification,
    herdr_bin: &str,
    notifier_bin: &str,
    foreground: bool,
) -> Result<(), String> {
    let script_path = write_focus_script(notification, herdr_bin, notifier_bin)
        .map_err(|err| format!("failed to write focus script: {err}"))?;

    send_notification(&script_path, foreground)
        .map_err(|err| format!("failed to send notification: {err}"))
}
