# M0-06 Linux 兼容性报告

- 日期：2026-11-24
- 状态：**未执行**（没有 Linux 开发机/容器）
- 结论：M0 出口的 Linux 项未关闭；不能宣称 MVP 支持 Ubuntu。

## 1. 已知约束

| 项 | 结论 |
|---|---|
| Tauri 2 运行时 | 依赖 WebKitGTK **4.1**（`libwebkit2gtk-4.1-dev`） |
| Ubuntu 22.04 | 默认源提供 4.1，理论可用，未实测 |
| Ubuntu 24.04 | 提供 4.1，理论可用，未实测 |
| Ubuntu 20.04 | 仅 WebKitGTK 4.0，**不能直接运行 Tauri 2**；与 FeatureList/PRD 的 22.04 基线一致，20.04 仅做探测 |
| 音频 | 需要 ALSA/PulseAudio；CI 容器无声卡，只能验设备枚举与编译，不能验真实出声 |
| 打包 | AppImage 需要 `linuxdeploy`/FUSE；deb/rpm 需要 `libwebkit2gtk-4.1-dev` 与 `patchelf` |

## 2. 待执行清单

1. 在 Ubuntu 22.04 与 24.04 真机/VM 上安装依赖：
   ```bash
   sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev
   ```
2. `pnpm tauri dev` 启动，确认窗口、IPC、日志目录（`~/.local/share/com.tagecode.biuplayer/logs` 或 XDG 路径）。
3. `cargo test --workspace`；`cargo run -p biu-audio --example output_probe -- 44100`（有声卡时）。
4. `pnpm tauri build` 产出 AppImage/deb/rpm，记录体积与启动时间。
5. 在 Wayland 与 X11 各跑一次，记录缩放/窗口装饰差异。
6. 20.04 仅记录“WebKitGTK 4.0 不满足，安装 4.1 失败/成功”的探测结果，不承诺支持。

## 3. 现有 CI

`.github/workflows/ci.yml` 已包含 `ubuntu-22.04` job，安装 WebKitGTK 4.1 依赖并跑 `pnpm build`、`cargo fmt/clippy/test`、`pnpm tauri build --no-bundle`。该 job 尚未在 GitHub Actions 实际运行，不能替代真机报告。

## 4. 对 M0 出口的影响

M0 出口要求“三平台各出一个可运行产物”。当前只有 macOS 本机验证；Linux 与 Windows 均未产出，M0 **未达出口**。建议下一步：

- 本地用 Docker/UTM 起 Ubuntu 22.04 做一次编译+启动；
- 推送到 GitHub 触发 CI，拿到三平台日志；
- 再决定是否进入 E1，或先补 Linux/Windows 冒烟。
