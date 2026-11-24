# E1-04 IPC 契约与错误码

日期：2026-11-24
任务：E1-04（2 人日）
状态：**基础契约完成**（命令/事件/错误 envelope/TS 绑定；超时与取消未实现）

## 1. 契约

命令（`src-tauri/src/commands/system.rs`）：

| 命令 | 返回 | 说明 |
|---|---|---|
| `app_info` | `AppInfo` | 名称、版本、Tauri 版本、OS/arch、日志目录 |
| `app_paths` | `AppPathsInfo` | 数据/缓存/日志目录、settings、DB、便携模式 |
| `app_ping` | `PingReply` | 回声 + `server_time_ms` |
| `audio_devices` | `Vec<AudioDevice>` | cpal 输出设备 |
| `emit_test_events` | `u32` | 发 `test:tick`，上限 10000 |
| `export_diagnostics` | `String` | 生成诊断 zip，返回路径 |

事件：

| 事件 | 载荷 | 频率 |
|---|---|---|
| `test:tick` | `{ seq, sent_at_ms }` | 仅测试命令触发 |
| `player:state` | 待 A1-07 | 4Hz |

## 2. 错误 envelope

`biu_ipc::CommandError`：

```json
{ "code": "E_INVALID_ARGUMENT", "message": "count must be between 0 and 10000" }
```

- 已定义：`E_INTERNAL`、`E_INVALID_ARGUMENT`、`E_AUDIO_DEVICE`、`E_WINAMP`。
- 前端 `formatCommandError` 统一显示 `code: message`，不再出现 `[object Object]`。
- `emit_test_events` 超过 10000 返回 `E_INVALID_ARGUMENT`，不再静默截断。
- 音频设备失败映射为 `E_AUDIO_DEVICE`；诊断包失败映射为 `E_INTERNAL`。

## 3. 验证

```bash
cd src-tauri
cargo test -p biuplayer --test ipc
cargo test -p biuplayer --lib exports_typescript_bindings
cd ..
pnpm build
```

结果：

- `emits_1000_events_without_loss`：通过。
- `ping_echoes_payload`：通过。
- `rejects_oversized_event_burst_with_stable_code`：通过，校验 `code` 与序列化 JSON。
- `src/bindings.ts` 重新生成，含 `CommandError` 类型；`pnpm build` 通过。

## 4. 未完成

- 超时/取消：当前命令均为毫秒级；长任务（扫描、导入）在 E1/D1 引入时补 `Job` 句柄与 `E_TIMEOUT`/`E_CANCELLED`。
- 错误码未集中到前端常量；目前以字符串比较。
- 事件 `player:state` 尚未实现（A1-07）。
