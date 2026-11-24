# P1 正在播放页

日期：2026-11-24

## 范围

- `/` 由占位页替换为正在播放页（P-01）。
- 展示封面占位、标题、艺术家、专辑、进度与总时长。
- 上一首/播放暂停/下一首/4 种模式循环/音量。
- 队列前 30 首，点击跳转；当前项高亮。
- 歌词区域先给占位，说明后续接入 LRC。

## 实现

- `src/pages/NowPlaying.tsx`：订阅 `player:state`，启动时 `playerSnapshot` + `playerQueue` 初始化；与 Shell 播放栏共用同一状态源。
- 进度条直接调用 `playerSeek`，音量调用 `playerSetVolume`，模式调用 `playerSetMode`。
- 封面暂用首字母渐变块，不读取内嵌封面。
- `src/router.tsx` 将 `/` 指向 `NowPlaying`。
- 新增 `.np-stage`、`.np-cover`、`.np-controls`、`.np-queue` 等样式。

## 验证

- `pnpm build` 通过：JS 367.58 kB / gzip 114.68 kB，CSS 16.42 kB。
- `pnpm tauri dev` 启动正常，路由 `/` 渲染。
- Rust workspace 测试 44 个通过。

## 已知限制

- 封面为占位，未接内嵌封面提取。
- 歌词未实现，仅占位。
- 队列只显示前 30 首，无搜索/拖拽。
- 进度条拖动时以 4Hz 事件回写，快速拖动可能有轻微回跳。
