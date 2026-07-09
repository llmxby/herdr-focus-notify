use serde::Deserialize;

use crate::icons::agent_icon_path;
use crate::notification::{notification_group_for_pane, FocusNotification};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PluginEventAction {
    Notify(FocusNotification),
    DismissPane {
        pane_id: String,
        replay_remaining: bool,
    },
}

#[derive(Debug, Deserialize)]
struct PluginEvent {
    event: Option<String>,
    data: Option<EventData>,
}

#[derive(Debug, Deserialize)]
struct EventData {
    #[serde(rename = "type")]
    event_type: Option<String>,
    pane_id: Option<String>,
    agent_status: Option<String>,
    agent: Option<String>,
    display_agent: Option<String>,
    title: Option<String>,
    custom_status: Option<String>,
    workspace_id: Option<String>,
}

pub(crate) fn event_action_from_event_json(
    json: &str,
) -> Result<Option<PluginEventAction>, String> {
    let event: PluginEvent =
        serde_json::from_str(json).map_err(|err| format!("invalid event json: {err}"))?;
    let Some(data) = event.data else {
        return Ok(None);
    };

    let event_name = event_name(event.event.as_deref(), data.event_type.as_deref());
    if event_matches(event_name, "pane.focused", "pane_focused") {
        return Ok(
            pane_id_from_data(&data).map(|pane_id| PluginEventAction::DismissPane {
                pane_id,
                replay_remaining: true,
            }),
        );
    }

    if !event_name
        .map(|name| {
            event_matches(
                Some(name),
                "pane.agent_status_changed",
                "pane_agent_status_changed",
            )
        })
        .unwrap_or(true)
    {
        return Ok(None);
    }

    event_action_from_agent_status_changed(data)
}

#[cfg(test)]
fn notification_from_event_json(json: &str) -> Result<Option<FocusNotification>, String> {
    match event_action_from_event_json(json)? {
        Some(PluginEventAction::Notify(notification)) => Ok(Some(notification)),
        Some(PluginEventAction::DismissPane { .. }) | None => Ok(None),
    }
}

fn event_action_from_agent_status_changed(
    data: EventData,
) -> Result<Option<PluginEventAction>, String> {
    let status = data
        .agent_status
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    let Some(pane_id) = pane_id_from_data(&data) else {
        return Ok(None);
    };

    if status != "blocked" && status != "done" {
        return Ok(Some(PluginEventAction::DismissPane {
            pane_id,
            replay_remaining: false,
        }));
    }

    let agent = first_non_empty([data.display_agent.as_deref(), data.agent.as_deref()])
        .unwrap_or("Agent")
        .to_string();
    let app_icon = agent_icon_path(&[data.display_agent.as_deref(), data.agent.as_deref()]);

    let base_title = match status.as_str() {
        "blocked" => format!("{agent} needs attention"),
        "done" => format!("{agent} finished"),
        _ => unreachable!("status already filtered"),
    };
    let title = if let Some(custom_status) = first_non_empty([data.custom_status.as_deref()]) {
        format!("{base_title}: {custom_status}")
    } else {
        base_title
    };

    let body = notification_body(&data, &pane_id, &status);
    let group = notification_group_for_pane(&pane_id);

    Ok(Some(PluginEventAction::Notify(FocusNotification {
        pane_id,
        status,
        title,
        body,
        group,
        app_icon,
    })))
}

fn event_name<'a>(event: Option<&'a str>, data_type: Option<&'a str>) -> Option<&'a str> {
    first_non_empty([event, data_type])
}

fn event_matches(event_name: Option<&str>, dotted: &str, snake: &str) -> bool {
    event_name
        .map(str::trim)
        .is_some_and(|name| name == dotted || name == snake)
}

fn pane_id_from_data(data: &EventData) -> Option<String> {
    data.pane_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn notification_body(data: &EventData, pane_id: &str, status: &str) -> String {
    let summary = first_non_empty([data.title.as_deref()])
        .map(|text| truncate(text, 180))
        .unwrap_or_else(|| "Click to focus this Herdr agent pane.".to_string());
    let mut details = vec![format!("pane {pane_id}")];

    if let Some(workspace_id) = first_non_empty([data.workspace_id.as_deref()]) {
        details.push(format!("workspace {workspace_id}"));
    }
    if let Some(custom_status) = first_non_empty([data.custom_status.as_deref()]) {
        details.push(format!("status {custom_status}"));
    } else {
        details.push(format!("status {status}"));
    }

    format!("{summary}\n{}", details.join(" | "))
}

fn first_non_empty<const N: usize>(values: [Option<&str>; N]) -> Option<&str> {
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn truncate(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let mut output: String = trimmed.chars().take(max_chars.saturating_sub(3)).collect();
    output.push_str("...");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_blocked_notification_from_event() {
        let json = r#"{
            "event": "pane.agent_status_changed",
            "data": {
                "pane_id": "w1:p3",
                "workspace_id": "herdr",
                "agent_status": "blocked",
                "agent": "codex",
                "display_agent": "Codex",
                "title": "Implement plugin",
                "custom_status": "Needs an answer"
            }
        }"#;

        let notification = notification_from_event_json(json).unwrap().unwrap();

        assert_eq!(notification.pane_id, "w1:p3");
        assert_eq!(notification.status, "blocked");
        assert_eq!(notification.title, "Codex needs attention: Needs an answer");
        assert_eq!(
            notification.body,
            "Implement plugin\npane w1:p3 | workspace herdr | status Needs an answer"
        );
        assert_eq!(notification.group, "herdr-w1-p3");
        assert!(notification
            .app_icon
            .as_deref()
            .unwrap()
            .ends_with("/icons/codex-color.png"));
    }

    #[test]
    fn builds_done_notification_from_herdr_hook_event_json() {
        let json = r#"{
            "event": "pane_agent_status_changed",
            "data": {
                "type": "pane_agent_status_changed",
                "pane_id": "p1",
                "agent_status": "done",
                "agent": "Codex",
                "title": "Implement plugin"
            }
        }"#;

        let notification = notification_from_event_json(json).unwrap().unwrap();

        assert_eq!(notification.status, "done");
        assert_eq!(notification.title, "Codex finished");
        assert_eq!(notification.body, "Implement plugin\npane p1 | status done");
        assert!(notification.app_icon.is_some());
    }

    #[test]
    fn builds_dismiss_action_from_pane_focused_event() {
        let json = r#"{
            "event": "pane_focused",
            "data": {
                "type": "pane_focused",
                "pane_id": "w1:p3",
                "workspace_id": "herdr"
            }
        }"#;

        let action = event_action_from_event_json(json).unwrap().unwrap();

        assert_eq!(
            action,
            PluginEventAction::DismissPane {
                pane_id: "w1:p3".to_string(),
                replay_remaining: true,
            }
        );
        assert!(notification_from_event_json(json).unwrap().is_none());
    }

    #[test]
    fn dismisses_other_statuses_to_clear_stale_notifications() {
        let json = r#"{
            "data": {
                "pane_id": "p1",
                "agent_status": "running",
                "agent": "Codex"
            }
        }"#;

        assert_eq!(
            event_action_from_event_json(json).unwrap(),
            Some(PluginEventAction::DismissPane {
                pane_id: "p1".to_string(),
                replay_remaining: false,
            })
        );
        assert!(notification_from_event_json(json).unwrap().is_none());
    }

    #[test]
    fn ignores_missing_pane_id() {
        let json = r#"{
            "data": {
                "agent_status": "blocked",
                "agent": "Codex"
            }
        }"#;

        assert!(notification_from_event_json(json).unwrap().is_none());
    }

    #[test]
    fn ignores_unrelated_events_even_if_they_have_status_like_fields() {
        let json = r#"{
            "event": "pane_focused",
            "data": {
                "type": "pane_focused",
                "pane_id": "p1",
                "agent_status": "blocked"
            }
        }"#;

        assert!(notification_from_event_json(json).unwrap().is_none());
    }
}
