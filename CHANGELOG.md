# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 与语义化版本。

## [Unreleased]

### Planned

- 设置中心补齐音频设备、DSP、快捷键、媒体库、高级分节。
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
