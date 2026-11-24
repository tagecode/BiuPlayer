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
