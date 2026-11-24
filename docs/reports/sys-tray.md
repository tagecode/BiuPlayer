# 系统托盘、单实例与关闭到托盘

日期：2026-11-24

## 范围

- 系统托盘图标与菜单。
- 单实例：重复启动只唤起已有窗口。
- 关闭主窗口时隐藏到托盘，而不是退出。
- 设置中的“启动时最小化”开始生效。

## 实现

### 托盘

`src-tauri/src/tray.rs`：

- 菜单：显示 / 隐藏、播放 / 暂停、上一首、下一首、退出 BiuPlayer。
- 左键点击托盘图标切换主窗口显示。
- 播放控制不直接操作音频线程，而是向前端发 `tray:action`，由 `Shell.tsx` 调用现有 `player_*` 命令，避免托盘与播放状态各持一份。
- 图标使用 Tauri 默认窗口图标。

### 单实例

- 引入 `tauri-plugin-single-instance`，在 Builder 中第一个注册。
- 第二次启动时回调显示并聚焦 `main` 窗口。

### 关闭到托盘

- `WindowEvent::CloseRequested` 调用 `api.prevent_close()`，然后 `window.hide()`。
- 退出只走托盘菜单“退出”或应用退出事件。

### 启动最小化

- setup 中读取 `settings.general.start_minimized`，为 true 时窗口创建后立即隐藏。

## 验证

- `cargo check`、`cargo clippy -D warnings`、`cargo fmt --check` 通过。
- `pnpm build` 通过：JS 373.12 kB / gzip 116.01 kB。
- macOS `pnpm tauri dev` 启动无 panic，日志正常初始化，托盘创建未报错。
- 未做托盘菜单点击自动化；未在 Windows/Linux 真机验证托盘行为。

## 已知限制

- 关闭到托盘是固定行为，设置里还没有“关闭时退出/最小化/询问”的选项。
- 托盘菜单文案固定中文，未接 i18n。
- 托盘没有“正在播放”标题、播放状态图标或进度。
- 单实例仅验证编译与启动，未做双开自动化测试。
- 启动最小化在 macOS 仍会保留 Dock 图标。
