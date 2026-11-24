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

## 桌面歌词窗口（补充）

- 命令 `desktop_lyrics_toggle`：首次调用创建 `desktop-lyrics` 窗口，之后在显示/隐藏之间切换。
- 窗口参数：无边框、置顶、跳过任务栏、880×180、最小 480×120、可缩放。
- 独立路由 `/desktop-lyrics`，不套主 Shell；页面订阅 `player:state`，显示标题/艺术家、当前行、下一行，整窗可拖动。
- 播放入口在正在播放页歌词卡片右上角“桌面歌词”。
- 验证：`pnpm build` 通过（JS 375.14 kB/gzip 116.62 kB），Rust workspace 41 个测试通过，clippy/fmt 通过。
- 已知限制：无点击穿透、无逐字高亮、无字体/颜色/位置持久化、无关闭按钮（需从主窗口再次点击隐藏）、多显示器位置未记忆。

## 内嵌歌词回退、偏移与样式（补充）

- `biu_lyrics::load_for_audio` 先找同名 `.lrc`，找不到再读标签 `ItemKey::Lyrics`；返回 `source = sidecar | embedded`，播放页头部显示来源。
- 新增集成测试 `falls_back_to_embedded_lyrics`：往 MP3 写入 `ItemKey::Lyrics`，不写 `.lrc`，仍能加载并解析。
- 设置新增：
  - `lyrics.offset_ms`：全局偏移，范围 -10000～10000 ms；播放页当前行计算变为 `position + 80 + offset`。
  - `lyrics.font_size`：桌面歌词字号，16～96。
  - `lyrics.desktop_color`：桌面歌词颜色，`#RRGGBB`。
- 设置页新增“歌词”卡片；修改后 `settings_update` 发 `settings:changed`，桌面歌词窗口实时刷新字号/颜色/偏移。
- 验证：`pnpm build` 通过（JS 376.51 kB/gzip 117.02 kB），clippy/fmt 通过，歌词集成测试 3 个通过。
- 限制：偏移是全局的，不按曲目记忆；内嵌歌词只取第一个歌词标签；不做双语/逐字。
