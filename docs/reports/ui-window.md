# 窗口置顶与界面不透明度

## 设置项

- `UiSettings.always_on_top: bool`，默认 `false`。
- `UiSettings.opacity: f64`，默认 `1.0`，范围 0.7–1.0。
- `SettingsDto` 暴露 `alwaysOnTop` / `opacity`，`SettingsPatch` 接受同名可选字段；`settings_update` 校验范围后原子写回 `settings.json`。

## 即时生效

- 置顶：前端通过 `@tauri-apps/api/window` 的 `getCurrentWindow().setAlwaysOnTop(...)` 调用；能力文件新增 `core:window:allow-set-always-on-top`。
- 不透明度：写入根节点 CSS 变量 `--ui-opacity`，`.shell { opacity: var(--ui-opacity, 1) }` 立即生效。
- 启动时 `Shell` 读取设置并应用；设置页修改后无需重启。

## 三平台说明

- 置顶使用 Tauri 官方 window API，Windows/macOS/Linux 均可用；Wayland 下置顶行为由合成器决定，不保证生效。
- 当前“不透明度”是 WebView 内容整体透明度，不是原生窗口逐像素透明；窗口背景仍不透明，因此不会出现桌面穿透。原生窗口透明需要 `transparent: true` 与平台私有 API，未启用。

## 验证

- `cargo test --workspace`：51 个测试通过；clippy/fmt 通过。
- `pnpm build`：JS 397.98 kB / gzip 122.81 kB。
- `pnpm tauri build --no-bundle`：release 构建通过，能力文件校验通过。
- 未做真机置顶/透明度截图与 Wayland 人工验收。

## 限制

- 无边框、窗口透明度（原生）未做。
- 置顶状态不随主题/皮肤变化；多窗口只作用于主窗口。

## 后续：无边框窗口与原生透明度（2026-11-25）

### 设置项

- `UiSettings.frameless`：无边框窗口，默认关闭；`SettingsDto`/`SettingsPatch` 暴露 `frameless`，设置页“界面”卡片可切换并即时生效。
- `UiSettings.opacity` 复用原有 0.7–1.0 滑块，但优先调用原生窗口透明度；平台不支持时由 `.shell { opacity: var(--ui-opacity) }` 降级。

### 原生透明度

- 新命令 `set_window_opacity(opacity)`：
  - macOS：`NSWindow.setAlphaValue:`（objc2 `msg_send!`）。
  - Windows：`WS_EX_LAYERED` + `SetLayeredWindowAttributes`。
  - Linux：返回 `false`，前端退回 CSS。
- 前端 `src/window.ts` 的 `applyOpacity` 根据返回值决定是否把 `--ui-opacity` 设为 1，避免 CSS 与原生透明度叠加。
- 实测：把 `settings.json` 的 `ui.opacity` 设为 0.8 后重启，CoreGraphics 查询到窗口 alpha `0.800000011920929`；恢复 1.0 后 alpha 为 1。

### 无边框窗口

- 新命令 `set_window_frameless(frameless)`，内部调用 `Window::set_decorations(!frameless)`；Shell 启动读取设置并应用，设置页切换时广播 `biu:frameless`。
- 为无边框模式提供 `window_minimize`、`window_toggle_maximize`、`window_close` 命令，避免前端依赖窗口权限。
- 标题栏增加 `data-tauri-drag-region`；无边框时右侧显示“最小化 / 最大化还原 / 关闭”三个自绘按钮。
- 实测：`frameless=true` 启动后窗口仍存在，日志出现 `window decorations updated frameless=true`；窗口控制按钮未做自动化点击验证。
- 已知边界：macOS 关闭按钮仍走“关闭到托盘”；Linux 置顶/透明度受合成器限制。

### 验证

- 重新生成 `src/bindings.ts`，新增 5 个命令与 `SettingsDto.frameless`。
- `cargo test --workspace` 59 passed；clippy `-D warnings`、`cargo fmt --all --check`、`pnpm build` 通过。
- 前端 `pnpm build`：JS 446.51 kB / gzip 134.89 kB。
