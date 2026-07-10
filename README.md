# Herdr Focus Notify

English | [简体中文](README.zh-CN.md)

Send clickable macOS notifications when a Herdr agent needs attention (`blocked`) or finishes (`done`). Clicking a notification focuses the matching Herdr pane.

Common agent notifications use bundled local icons, including Codex, Claude Code, Claude, Cursor, Gemini CLI, Gemini, GitHub Copilot, DeepSeek, Grok, Qwen, OpenCode, OpenHands, Roo Code, Cline, Windsurf, Devin, Manus, Kiro, Trae, Zencoder, Lovable, and v0.

Notifications are sent only when you are **not already looking at that pane**:

- Herdr is not the frontmost app.
- Herdr is frontmost, but another pane is focused.

If you manually focus the pane later, Herdr's `pane.focused` event removes any
still-visible notification for that pane.

When several panes notify at once, the plugin keeps a small state file under the
Herdr plugin state directory. If macOS clears the whole notification stack after
you click one alert, the plugin replays the other still-active pane notifications.
Notification bodies include the pane id, workspace id when available, and status
details so similarly named agents are easier to tell apart.

## Requirements

- macOS
- Herdr `0.7.0` or later
- [alerter](https://github.com/vjeantet/alerter), used for clickable notifications

Install alerter:

```bash
brew install vjeantet/tap/alerter
```

## Installation

Build and link locally:

```bash
cargo build --release
herdr plugin link .
```

Or install from GitHub:

```bash
herdr plugin install llmxby/herdr-focus-notify
```

## CLI

```bash
herdr-focus-notify --help
herdr-focus-notify --version
herdr-focus-notify --test
herdr-focus-notify --focus-latest
```

`--help` and `--version` print to stdout. `--test` sends a foreground test notification. `--focus-latest` performs the same action as clicking the newest still-active notification: it optionally activates `HERDR_FOCUS_NOTIFY_ACTIVATE_APP`, runs `herdr agent focus <pane>`, removes that notification, and replays any older remaining notifications if needed. Configuration or notification backend failures are printed to stderr and return a non-zero exit code. Normal plugin invocations without `HERDR_PLUGIN_EVENT_JSON` still exit quietly with `0`.

The plugin also exposes the same command as a Herdr action:

```bash
herdr plugin action invoke focus-latest --plugin herdr-focus-notify
```

That action is the easiest target for Raycast, Keyboard Maestro, Hammerspoon, skhd, or other global shortcut tools.

## Configuration

Find the plugin config directory:

```bash
herdr plugin config-dir herdr-focus-notify
```

Create a `.env` file in that directory.

The `.env` parser supports `KEY=value`, optional `export KEY=value`, single-quoted values, double-quoted values, and inline comments after unquoted values.

### Recommended

```env
HERDR_FOCUS_NOTIFY_NOTIFIER=/opt/homebrew/bin/alerter
HERDR_FOCUS_NOTIFY_ACTIVATE_APP=kitty
```

`ACTIVATE_APP` can be an app name, such as `kitty`, or a `.app` path, such as `/Applications/kitty.app`. This is easier to configure than a bundle id.

Configuring `ACTIVATE_APP` is recommended. It is used to bring your terminal app to the front when you click a notification, and to decide whether you are already looking at the current Herdr pane. The plugin skips a notification only when it can confirm both conditions: the current focused pane is the target pane, and the frontmost app is the app resolved from `ACTIVATE_APP`. If macOS frontmost-app detection fails, or the app name cannot be resolved, the plugin sends the notification to avoid missing an important state change.

### Common Options

| Variable | Description | Default |
|---|---|---|
| `HERDR_FOCUS_NOTIFY_NOTIFIER` | Notification backend path. The plugin reports an error if no executable notifier is found. | Auto-detect `alerter` |
| `HERDR_FOCUS_NOTIFY_ACTIVATE_APP` | Terminal app name or `.app` path to activate when a notification is clicked. | None |
| `HERDR_FOCUS_NOTIFY_TIMEOUT` | Seconds before a notification auto-dismisses. Set to `0` to keep it until dismissed. | `3600` |
| `HERDR_FOCUS_NOTIFY_AWAY_MODE` | Away fallback behavior: `off`, `replace`, or `also`. | `off` |
| `HERDR_FOCUS_NOTIFY_AWAY_WHEN` | When to use the away fallback. Currently only `locked` is supported. | `locked` |
| `HERDR_FOCUS_NOTIFY_AWAY_COMMAND` | External command to run for away delivery. | None |
| `HERDR_FOCUS_NOTIFY_AWAY_RECIPIENTS` | Comma-separated recipient identifiers passed to the away command. | None |

If `ACTIVATE_APP` is not configured, clicking a notification still runs `herdr agent focus <pane>`, but the plugin cannot reliably tell whether the frontmost app is the terminal that hosts Herdr, so it may send extra notifications.

### Away fallback

If you want desktop alerts while you are active, but an IM/bot fallback when your Mac session is locked, configure an away command:

```env
HERDR_FOCUS_NOTIFY_AWAY_MODE=replace
HERDR_FOCUS_NOTIFY_AWAY_WHEN=locked
HERDR_FOCUS_NOTIFY_AWAY_COMMAND=/absolute/path/to/repo/helpers/dx-notify-helper/run.sh
HERDR_FOCUS_NOTIFY_AWAY_RECIPIENTS=linmiaobin
```

When the away condition matches, the plugin exports these environment variables to the command and executes it:

- `HERDR_FOCUS_NOTIFY_TITLE`
- `HERDR_FOCUS_NOTIFY_BODY`
- `HERDR_FOCUS_NOTIFY_STATUS`
- `HERDR_FOCUS_NOTIFY_PANE_ID`
- `HERDR_FOCUS_NOTIFY_GROUP`
- `HERDR_FOCUS_NOTIFY_RECIPIENTS`

`replace` skips the desktop notification and only runs the away command. `also` runs both. If the lock state cannot be detected, the plugin falls back to the normal desktop notification path.

### Bundled Daxiang helper

This repository includes a Java helper under `helpers/dx-notify-helper/` that sends a Daxiang robot single-chat message using the same flow already proven in internal tooling:

1. `DX_CLIENT_ID` + `DX_CLIENT_SECRET` → SSO OIDC client credentials token
2. recipient MIS list → Daxiang UID list
3. `sendChatMsgByRobot` → Markdown single chat

Build it once:

```bash
helpers/dx-notify-helper/build.sh
```

Then point `HERDR_FOCUS_NOTIFY_AWAY_COMMAND` at:

```bash
helpers/dx-notify-helper/run.sh
```

The helper requires these environment variables in the shell or launcher that starts Herdr/plugin actions:

```bash
export DX_CLIENT_ID=your_client_id
export DX_CLIENT_SECRET=your_client_secret
# optional
export DX_AUDIENCE=xm-xai
```

The helper reads the away command env injected by the plugin and sends a Markdown message containing the title, body, status, and pane id.

For troubleshooting, temporarily set `HERDR_FOCUS_NOTIFY_DEBUG=1`. To pause the plugin without unlinking it, set `HERDR_FOCUS_NOTIFY_ENABLED=0`.

Bundled agent icons are vendored from `@lobehub/icons-static-png` under the MIT license. See `assets/icons/NOTICE.md`.
