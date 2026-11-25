# 博音 BiuPlayer

本地优先、离线可用的跨平台音乐播放器。目标是用 Tauri + Rust + React 做一款“现代版千千静听”：无损播放、本地媒体库、歌词、换肤与插件，不依赖在线曲库。

- 中文名：博音音乐播放器
- 英文名：BiuPlayer
- 当前阶段：**MVP 收尾**（M0/E1/D1 完成；A1 播放、L1 媒体库、P1 播放列表/正在播放、歌词、设置、托盘已落地；发布材料已就绪）

## 文档

| 文档 | 内容 |
|---|---|
| [docs/Idea.md](docs/Idea.md) | 产品初衷与六条诉求 |
| [docs/FeatureList.md](docs/FeatureList.md) | 168 项功能清单（P0/P1/P2） |
| [docs/Prd.md](docs/Prd.md) | 产品需求：画像、场景、页面、FR、NFR |
| [docs/Arch.md](docs/Arch.md) | 技术架构：crates、音频、库、IPC、插件、打包 |
| [docs/Mvp.md](docs/Mvp.md) | MVP 范围、56 个任务、验收与门禁 |
| [docs/reports/](docs/reports/) | M0 验证报告 |

## 当前已验证

| 能力 | 结果 |
|---|---|
| 解码 | 7/7：MP3、FLAC、WAV、AAC/M4A、ALAC、OGG、AIFF（symphonia） |
| 输出 | cpal 共享输出、设备枚举与切换、设备丢失回退 |
| IPC | 命令错误封套、`player:state` 4Hz 事件、1000 事件无丢失 |
| 日志 | tracing 按天落盘到系统日志目录，panic 崩溃文件 |
| 媒体库 | SQLite + FTS5 v2、增量扫描/暂停/取消/续扫、拼音搜索、虚拟滚动与列排序 |
| 播放 | 7 格式流式解码、cpal 共享输出、队列/上一首/下一首/停止/4 模式/全局播放栏、坏文件跳过与设备恢复 |
| 播放列表 | 创建/重命名/删除、加入/移除、上移/下移、整列表播放 |
| 浏览 | 正在播放页、专辑/艺术家聚合与详情、虚拟滚动曲目表、内嵌封面提取与缓存 |
| 设置 | 常规/播放/音频/歌词/界面/关于、settings.json 持久化、主题跟随系统、设备切换、侧栏调宽/折叠、窗口置顶、无边框窗口、原生窗口透明度（Linux 降级为界面不透明度）；设置页三语化 |
| 系统集成 | 托盘菜单、单实例文件转交、文件拖放、关闭到托盘、启动最小化 |
| 歌词 | 同名 .lrc 优先 + 内嵌歌词回退、GBK/UTF-8 解码、同步滚动、点击跳转、桌面置顶歌词窗口与字号/颜色设置 |
| 前端 | React 19 + 路由 + 三语 i18n + 状态页，曲库列表、搜索与播放列表页 |
| 打包 | Windows NSIS + portable zip、macOS x64/arm64 dmg、Linux AppImage/deb/rpm；CI 四作业全绿 |
| 路径/配置 | 平台数据目录、便携模式、settings.json 原子写、损坏隔离 |
| 诊断 | panic 崩溃日志、settings+最近日志诊断 zip |

未验证/待补证据：Windows 11 与 macOS arm64 真机启动与出声、Ubuntu 24.04/Wayland、8 小时长稳与 10 万行 fps/CPU 基准、真实标签样本（ID3v1/APEv2/Vorbis/MP4 atoms/WAV-ID3）、设备拔插真机恢复。

## 环境要求

- Node.js 22、pnpm 10
- Rust **1.99.1**（仓库 `rust-toolchain.toml` 已固定；specta 2.0.0-rc 需要该版本）
- Tauri 2 系统依赖：macOS Xcode CLT；Windows WebView2 + MSVC；Ubuntu 22.04+ `libwebkit2gtk-4.1-dev` 等（见 `.github/workflows/ci.yml`）

## 开发

```bash
pnpm install
pnpm tauri dev
```

仅前端预览：

```bash
pnpm dev
```

> 浏览器里没有 Tauri IPC，验证台会显示“浏览器预览”。

## 测试与检查

```bash
pnpm build

cd src-tauri
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm test:e2e        # 浏览器 E2E（mock Tauri IPC）
```

音频夹具与探针：

```bash
cd src-tauri
cargo run -p biu-audio --example probe -- ../fixtures/audio/m0/*
cargo run -p biu-audio --example output_probe -- 44100
cargo run -p biu-library --example scan_bench -- 10000
```

## 目录

```text
src/                     React 前端（视图 + 播放栏 + 设置）
src/bindings.ts          tauri-specta 生成的类型与命令
src-tauri/
  crates/
    biu-core/            路径校验、基础类型
    biu-audio/           symphonia 解码 + cpal 输出
    biu-library/         SQLite/FTS5 扫描与查询
    biu-ipc/             IPC DTO
  src/                   Tauri 入口、命令、日志、事件
  tests/                 IPC 集成测试
fixtures/audio/m0/       7 格式音频夹具
docs/                    产品/架构/MVP/报告
```

## 命令行与右键

- 曲目表右键：选择目标播放列表即时加入，或新建列表后加入。
- 命令行：`BiuPlayer --playlist <playlist-id> <file-or-dir>...`，把文件/目录加入指定播放列表；已运行实例会转发参数到现有窗口。
- 播放列表 ID 可在详情页 URL（`#/playlists/<id>`）中查看。

## 发布产物

最近一次绿色矩阵：<https://github.com/tagecode/BiuPlayer/actions/runs/45320213684>

- Windows：`BiuPlayer_0.1.0_windows_x64-setup.exe`、`BiuPlayer_0.1.0_windows_x64-portable.zip`
- macOS：`BiuPlayer_0.1.0_macos_x64.dmg`、`BiuPlayer_0.1.0_macos_arm64.dmg`
- Linux：`BiuPlayer_0.1.0_linux_x64.AppImage`、`.deb`、`.rpm`

> 产物未签名/未公证；macOS 需右键打开或清除 quarantine；Windows 需要 WebView2。

## 发布

- 安装/卸载：[docs/INSTALL.md](docs/INSTALL.md)
- 发布流程：[docs/RELEASE.md](docs/RELEASE.md)
- 变更记录：[CHANGELOG.md](CHANGELOG.md)

## 下一步

1. 打 `v0.1.0` tag 实跑发布矩阵，在干净机验证安装/升级/卸载与绿色版。
2. 补 UI-15 证据：10 万行滚动 fps、播放 CPU ≤3%、8h 长稳（`powershell -ExecutionPolicy Bypass -File scripts\soak.ps1 -Hours 8`）。
3. QA 收尾：诊断包路径脱敏、覆盖率门槛接入 CI、专辑艺术家视图与各标签格式夹具。

## 许可

代码采用 MIT；依赖与音频格式的许可清单在发布前复核。
