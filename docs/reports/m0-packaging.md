# M0-05 打包矩阵报告

日期：2026-11-24
任务：M0-05 三平台打包（3 人日）
状态：**通过（四类产物全部构建成功）**
工作流：`.github/workflows/release.yml`，手动 `workflow_dispatch` + `v*` tag。
最终 run：<https://github.com/tagecode/BiuPlayer/actions/runs/45320213684>（commit `ceb1eb6`）

## 1. 矩阵

| 作业 | runner | target | 产物 |
| --- | --- | --- | --- |
| windows-x64 | windows-latest | x86_64-pc-windows-msvc | NSIS 安装版 + portable zip |
| linux-x64 | ubuntu-22.04 | x86_64-unknown-linux-gnu | AppImage + deb + rpm |
| macos-x64 | macos-latest | x86_64-apple-darwin | dmg |
| macos-arm64 | macos-latest | aarch64-apple-darwin | dmg |

## 2. 产物（最终 run）

| 文件 | 大小 |
| --- | --- |
| `BiuPlayer_0.1.0_x64-setup.exe` | 2,629,873 B（2.51 MiB） |
| `BiuPlayer_0.1.0_x64-portable.zip` | 3,838,033 B（3.66 MiB） |
| `BiuPlayer_0.1.0_amd64.AppImage` | 83,462,648 B（79.59 MiB） |
| `BiuPlayer_0.1.0_amd64.deb` | 5,324,344 B（5.08 MiB） |
| `BiuPlayer-0.1.0-1.x86_64.rpm` | 5,329,657 B（5.08 MiB） |
| `BiuPlayer_0.1.0_x64.dmg` | 4,094,165 B（3.90 MiB） |
| `BiuPlayer_0.1.0_aarch64.dmg` | 4,017,969 B（3.83 MiB） |

artifact 汇总：Windows 6,427,216 B；Linux 93,332,182 B；macOS x64 4,056,985 B；macOS arm64 3,984,206 B。

## 3. 遇到的问题与修复

1. **macOS arm64 标签不可用**
   - `macos-14` 直接取消：`The configuration 'macos-14-arm64-us-default' is not supported`。
   - `macos-13` 作为 arm64 交叉编译机时长期排队（15 分钟未启动）。
   - 最终改用 `macos-latest` 原生/交叉构建，x64 与 arm64 均成功。
2. **Linux artifact 过大**
   - 原路径 `bundle/**/*` 把解包后的 `BiuPlayer.AppDir` 一起上传，artifact 206 MB、405 个文件。
   - 改为只收最终文件：`appimage/*.AppImage`、`deb/*.deb`、`rpm/*.rpm`、`dmg/*.dmg`、`nsis/*.exe`、portable zip；artifact 降至 93 MB。
3. **Windows 打包编译失败**
   - `fallback_roots` 在 Windows 分支未使用，`-D warnings` 下报 `function fallback_roots is never used`。
   - 修复 `src-tauri/crates/biu-core/src/paths.rs`：Windows 也通过 `fallback_roots` 提供无环境变量时的回退，提交 `84756c0`。

## 4. 验收与限制

已验收：

- 四类产物全部由 CI 构建成功，Windows/Linux/macOS x64/macOS arm64 四个作业均 success。
- macOS x64 dmg 已本机挂载并启动 `BiuPlayer.app`，日志目录正常解析。
- Windows NSIS 与 portable zip 均生成；portable zip 内含 `biuplayer.exe`，存在 `WebView2Loader.dll` 时会一并打包。
- Linux AppImage/deb/rpm 均生成；AppImage 可在 Ubuntu 22.04 运行（CI 构建环境）。

已知限制：

- **未签名/未公证**：Windows SmartScreen、macOS Gatekeeper 会拦截；macOS 需右键打开或 `xattr -dr com.apple.quarantine`。符合 IDEA 的零预算约束。
- **WebView2 依赖**：NSIS 当前使用 Tauri 默认的 `downloadBootstrapper`，无网络或未安装 WebView2 的机器需要先安装运行时；portable 版直接依赖系统 WebView2。后续评估 `offlineInstaller`（包体增大）或文档说明。
- **Linux 运行时**：AppImage 需要 FUSE2 或 `--appimage-extract-and-run`；deb/rpm 依赖系统 WebKitGTK 4.1。
- **macOS arm64** 为 `macos-latest` 构建，未单独在 Apple Silicon 真机启动验证；M0-06/后续补。
- 未做代码签名、公证、自动更新；更新只提示手动下载。
