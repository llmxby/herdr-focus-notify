# Herdr Focus Notify

Clickable macOS desktop notifications for Herdr agent status changes.

When an agent becomes `blocked` or `done`, this plugin sends a local desktop notification. Clicking the notification runs:

```bash
herdr agent focus <pane_id>
```

That brings Herdr back to the relevant agent pane instead of only telling you which pane needs attention.

## Requirements

- macOS
- Herdr `0.7.0` or newer
- Rust toolchain for building the plugin
- `terminal-notifier`

Install `terminal-notifier` with Homebrew:

```bash
brew install terminal-notifier
```

## Install

Build and link the plugin:

```bash
cargo build --release
herdr plugin link .
```

Herdr can also run the build command from `herdr-plugin.toml` when linking/installing, depending on your Herdr version and workflow.

## Test

Invoke the built-in test action:

```bash
herdr plugin action invoke test --plugin local.herdr-focus-notify
```

You should see a notification. Clicking it attempts to focus `test-pane`, so the click action itself may fail unless that pane exists, but it verifies that macOS receives the clickable notification.

## Configuration

Create or edit the plugin config `.env` file:

```bash
CONFIG_DIR="$(herdr plugin config-dir local.herdr-focus-notify)"
cp .env.example "$CONFIG_DIR/.env"
```

Useful options:

```dotenv
HERDR_FOCUS_NOTIFY_ENABLED=1
HERDR_FOCUS_NOTIFY_STATUSES=blocked,done
HERDR_FOCUS_NOTIFY_NOTIFIER=/opt/homebrew/bin/terminal-notifier
HERDR_FOCUS_NOTIFY_DEBUG=1
```

The plugin also uses `HERDR_BIN_PATH` when Herdr provides it. If that environment variable is missing, it falls back to `herdr`.

## How It Works

Herdr invokes the plugin for `pane.agent_status_changed`. The plugin reads `HERDR_PLUGIN_EVENT_JSON`, extracts `data.pane_id`, writes a small focus script into `HERDR_PLUGIN_STATE_DIR`, and passes that script to `terminal-notifier -execute`.

The script indirection keeps shell quoting simple and lets the click action run after the plugin process exits.

