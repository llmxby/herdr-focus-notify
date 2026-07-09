use crate::util::sanitize_group_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusNotification {
    pub(crate) pane_id: String,
    pub(crate) status: String,
    pub(crate) title: String,
    pub(crate) body: String,
    pub(crate) group: String,
    pub(crate) app_icon: Option<String>,
}

pub(crate) fn notification_group_for_pane(pane_id: &str) -> String {
    format!("herdr-{}", sanitize_group_id(pane_id))
}
