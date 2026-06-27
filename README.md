# Herdr Focus Notify

Clickable macOS desktop notifications for Herdr agent status changes.

When a Herdr agent becomes `blocked` or `done`, this plugin sends a local desktop notification. Clicking the notification focuses the matching Herdr agent pane by running:

```bash
herdr agent focus <pane_id>
```

## Installation

Install the required macOS notifier:

```bash
brew install terminal-notifier
```

Build and link the plugin:

```bash
cargo build --release
herdr plugin link .
```

The plugin requires Herdr `0.7.0` or newer.
