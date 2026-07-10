#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliAction {
    Event,
    Test,
    FocusLatest,
    OnClick(String),
    Help,
    Version,
}

pub(crate) fn parse_cli_args<I, S>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut action = CliAction::Event;
    let mut args = args.into_iter().peekable();

    while let Some(arg) = args.next() {
        let arg = arg.as_ref();
        match arg {
            "--test" => set_action(&mut action, CliAction::Test, arg)?,
            "--focus-latest" => set_action(&mut action, CliAction::FocusLatest, arg)?,
            "--on-click" => {
                let pane_id = args
                    .next()
                    .ok_or_else(|| "--on-click requires a pane id".to_string())?
                    .as_ref()
                    .to_string();
                if pane_id.is_empty() {
                    return Err("--on-click requires a pane id".to_string());
                }
                set_action(&mut action, CliAction::OnClick(pane_id), arg)?;
            }
            "-h" | "--help" => set_action(&mut action, CliAction::Help, arg)?,
            "-V" | "--version" => set_action(&mut action, CliAction::Version, arg)?,
            _ => {
                return Err(format!(
                    "unknown argument: {arg}; run with --help for usage"
                ));
            }
        }
    }

    Ok(action)
}

fn set_action(action: &mut CliAction, next: CliAction, arg: &str) -> Result<(), String> {
    if *action != CliAction::Event {
        return Err(format!("cannot combine {arg} with another command"));
    }

    *action = next;
    Ok(())
}

pub(crate) fn print_usage() {
    println!(
        "herdr-focus-notify {}\n\nUsage:\n  herdr-focus-notify\n  herdr-focus-notify --test\n  herdr-focus-notify --focus-latest\n  herdr-focus-notify --on-click <pane-id>\n\nOptions:\n  --test          Send a test focus notification\n  --focus-latest  Focus the most recent active notification pane\n  --on-click      Dismiss the clicked pane's notification and replay others\n  -h, --help      Show this help\n  -V, --version\n                 Show the version",
        env!("CARGO_PKG_VERSION")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cli_actions() {
        assert_eq!(
            parse_cli_args(Vec::<&str>::new()).unwrap(),
            CliAction::Event
        );
        assert_eq!(parse_cli_args(["--test"]).unwrap(), CliAction::Test);
        assert_eq!(
            parse_cli_args(["--focus-latest"]).unwrap(),
            CliAction::FocusLatest
        );
        assert_eq!(
            parse_cli_args(["--on-click", "w1:p3"]).unwrap(),
            CliAction::OnClick("w1:p3".to_string())
        );
        assert!(parse_cli_args(["--on-click"]).is_err());
        assert_eq!(parse_cli_args(["--help"]).unwrap(), CliAction::Help);
        assert_eq!(parse_cli_args(["-h"]).unwrap(), CliAction::Help);
        assert_eq!(parse_cli_args(["--version"]).unwrap(), CliAction::Version);
        assert_eq!(parse_cli_args(["-V"]).unwrap(), CliAction::Version);
    }

    #[test]
    fn rejects_unknown_or_combined_cli_args() {
        assert!(parse_cli_args(["--wat"]).is_err());
        assert!(parse_cli_args(["--test", "--help"]).is_err());
    }
}
