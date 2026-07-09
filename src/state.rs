use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::notification::FocusNotification;

const ACTIVE_NOTIFICATIONS_FILE: &str = "active-notifications.json";

pub(crate) fn save_active_notification(notification: &FocusNotification) -> io::Result<()> {
    let mut notifications = load_active_notifications()?;
    upsert_active_notification(&mut notifications, notification.clone());
    write_active_notifications(&notifications)
}

pub(crate) fn dismiss_active_notification(pane_id: &str) -> io::Result<Vec<FocusNotification>> {
    let mut notifications = load_active_notifications()?;
    remove_active_notification(&mut notifications, pane_id);
    write_active_notifications(&notifications)?;
    Ok(notifications)
}

pub(crate) fn latest_active_notification() -> io::Result<Option<FocusNotification>> {
    Ok(load_active_notifications()?.into_iter().last())
}

fn upsert_active_notification(
    notifications: &mut Vec<FocusNotification>,
    notification: FocusNotification,
) {
    notifications.retain(|active| active.pane_id != notification.pane_id);
    notifications.push(notification);
}

fn remove_active_notification(notifications: &mut Vec<FocusNotification>, pane_id: &str) {
    notifications.retain(|active| active.pane_id != pane_id);
}

fn load_active_notifications() -> io::Result<Vec<FocusNotification>> {
    let path = active_notifications_path();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };

    serde_json::from_str(&content).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn write_active_notifications(notifications: &[FocusNotification]) -> io::Result<()> {
    let state_dir = state_dir();
    fs::create_dir_all(&state_dir)?;
    let path = state_dir.join(ACTIVE_NOTIFICATIONS_FILE);
    let content = serde_json::to_string_pretty(notifications).map_err(io::Error::other)?;
    fs::write(path, content)
}

fn active_notifications_path() -> PathBuf {
    state_dir().join(ACTIVE_NOTIFICATIONS_FILE)
}

fn state_dir() -> PathBuf {
    env::var_os("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("herdr-focus-notify"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(pane_id: &str, body: &str) -> FocusNotification {
        FocusNotification {
            pane_id: pane_id.to_string(),
            status: "blocked".to_string(),
            title: format!("{pane_id} title"),
            body: body.to_string(),
            group: crate::notification::notification_group_for_pane(pane_id),
            app_icon: None,
        }
    }

    #[test]
    fn active_notifications_are_upserted_and_removed_by_pane() {
        let mut notifications = Vec::new();

        upsert_active_notification(&mut notifications, notification("p1", "first"));
        upsert_active_notification(&mut notifications, notification("p2", "second"));
        upsert_active_notification(&mut notifications, notification("p1", "updated"));
        remove_active_notification(&mut notifications, "p1");

        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].pane_id, "p2");
        assert_eq!(notifications[0].body, "second");
    }
}
