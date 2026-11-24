# A1-05/06/07 + P1-05 + U1-02：连续播放闭环

日期：2026-11-24
范围：MVP 审计建议的第 ① 项——队列、上一首/下一首、4 种播放模式、可用全局播放栏。

## 实现

### 播放器

- `biu-audio::Player` 增加 EOF 标记 `finished`：缓冲排空且解码到末尾时置位，`load`/`play`/`stop`/`seek` 清除。
- `PlayerSnapshot` 暴露 `finished`，供队列控制器判断“自然播完”而不是用户手动停止。

### 队列

新增 `src-tauri/src/queue.rs`：

- `PlayMode`：`sequential`、`list_loop`、`single_loop`、`shuffle`。
- `AudioQueue` 持有 items、当前 index、模式、静音与洗牌顺序。
- `next_index` 按模式推进；顺序模式到末尾返回 `None`，列表循环回绕，单曲循环保持当前，随机模式不立即重复当前曲。
- `prev_index` 手动上一首；单曲循环保持当前，其余回绕。
- 会话持久化到 `data_dir/player-session.json`（原子写）：重启恢复队列、当前项、模式、静音。
- 单元测试：顺序/循环、单曲循环、删除当前项、重启恢复，共 4 个。

### Tauri 命令

`src-tauri/src/commands/player.rs`：

- `player_play_queue(items, start_index, mode)`：替换队列并开始播放，只校验扩展名，不要求文件存在（坏文件进入播放错误并由自动切歌跳过）。
- `player_next`、`player_prev`、`player_jump`、`player_set_mode`。
- `player_queue`、`player_queue_add`、`player_queue_remove`、`player_queue_clear`。
- `player_set_muted`；`player_set_volume` 自动解除静音。
- `player:state` 载荷扩展：`title`、`artist`、`album`、`muted`、`mode`、`queue_index`、`queue_len`、`finished`。
- 4Hz 事件线程检测 `finished` 后按模式自动切歌；顺序模式末尾停止并清除标记。
- 连续播放失败最多跳过 3 个文件，随后停止并保留错误信息；成功进入 Playing/Paused 后失败计数清零。
- cpal 错误回调置位 `device_lost`；事件线程每 2s 尝试重建默认输出流，恢复当前曲目与位置，静音状态保持。
- `PlayerStateDto` 增加 `device_lost` 与单调递增 `seq`，前端可据此忽略乱序事件。
- 播放队列状态 `AudioQueueState` 在 setup 初始化，路径为 `paths.data_dir/player-session.json`。

### 前端

- `src/Shell.tsx` 全局播放栏：封面占位、标题/艺术家、上一首/播放暂停/下一首、可拖 seek、音量、静音、模式循环、队列开关。
- 队列面板：当前项高亮、点击跳转、单曲移除、清空。
- 订阅 `player:state`，启动时用 `player_snapshot` + `player_queue` 初始化。
- `src/pages/Library.tsx` 点击行改为把当前可见列表作为队列播放（默认 `list_loop`），不再只播单曲。
- 新增 i18n 键：暂停/继续、队列、4 种模式、静音、移除、清空等。

## 验证

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm build
```

结果：

- Rust 测试 42 个通过（含队列 4 个、IPC 8 个）。
- 前端 `tsc && vite build` 通过，JS 351.42 kB / gzip 111.55 kB。
- `pnpm tauri dev` 启动成功，无 panic；队列状态与 sessions 文件初始化正常。

## 已知限制

- 静音恢复时音量回到 1.0，未保存静音前音量。
- 删除“正在播放”的队列项只调整队列索引，不立即切歌。
- 队列面板不支持拖拽排序；无虚拟滚动，超大队列需后续优化。
- 自动切歌依赖 4Hz 轮询，切歌延迟最多约 250ms。
- 输出设备恢复为尽力而为：默认设备不可用时每 2s 重试，UI 仅显示提示；未在真机拔插 USB 声卡验证。
- 坏文件跳过是“错误后切下一首”，不是预检；连续 3 个失败后停止，第 4 首好文件不会自动继续。
- 未做 E2E 自动点击验证；需在 Windows/macOS 真机补一次“连播 3 首 + 4 模式 + 拔插声卡”手工冒烟。
