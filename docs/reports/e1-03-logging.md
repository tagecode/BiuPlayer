# E1-03 日志、panic 与诊断包

日期：2026-11-24
任务：E1-03 日志、panic、诊断包（2 人日）

## 实现

- `src-tauri/src/diagnostics.rs`
  - `install_panic_hook(log_dir)`：panic 时写入 `logs/crash-<unix>.log`，包含时间、线程、位置、payload 与 `Backtrace`，随后调用默认 hook，终端仍可见。
  - `export_diagnostics(paths, version)`：生成 `data/diagnostics/biu-diagnostics-<unix>.zip`。
    - `system.json`：版本、OS/架构、便携模式、数据/缓存/日志目录、settings schema、DB 是否存在。
    - `settings.json`：原样副本（若存在）。
    - `logs/0-…`：最近 5 个日志文件，每个最多 2 MiB。
    - `README.txt`：说明不包含音频与数据库。
  - 明确不打包 `library.db`、音频文件、封面缓存，避免诊断包泄露曲库。
- 接入：
  - `setup` 在日志初始化前安装 panic hook。
  - 新增 `export_diagnostics` 命令；前端“运行环境”卡增加“导出诊断包”按钮与路径显示。
- 日志仍按天滚动到 `data/logs/biu.log.YYYY-MM-DD`；日志级别可用 `RUST_LOG` 覆盖。

## 验证

```bash
cd src-tauri
cargo test -p biuplayer --lib diagnostics
cargo clippy --workspace --all-targets -- -D warnings
cd ..
pnpm build
```

结果：

- `diagnostics_include_settings_logs_and_metadata`：通过；校验 zip 内含 `system.json`、`settings.json`、`logs/`、`README.txt`，且 `system.json` 含版本与 portable 字段。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `pnpm build`：通过。
- 本机启动 `target/debug/biuplayer`：日志正常写入 `~/Library/Application Support/BiuPlayer/logs/`，无 panic。

## 验收边界

- 已覆盖：日志落盘、panic 文件、诊断包生成与内容白名单。
- 未覆盖：真实 GUI panic 的弹窗提示（Tauri 无内置崩溃对话框，后续可在前端错误边界补提示）；诊断包上传（MVP 不做遥测/自动上传，只本地导出）。
- 诊断包大小取决于日志量；当前最多 5×2 MiB，未压缩前上限约 10 MiB。
