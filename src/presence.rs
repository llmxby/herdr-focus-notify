use std::process::Command;

use crate::config::away_when;

pub(crate) fn should_use_away_delivery() -> bool {
    match away_when().as_str() {
        "locked" => is_session_locked().unwrap_or(false),
        _ => false,
    }
}

fn is_session_locked() -> Option<bool> {
    if let Ok(value) = std::env::var("HERDR_FOCUS_NOTIFY_TEST_SESSION_LOCKED") {
        return match value.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        };
    }

    is_session_locked_from_ioreg_output(
        &Command::new("ioreg")
            .arg("-d1")
            .arg("-w0")
            .arg("-l")
            .output()
            .ok()?
            .stdout,
    )
}

fn is_session_locked_from_ioreg_output(output: &[u8]) -> Option<bool> {
    let text = String::from_utf8(output.to_vec()).ok()?;

    if text.contains("\"IOConsoleLocked\" = Yes")
        || text.contains("\"CGSSessionScreenIsLocked\"=Yes")
        || text.contains("\"CGSSessionScreenIsLocked\" = Yes")
    {
        return Some(true);
    }

    if text.contains("\"IOConsoleLocked\" = No")
        || text.contains("\"CGSSessionScreenIsLocked\"=No")
        || text.contains("\"CGSSessionScreenIsLocked\" = No")
    {
        return Some(false);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_locked_session_from_ioreg_output() {
        let output = b"+-o Root  <class IORegistryEntry>\n  | {\n  |   \"IOConsoleLocked\" = Yes\n  |   \"IOConsoleUsers\" = ({\"CGSSessionScreenIsLocked\"=Yes})\n  | }\n";

        assert_eq!(is_session_locked_from_ioreg_output(output), Some(true));
    }

    #[test]
    fn detects_unlocked_session_from_ioreg_output() {
        let output = b"+-o Root  <class IORegistryEntry>\n  | {\n  |   \"IOConsoleLocked\" = No\n  |   \"IOConsoleUsers\" = ({\"CGSSessionScreenIsLocked\"=No})\n  | }\n";

        assert_eq!(is_session_locked_from_ioreg_output(output), Some(false));
    }

    #[test]
    fn returns_none_when_lock_state_is_missing() {
        assert_eq!(
            is_session_locked_from_ioreg_output(b"no lock markers"),
            None
        );
    }
}
