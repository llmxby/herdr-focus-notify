# dx-notify-helper

Bundled Daxiang single-chat helper for `herdr-focus-notify` away fallback.

## Build

```bash
./build.sh
```

## Run

The helper is usually launched by the Rust plugin through:

```bash
./run.sh
```

Required env:

- `DX_CLIENT_ID`
- `DX_CLIENT_SECRET`
- `HERDR_FOCUS_NOTIFY_TITLE`
- `HERDR_FOCUS_NOTIFY_BODY`
- `HERDR_FOCUS_NOTIFY_RECIPIENTS`

Optional env:

- `DX_AUDIENCE` (default `xm-xai`)
- `HERDR_FOCUS_NOTIFY_STATUS`
- `HERDR_FOCUS_NOTIFY_PANE_ID`
- `HERDR_FOCUS_NOTIFY_GROUP`

## What it does

1. Uses SSO OIDC client credentials to get a token.
2. Resolves recipient MIS values to Daxiang UID values.
3. Sends a Markdown single chat via `sendChatMsgByRobot`.

