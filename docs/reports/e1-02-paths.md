# E1-02 路径、配置与便携模式验证

日期：2026-11-24
任务：E1-02 配置、路径、便携模式（2 人日）

## 实现

- 新增 `src-tauri/crates/biu-core/src/paths.rs`
  - `AppPaths::resolve(exe_dir)`：检测 `portable.flag`；否则按平台解析数据/缓存目录。
  - 平台目录与 `docs/Arch.md` §12.1 一致：
    - Windows：`%APPDATA%\BiuPlayer` / `%LOCALAPPDATA%\BiuPlayer\cache`
    - macOS：`~/Library/Application Support/BiuPlayer` / `~/Library/Caches/BiuPlayer`
    - Linux：`$XDG_DATA_HOME/biuplayer` 或 `~/.local/share/biuplayer` / `$XDG_CACHE_HOME/biuplayer` 或 `~/.cache/biuplayer`
    - 便携：`<exe>/data`、`<exe>/data/cache`
  - `AppPaths::ensure()` 创建 data/cache/logs/backups。
- 新增 `src-tauri/crates/biu-core/src/settings.rs`
  - `Settings` 与 `settings.json` 结构；`schemaVersion = 1`。
  - `load_settings`：缺失用默认值；损坏文件隔离为 `settings.corrupt.<ts>.json` 后回退默认；更高版本只读。
  - `save_settings`：临时文件 + fsync + rename；旧文件保留为 `settings.json.bak`；未知字段通过 `serde(flatten)` 保留。
- 接入 Tauri：
  - `setup` 解析路径、创建目录、初始化日志、首次启动写入默认 `settings.json`。
  - 新增 `app_paths` 命令与 `AppPathsInfo` DTO；`app_info.log_dir` 改为同一路径来源。
  - 前端“运行环境”卡展示数据目录、日志目录与便携模式。

## 验证

```bash
cd src-tauri
cargo test -p biu-core
cargo clippy --workspace --all-targets -- -D warnings
cd ..
pnpm build
```

结果：

- `cargo test -p biu-core`：10 passed（路径/便携/默认目录/设置往返/备份/损坏隔离/新版本只读）。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `pnpm build`：通过（Vite 7.3.8）。
- 本机启动 `target/debug/biuplayer`：
  - 日志：`~/Library/Application Support/BiuPlayer/logs/biu.log.2026-11-24`
  - 设置：`~/Library/Application Support/BiuPlayer/settings.json`
  - 首次启动自动写入 `schemaVersion=1`、`language=zh-CN`、`sidebarWidth=220`。

## 平台状态

| 平台 | 路径实现 | 本机运行 | 备注 |
|---|---|---|---|
| macOS 15 (arm64) | 是 | 是 | 已生成 data/logs/backups/settings.json |
| Windows 10/11 | 是 | 未运行 | 环境变量分支有单元测试覆盖，待 CI/VM 验证 |
| Linux 22.04+ | 是 | 未运行 | XDG 分支，待 CI/VM 验证 |

## 已知限制

- 尚未把 `Settings` 暴露为前端可读写命令；当前只做首次创建与内部加载。E1-02 验收只要求路径与配置可用，设置中心属于 S1-02。
- 日志按天滚动，但尚未接入 panic hook 与诊断包（E1-03）。
- `save_settings` 在 Windows 上先删除旧文件再 rename，极端断电窗口内仍可能丢失旧文件；已有 `.bak` 兜底，后续可换 `ReplaceFileW`。
