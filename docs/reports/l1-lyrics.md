# 歌词：LRC 解析与同步显示

日期：2026-11-24

## 范围

- 同名 `.lrc` 自动加载。
- 标准/增强 LRC、多时间标签、元信息、offset。
- 播放页逐行滚动、当前行高亮、点击行跳转。

## 解析器

新增 `src-tauri/crates/biu-lyrics`：

- `find_sidecar(audio)`：`.lrc` 优先，其次 `.LRC`。
- `load_sidecar(audio)`：读取字节并按编码解码。
- `decode(bytes)`：
  - UTF-8 BOM / UTF-16 LE / UTF-16 BE BOM。
  - 合法 UTF-8 直接使用。
  - 否则按 GBK 解码（`encoding_rs`），覆盖常见中文歌词。
- `parse(&str)`：
  - 元信息 `[ti:]`、`[ar:]`、`[al:]`、`[by:]`、`[offset:]`。
  - 时间标签 `[mm:ss]`、`[mm:ss.xx]`、`[mm:ss.xxx]`，一行可多个。
  - 增强标签 `<mm:ss.xx>` 从显示文本中剔除。
  - 按时间排序；offset 应用到每行，负值截到 0。
- 测试：标准/增强、GBK 解码、无效行过滤与排序，共 3 个。

## 命令

`src-tauri/src/commands/lyrics.rs`：

- `lyrics_load(path) -> Option<LyricsDto>`
- 校验音频文件存在；无同名歌词返回 `null`，不报错。
- DTO 使用 f64 毫秒，避免 specta 的 u64 限制。

集成测试 `src-tauri/tests/lyrics.rs`：同名 `.lrc` 加载、offset、无歌词返回 `null`。

## 前端

- 正在播放页歌词卡片：播放曲目变化时自动加载。
- 当前行按 `position_ms + 80ms` 容差计算，自动滚动到中间。
- 点击任意行调用 `playerSeek`，播放不中断。
- 无歌词/加载中/无曲目分别显示提示。
- 样式 `.np-lyrics`、`.np-lyrics .active`。

## 验证

- `cargo test -p biu-lyrics` 3 个通过。
- `cargo test --test lyrics` 2 个通过。
- `pnpm build` 通过：JS 373.97 kB / gzip 116.29 kB。

## 已知限制

- 仅外挂同名 `.lrc`，尚未读内嵌 USLT/LYRICS/©lyr。
- 未做 offset 调整 UI、双语双行、逐字/卡拉 OK。
- 未做独立桌面歌词窗口与打轴编辑器。
- 桌面端只验证了 macOS 构建与单元测试，未做播放中的视觉验收。
