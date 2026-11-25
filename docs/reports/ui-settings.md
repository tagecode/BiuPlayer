# 界面/设置缺口（F-UI-01/09）

日期：2026-11-25

## 侧栏折叠

- `UiSettings.sidebar_collapsed` 新增，默认 false；`SettingsDto/Patch` 暴露 `sidebarCollapsed`。
- 顶栏左侧新增折叠按钮，Shell 以 `shell collapsed` 切换 64px 窄栏；导航改为“图标 + 文字”，折叠时只显示图标。
- 折叠状态写入 `settings.json`，重启恢复；设置页“界面”卡片也可切换，并通过 `biu:sidebar-collapsed` 事件即时同步 Shell。
- 宽度仍可拖动（160–400），与折叠状态互不影响；窄屏（≤860px）自动使用 64px 布局。

## 800×600 布局

- 新增 `@media (max-height: 620px)`：收紧侧栏间距、导航行高、标题栏高度与内容内边距。
- 现有 `@media (max-width: 860px)` 已隐藏侧栏文字、进度条与模式/音量滑块；内容区可横向滚动。

## 托盘状态同步（F-UI-09）

- `tray::sync_state` 监听 `player:state`：
  - tooltip 显示“曲名 - 艺术家”，无曲目时为“博音 BiuPlayer”。
  - “播放/暂停”菜单项文字随状态变为“播放”或“暂停”。
- 托盘菜单仍通过 `tray:action` 驱动前端播放命令。

## 验证

- `cargo test --workspace`：58 passed。
- clippy `-D warnings`、`cargo fmt --all --check`、`pnpm build` 通过。
- `pnpm tauri dev` 启动 30s 无 panic；托盘的动态 tooltip/菜单文字未做自动化断言。

## 未完成

- 原生窗口透明度仍需平台 API（macOS `NSWindow.alphaValue`、Windows layered window）；当前透明度是 WebView 内容透明度。
- 无边框模式未做，仍使用系统标题栏。
- 800×600 为 CSS 规则验证，未做截图/真机窗口尺寸断言。
