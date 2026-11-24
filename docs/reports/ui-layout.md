# 侧栏调宽与窗口状态记忆

## 侧栏宽度

- `SettingsDto.sidebarWidth` / `SettingsPatch.sidebarWidth` 暴露 `ui.sidebar_width`，范围 160–400。
- `src/Shell.tsx` 启动时读取设置，把宽度写入 `.shell` 的 `--sidebar-width` CSS 变量。
- 侧栏右缘 `.sidebar-resizer` 支持指针拖动；拖动中只更新本地状态，松开后写回 `settings.json`。
- 窄屏（≤860px）仍由媒体查询强制 64px 图标栏，避免覆盖小窗口。

## 窗口状态

- 引入 `tauri-plugin-window-state`，在 Builder 中注册；自动保存/恢复窗口尺寸、位置、最大化状态。
- 与现有“启动最小化到托盘”兼容：恢复状态后再按设置隐藏窗口。

## 验证

- `cargo test --workspace`：51 个测试通过；clippy/fmt 通过。
- `pnpm build` 通过：JS 382.76 kB / gzip 118.93 kB，CSS 19.69 kB。
- 本机 `pnpm tauri dev` 启动成功，无 panic；未做多屏拔插与 DPI 缩放的截图验收。

## 限制

- 面板折叠（F-UI-01）未做。
- 无边框/透明度/置顶（F-UI-08 其余项）未做。
- 窗口状态文件由插件管理，未在设置页暴露重置入口。
