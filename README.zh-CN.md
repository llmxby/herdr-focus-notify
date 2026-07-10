# Herdr Focus Notify

[English](README.md) | 简体中文

当 Herdr agent 需要关注（`blocked`）或完成（`done`）时，发送可点击的 macOS 通知。点击通知会聚焦到对应的 Herdr pane。

常见 agent 会自动使用插件内置的本地图标，包括 Codex、Claude Code、Claude、Cursor、Gemini CLI、Gemini、GitHub Copilot、DeepSeek、Grok、Qwen、OpenCode、OpenHands、Roo Code、Cline、Windsurf、Devin、Manus、Kiro、Trae、Zencoder、Lovable、v0。

通知只会在你**没有在看这个 pane** 时发送：

- 你当前在其它 App（Herdr 不在前台）
- 你在 Herdr 里，但聚焦的是另一个 pane

如果之后你手动切到对应 pane，插件会监听 Herdr 的 `pane.focused` 事件，自动关闭这个 pane 还停留着的通知。

多个 pane 同时有通知时，插件会在 Herdr 插件 state 目录里记录一份轻量状态。如果 macOS 在你点击其中一个通知后把整组通知都清掉，插件会把其它仍然活跃的 pane 通知补回来。通知正文也会带上 pane id、可用时的 workspace id，以及状态信息，方便区分多个 Claude/Codex pane。

## 前提条件

- macOS
- Herdr `0.7.0` 或更新版本
- [alerter](https://github.com/vjeantet/alerter)：用于显示可点击通知

安装 alerter：

```bash
brew install vjeantet/tap/alerter
```

## 安装

本地构建并链接：

```bash
cargo build --release
herdr plugin link .
```

或从 GitHub 安装：

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

`--help` 和 `--version` 会输出到 stdout。`--test` 会发送一条前台测试通知。`--focus-latest` 会执行与“点击最新一条仍然活跃的通知”相同的动作：按需激活 `HERDR_FOCUS_NOTIFY_ACTIVATE_APP`、执行 `herdr agent focus <pane>`、移除这条通知，并在需要时把更早的剩余通知补回。配置错误或通知后端错误会输出到 stderr，并返回非零退出码。普通插件调用如果没有 `HERDR_PLUGIN_EVENT_JSON`，仍会安静地以 `0` 退出。

插件同时把这个能力暴露成了一个 Herdr action：

```bash
herdr plugin action invoke focus-latest --plugin herdr-focus-notify
```

后续无论你用 Raycast、Keyboard Maestro、Hammerspoon、skhd 还是别的全局快捷键工具，直接绑定这条命令通常最方便。

## 配置

找到插件配置目录：

```bash
herdr plugin config-dir herdr-focus-notify
```

在该目录下创建 `.env` 文件。

`.env` 解析支持 `KEY=value`、可选的 `export KEY=value`、单引号值、双引号值，以及未加引号值后面的行尾注释。

### 推荐配置

```env
HERDR_FOCUS_NOTIFY_NOTIFIER=/opt/homebrew/bin/alerter
HERDR_FOCUS_NOTIFY_ACTIVATE_APP=kitty
```

`ACTIVATE_APP` 填 app 名称（如 `kitty`）、`.app` 路径（如 `/Applications/kitty.app`）都可以，比 bundle id 更容易找到。

建议配置 `ACTIVATE_APP`。它用于点击通知时把终端 App 提到前台，也用于判断你是否正在看当前 Herdr pane。只有在插件能确认「当前 focused pane 是这个 pane」并且「前台 App 是 `ACTIVATE_APP` 对应的 App」时，才会跳过通知；如果 macOS 前台 App 查询失败或 App 名称无法解析，插件会选择发送通知，避免漏掉需要关注的状态。

### 常用配置

| 变量 | 说明 | 默认值 |
|---|---|---|
| `HERDR_FOCUS_NOTIFY_NOTIFIER` | 通知后端路径；找不到可执行通知后端时会报错 | 自动查找 `alerter` |
| `HERDR_FOCUS_NOTIFY_ACTIVATE_APP` | 点击通知时激活的终端 app 名或 `.app` 路径 | 无 |
| `HERDR_FOCUS_NOTIFY_TIMEOUT` | 通知自动消失时间（秒），`0` 表示不自动消失 | `3600` |
| `HERDR_FOCUS_NOTIFY_AWAY_MODE` | 离席兜底行为：`off`、`replace` 或 `also` | `off` |
| `HERDR_FOCUS_NOTIFY_AWAY_WHEN` | 何时启用离席兜底；当前仅支持 `locked` | `locked` |
| `HERDR_FOCUS_NOTIFY_AWAY_COMMAND` | 离席时执行的外部命令 | 无 |
| `HERDR_FOCUS_NOTIFY_AWAY_RECIPIENTS` | 传给外部命令的逗号分隔接收人标识 | 无 |

如果没有配置 `ACTIVATE_APP`，通知点击后仍会执行 `herdr agent focus <pane>`，但插件无法可靠判断前台 App 是否就是 Herdr 所在终端，因此可能会多发通知。

### 离席兜底

如果你想在人在电脑前时继续收桌面通知，但在 Mac 已锁屏时改为走 IM / 机器人通知，可以这样配置：

```env
HERDR_FOCUS_NOTIFY_AWAY_MODE=replace
HERDR_FOCUS_NOTIFY_AWAY_WHEN=locked
HERDR_FOCUS_NOTIFY_AWAY_COMMAND=/absolute/path/to/repo/helpers/dx-notify-helper/run.sh
HERDR_FOCUS_NOTIFY_AWAY_RECIPIENTS=linmiaobin
```

当满足离席条件时，插件会给该命令注入这些环境变量：

- `HERDR_FOCUS_NOTIFY_TITLE`
- `HERDR_FOCUS_NOTIFY_BODY`
- `HERDR_FOCUS_NOTIFY_STATUS`
- `HERDR_FOCUS_NOTIFY_PANE_ID`
- `HERDR_FOCUS_NOTIFY_GROUP`
- `HERDR_FOCUS_NOTIFY_RECIPIENTS`

`replace` 表示只执行外部命令、不再发桌面通知；`also` 表示两边都发。如果锁屏状态无法检测，插件会保守地回退到原本的桌面通知路径。

### 仓库内置的大象 helper

当前仓库已经内置了一个 Java 版 helper，目录在 `helpers/dx-notify-helper/`，它会复用内部已经验证过的那套链路发大象机器人单聊：

1. `DX_CLIENT_ID` + `DX_CLIENT_SECRET` → 通过 SSO OIDC client credentials 获取 token
2. MIS 列表 → 查询大象 UID
3. 调用 `sendChatMsgByRobot` 发送 Markdown 单聊消息

先构建一次：

```bash
helpers/dx-notify-helper/build.sh
```

然后把 `HERDR_FOCUS_NOTIFY_AWAY_COMMAND` 指到：

```bash
helpers/dx-notify-helper/run.sh
```

启动 Herdr / 插件动作的 shell 环境里需要提供这些变量：

```bash
export DX_CLIENT_ID=your_client_id
export DX_CLIENT_SECRET=your_client_secret
# 可选
export DX_AUDIENCE=xm-xai
```

helper 会读取主插件注入的 away 环境变量，并把标题、正文、状态、pane id 组织成 Markdown 单聊消息发到大象。

排障时可以临时配置 `HERDR_FOCUS_NOTIFY_DEBUG=1`；需要暂停插件时可以配置 `HERDR_FOCUS_NOTIFY_ENABLED=0`。

内置 agent 图标来自 `@lobehub/icons-static-png`，许可证为 MIT。见 `assets/icons/NOTICE.md`。
