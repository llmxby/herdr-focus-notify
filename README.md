# Herdr Focus Notify

Clickable macOS notifications that focus the matching Herdr agent pane.

Herdr already has native notifications. This plugin is for the missing click target: when a Herdr agent becomes `blocked` or `done`, it sends one macOS notification. Clicking the notification body or the `Focus` action runs:

```bash
open <configured terminal app>
herdr agent focus <pane_id>
```

## Prerequisites

- macOS.
- Herdr `0.7.0` or newer.
- [alerter](https://github.com/vjeantet/alerter) for reliable click callbacks.
- Optional but recommended: the bundle id or `.app` path for the terminal app that runs Herdr, such as kitty.

Install alerter:

```bash
brew install vjeantet/tap/alerter
```

The legacy `terminal-notifier` backend is display-only on modern macOS because its `-execute` callback is silently dropped on macOS 12+.

## Herdr Config

Turn off Herdr's native toast delivery to avoid duplicate notifications. Add this to `~/.config/herdr/config.toml`:

```toml
[ui.toast]
delivery = "off"
```

Reload Herdr after changing the config:

```bash
herdr server reload-config
```

## Plugin Config

The plugin reads optional settings from its Herdr plugin config directory. You can find that directory with:

```bash
herdr plugin config-dir herdr-focus-notify
```

Create a `.env` file there when you want terminal activation or debugging.

Recommended kitty setup:

```env
HERDR_FOCUS_NOTIFY_NOTIFIER=/opt/homebrew/bin/alerter
HERDR_FOCUS_NOTIFY_ACTIVATE_BUNDLE_ID=net.kovidgoyal.kitty
HERDR_FOCUS_NOTIFY_TIMEOUT=3600
```

You can also use an app path or app name:

```env
HERDR_FOCUS_NOTIFY_ACTIVATE_APP=/Applications/kitty.app
# or:
HERDR_FOCUS_NOTIFY_ACTIVATE_APP=kitty
```

Leave `HERDR_FOCUS_NOTIFY_SENDER` unset unless you intentionally want macOS to treat the notification as coming from a different app. The activation setting is enough to open the terminal app before focusing the pane.

Useful optional settings:

```env
# Print diagnostic output to focus-click.log in the plugin state dir.
HERDR_FOCUS_NOTIFY_DEBUG=1

# Defaults to blocked,done.
HERDR_FOCUS_NOTIFY_STATUSES=blocked,done

# 0 keeps alerter notifications until dismissed. Default 3600.
HERDR_FOCUS_NOTIFY_TIMEOUT=3600
```

See `.env.example` for all supported settings.

## Installation

Install and enable the plugin:

```bash
cargo build --release
herdr plugin link .
```

Or install from GitHub:

```bash
herdr plugin install yankewei/herdr-focus-notify
```

After it is enabled, Herdr provides the plugin context and Herdr binary path. The plugin also checks common macOS locations for the notifier binary. Set `HERDR_FOCUS_NOTIFY_NOTIFIER` if your alerter binary is elsewhere.

## Test

Run the test action:

```bash
herdr plugin action invoke test --plugin herdr-focus-notify
```

## Troubleshooting

If two notifications appear for one agent state change, Herdr native toast delivery is still enabled. Set:

```toml
[ui.toast]
delivery = "off"
```

If notifications do not appear, install alerter and point the plugin to it:

```env
HERDR_FOCUS_NOTIFY_NOTIFIER=/opt/homebrew/bin/alerter
```

If clicking focuses the pane but does not bring your terminal app forward, set `HERDR_FOCUS_NOTIFY_ACTIVATE_APP` or `HERDR_FOCUS_NOTIFY_ACTIVATE_BUNDLE_ID`.

If clicking does nothing, enable debug logging:

```env
HERDR_FOCUS_NOTIFY_DEBUG=1
```

Then inspect `focus-click.log` inside the plugin state directory. It records the alerter result, terminal activation exit status, and `herdr agent focus` exit status.
