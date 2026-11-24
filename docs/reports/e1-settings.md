# E1 设置中心与主题跟随

日期：2026-11-24

## 范围

- `/settings/:section` 设置中心，首批实现常规、播放、界面三组。
- 设置持久化到 `settings.json`，通过 Tauri 命令读写，前端不直接碰文件。
- 主题支持跟随系统/深色/浅色，系统切换时实时更新。

## 后端

新增 `src-tauri/src/commands/settings.rs`：

- `SettingsState { path, settings: Mutex<Settings> }`，启动时加载一次。
- `SettingsDto`：语言、主题、启动最小化、默认播放模式、淡入淡出、无缝、恢复上次、列表密度、减少动效；时间/数值用 f64 以兼容 specta。
- `SettingsPatch`：全部可选，逐项校验：
  - 语言 `zh-CN` / `zh-TW` / `en`
  - 主题 `system` / `light` / `dark`
  - 模式 `sequential` / `list_loop` / `single_loop` / `shuffle`
  - 淡入淡出 0–10000 ms
  - 密度 `comfortable` / `compact`
- `settings_update` 成功后调用 `biu_core::save_settings`（临时文件 + fsync + rename），返回最新 DTO。

`src-tauri/src/lib.rs`：setup 注册 `SettingsState`，命令 `settings_get`、`settings_update`。

## 前端

- `src/pages/Settings.tsx`：三张卡片，改动即保存；错误以 `code: message` 显示。
- `src/theme.ts`：`applyTheme(mode)` 解析 system，写 `data-theme` 与 `localStorage["biu.theme"]`/`["biu.themeMode"]`；`initTheme()` 在启动时应用并监听系统主题变化。
- 语言切换同时更新 `I18nProvider` 与后端设置。
- 路由 `/settings/general` 等指向真实页面。
- 样式：`.settings-grid`、`.settings-card`、`.setting-row`。

## 验证

- `pnpm build` 通过：JS 372.74 kB / gzip 115.92 kB，CSS 17.11 kB。
- `cargo test --workspace` 45 个通过；clippy `-D warnings`、fmt 通过。
- `pnpm tauri dev` 启动正常；设置页路由与命令绑定通过类型检查，尚未做 UI 自动化点击。

## 已知限制

- 音频设备、DSP、歌词、快捷键、媒体库、高级等分节仍是占位说明。
- 启动最小化仅保存配置，托盘尚未实现，所以暂不生效。
- 淡入淡出/无缝是配置项，播放内核尚未消费。
- 设置项未做乐观更新回滚；保存失败时界面保留旧值。
