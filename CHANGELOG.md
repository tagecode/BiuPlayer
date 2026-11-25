# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与语义化版本。

## [Unreleased]

### Added

- 全局搜索页与顶栏搜索入口，支持高亮命中与队列播放。
- 启动时检测上次崩溃日志，提示导出诊断包。
- 媒体库支持手动排序（上移/下移并持久化）。
- 设置中心分类导航与 URL 锚点，分类口径与 MVP 实现对齐。
- 无边框窗口与自定义窗口控制按钮。
- macOS/Windows 原生窗口透明度，Linux 自动降级为内容透明度。
- 媒体库扫描默认跳过隐藏文件/目录，支持设置页配置排除目录；文件夹详情可整体移出媒体库。
- 搜索结果在标题/艺术家/专辑列高亮命中词。
- 侧栏可折叠并记忆状态；800×600 窗口使用紧凑布局；托盘 tooltip 与播放菜单随播放状态更新。
- 播放列表本身支持拖拽手动排序，顺序持久化并参与启动加载。
- 队列面板支持拖拽调整曲目顺序，当前播放项保持选中。
- 将文件/目录拖到播放列表详情页时直接加入该列表，不再触发播放或扫描。
- 播放列表复制、拖拽排序、多选批量删除、撤销与“下一首播放”插播。
- 曲目右键加入播放列表，以及 `--playlist` 命令行入口。
- 媒体库新增流派、年代、文件夹视图；艺术家页展示专辑；10 万条搜索基准。
- 全前端三语本地化：主界面、媒体库、播放列表、播放页、桌面歌词与 M0 验证台。
- 后端错误码到用户文案的映射（`E_INVALID_ARGUMENT`/`E_AUDIO_DEVICE`/`E_INTERNAL`/`E_WINAMP`）。

- 曲目表虚拟滚动与标题/艺术家/专辑/时长/添加时间/路径列排序。
- 拖放音频播放、拖放目录扫描，以及第二实例文件参数转交原窗口。
- 设置页音频输出设备枚举与切换，切换后恢复当前曲目位置。
- 全局播放栏停止按钮。
- 设置页“关于”：版本、系统、Tauri 版本、检查更新、开源许可；新增 MIT LICENSE。
- Windows 绿色版写入 `portable.flag`，便携模式生效。
- 断点续播：常驻保存当前曲目、进度与播放状态（`player-progress.json`，最多每秒一次），启动后按设置恢复曲目、进度、输出设备与播放状态。
- 前端质量门：`pnpm typecheck`、`pnpm lint`（ESLint + typescript-eslint）、`pnpm test`（Vitest），CI 三个平台同步执行。
- 媒体库列表新增“添加时间”“路径”两列与排序入口，补齐 7 列排序；空目录输入框旁新增“浏览…”原生目录选择器。
- 播放列表详情页改为窗口化渲染，万级列表不再一次性挂载全部行。
- 托盘菜单、播放状态与 tooltip 支持简体中文/繁体中文/英文，切换语言立即生效；启动时按 `settings.json` 应用语言。
- 发布产物统一为 `BiuPlayer_<version>_<os>_<arch>`，新增版本一致性检查（`pnpm check:version`，CI + 发布流程）；Release 正文从 CHANGELOG 对应版本段落提取。
- 新增 51 项 MVP 验收矩阵（`docs/reports/mvp-acceptance-matrix.md`）、零出站静态审计（`docs/reports/privacy-egress.md`）与 8 小时长稳采样脚本（`scripts/soak.ps1`）。
- 媒体库新增“专辑艺术家”视图：schema v5 增加 `tracks.album_artist`，扫描读取 `AlbumArtist` 标签，按专辑艺术家聚合并可进入详情页播放。
- 标签格式夹具测试：ID3v1、ID3v2、APEv2、Vorbis Comments、MP4 atoms、WAV ID3 均由 lofty 写入后扫描回读（`cargo test -p biu-library --test tag_formats`）。
- UI-15 基准：`pnpm test:perf` 在生产构建上以 10 万行媒体库测滚动帧率（本机 60fps、DOM 行数 <80）。
- 覆盖率门禁：前端 `pnpm test:coverage`（`theme.ts`/`lyricsTiming.ts` 行覆盖阈值），CI 新增 `cargo llvm-cov` 行覆盖率作业（audio/library/lyrics/core，阈值 45，实测 66.44%）。

### Fixed

- `settings.json` 在 Windows 上不再先删后改名：改为 rename 失败时把旧文件移到旁路并回滚，避免写入中断丢失配置；损坏配置按验收要求备份为 `settings.json.bak`。
- 无边框开关现在会写入设置并在重启后恢复（此前 `settings_update` 丢弃 `frameless` 字段）。
- 主题切换与设置页、系统主题保持同步，并回写 `settings.json`；首帧由 `index.html` 内联脚本决定主题，避免浅色主题启动时闪深色。
- 浅色主题补齐全部语义色变量与硬编码颜色，侧栏边框、hover、输入框、卡片与封面不再残留深色值。
- `cargo test -p biuplayer` 在 Windows 可运行：测试目标不再启用 `common-controls-v6`，避免未带 comctl32 v6 manifest 的测试二进制在加载 `TaskDialogIndirect` 时以 `STATUS_ENTRYPOINT_NOT_FOUND` 退出；CI 不再 `--exclude biuplayer`。
- 设置页移除尚未实现的 Gapless、交叉淡化、列表密度与减少动效开关，并为“启动时最小化”标注需重启。
- 搜索结果补齐艺术家与专辑字段（此前统一显示“未知艺术家/未知专辑”），并支持按文件名检索。
- 诊断包与崩溃日志脱敏：数据/缓存/日志目录、用户名与家目录替换为占位符，settings.json 与日志同样处理；崩溃文件名改用毫秒时间戳，避免同秒 panic 互相覆盖。
- 播放页歌词偏移真正生效，并以 100ms 插值推进当前行（此前固定 +80ms 且 `lyricsOffsetMs` 只在桌面歌词生效）。

### Planned

- 设置中心补齐 DSP、快捷键、媒体库、高级分节。
- 桌面歌词点击穿透、位置/透明度持久化。
- E2E 与 10 万曲库长稳测试。
- 插件/皮肤运行时（P1）。

## [0.1.0] - 2026-11-24

首个可用的 MVP 预览版，未签名、未公证。

### Added

- 本地媒体库：添加目录、递归扫描、暂停/取消/断点续扫、增量跳过未变化文件。
- 7 种格式解码：MP3、FLAC、WAV、AAC/M4A、ALAC、OGG Vorbis、AIFF。
- 播放内核：cpal 共享输出、seek、音量、暂停/继续、播放队列、上一首/下一首、顺序/列表循环/单曲循环/随机。
- 播放列表：创建、重命名、删除、加入/移除、上移/下移、整列表播放。
- 曲库浏览：曲目表、专辑/艺术家聚合与详情、拼音全拼/首字母搜索（FTS5）、内嵌封面提取与缓存。
- 歌词：同名 LRC 优先、内嵌歌词回退、编码回退、播放页同步滚动、全局偏移、桌面置顶歌词窗口与字号/颜色设置。
- 设置：常规/播放/歌词/界面分节、`settings.json` 原子写入、主题跟随系统、语言切换。
- 系统集成：托盘菜单、单实例、关闭到托盘、启动最小化。
- 诊断：按日滚动日志、panic 崩溃文件、诊断 zip（不含曲库/音频）。
- 三平台构建：Windows NSIS + 绿色 zip、macOS x64/arm64 dmg、Linux AppImage/deb/rpm。

### Fixed

- Windows 下 SQLite 测试临时目录删除失败。
- Windows 测试二进制缺少 `WebView2Loader.dll`。
- 封面 `cover_path` 为 NULL 时的读取错误。
- 队列切歌后状态事件乱序（`seq`）。
- Linux 运行冒烟日志路径与超时。

### Known limitations

- 安装包未签名：macOS 需右键打开或手动移除隔离属性；Windows SmartScreen 会提示。
- 不支持内嵌 CUE 分轨、DSD 原生输出、独占/位完美输出。
- 歌词不支持逐字、双语、打轴编辑器。
- 桌面歌词不支持点击穿透与位置记忆。
- 未实现插件/皮肤运行时，相关页面为占位。
- 未做 10 万真实曲库与 24 小时长稳测试。

[Unreleased]: https://github.com/tagecode/BiuPlayer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tagecode/BiuPlayer/releases/tag/v0.1.0
