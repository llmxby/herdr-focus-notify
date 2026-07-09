use std::process::Command;

use crate::config::activate_app;

pub(crate) fn activate_configured_app() -> Result<(), String> {
    let Some(app) = activate_app() else {
        return Ok(());
    };

    let status = if app.contains('/') {
        Command::new("open")
            .arg(&app)
            .status()
            .map_err(|err| format!("failed to run open for activate app: {err}"))?
    } else {
        Command::new("open")
            .arg("-a")
            .arg(&app)
            .status()
            .map_err(|err| format!("failed to run open -a for activate app: {err}"))?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("activate app exited with {status}"))
    }
}
