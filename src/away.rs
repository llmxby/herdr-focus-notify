use std::process::Command;

use crate::notification::FocusNotification;

pub(crate) fn run_away_command(
    command: &str,
    recipients: &[String],
    notification: &FocusNotification,
) -> Result<(), String> {
    let status = Command::new(command)
        .env("HERDR_FOCUS_NOTIFY_TITLE", &notification.title)
        .env("HERDR_FOCUS_NOTIFY_BODY", &notification.body)
        .env("HERDR_FOCUS_NOTIFY_STATUS", &notification.status)
        .env("HERDR_FOCUS_NOTIFY_PANE_ID", &notification.pane_id)
        .env("HERDR_FOCUS_NOTIFY_GROUP", &notification.group)
        .env("HERDR_FOCUS_NOTIFY_RECIPIENTS", recipients.join(","))
        .status()
        .map_err(|err| format!("failed to run away command: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("away command exited with {status}"))
    }
}
