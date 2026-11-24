# M0-03 Tauri 壳、IPC 与日志报告

- 日期：2026-11-24
- 环境：macOS 26.5.1（x86_64）、Tauri 2.12.7、Rust 1.99.1、React 19 + Vite 7
- 结论：**命令/事件双向可用，1000 事件无丢失；tracing 已落盘；TypeScript 绑定已生成。**

## 1. 已实现

| 能力 | 实现 |
|---|---|
| 命令 | `app_info`、`app_ping`、`emit_test_events`、`audio_devices` |
| 事件 | `test:tick`（`TestTick { seq, sent_at_ms }`），1–10000 可控 |
| 类型绑定 | `tauri-specta` 生成 `src/bindings.ts`，前端只 import 生成类型 |
| 日志 | `tracing` + `tracing-appender` 按天滚动到应用日志目录 |
| 前端 | M0 验证台：环境信息、Ping、1000 事件计数、设备枚举、检查清单 |

启动日志示例：

```text
2026-11-24T01:04:52.902618Z  INFO biuplayer_lib::logging: BiuPlayer logging initialized version=0.1.0 log_dir=/Users/tagecode/Library/Logs/com.tagecode.biuplayer
```

macOS 日志文件：`~/Library/Logs/com.tagecode.biuplayer/biu.log.YYYY-MM-DD`。

## 2. 验收证据

```text
$ cargo test -p biuplayer --test ipc -- --nocapture
running 2 tests
test ping_echoes_payload ... ok
test emits_1000_events_without_loss ... ok
```

- `emits_1000_events_without_loss` 使用 Tauri mock app 注册 `test:tick` 监听，直接调用 `emit_test_events_impl(1000)`，断言发送 1000、接收 1000。
- 前端按钮 `发送 1000 事件` 同样统计 `received`、`burst`、`missing`，用于人工复核；1000/1000 且无缺口时显示“通过”。
- `pnpm build` 通过；`pnpm tauri dev` 可启动窗口并写入日志。

## 3. 决策与偏差

- **Rust 工具链固定为 1.99.1**：`specta 2.0.0-rc.28` 使用了 `std::fmt::from_fn`，在 Rust 1.92 稳定版上报 `E0658`（unstable `debug_closure_helpers`）；1.99.1 已稳定该 API。仓库新增 `rust-toolchain.toml`，CI 同步 1.99.1。
- **时间戳用 f64 毫秒**：specta-typescript 默认禁止 `u64` 导出（避免 BigInt 精度问题），`server_time_ms`/`sent_at_ms` 改为 `f64`；前端类型为 `number | null`。
- **事件不做频谱/节流**：M0 只验证不丢失；`player:state` 的 4 Hz 节流在 A1-07 实现。

## 4. 未覆盖 / 后续

- 未验证 1000 事件在真实窗口 + 频繁重渲染下的 UI 帧率；A1-07 做 1000 次切歌压测。
- 未实现错误 envelope、取消/超时、Channel；属 E1-04。
- 未做日志滚动保留/诊断包；属 E1-03。
