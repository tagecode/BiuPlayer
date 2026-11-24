# M0-06 Linux 兼容性报告

日期：2026-11-24
任务：M0-06 Linux 兼容性验证（2 人日）
状态：**部分通过**（Ubuntu 22.04 构建 + 运行冒烟通过；20.04/24.04 与真实桌面未测）

## 1. 环境与依赖

- runner：`ubuntu-22.04`（GitHub Actions）
- WebKit：`libwebkit2gtk-4.1-dev`（Tauri 2 必需）
- 其他：`libappindicator3-dev`、`librsvg2-dev`、`patchelf`、`libasound2-dev`、`libfuse2`、`rpm`、`xvfb`
- Rust：1.99.1；Node 22；pnpm 10

## 2. 构建

`release.yml` 的 `linux-x64` 作业在 Ubuntu 22.04 上执行：

```bash
pnpm tauri build --target x86_64-unknown-linux-gnu --bundles appimage,deb,rpm
```

结果：

- AppImage：83,462,648 B
- deb：5,324,344 B
- rpm：5,329,657 B
- 作业 success，run 45320213684。

## 3. 运行冒烟

`ci.yml` 新增 `Linux runtime smoke`：

```bash
timeout --signal=TERM --kill-after=5 45s \
  xvfb-run -a src-tauri/target/release/biuplayer
# 期望 timeout 返回 124
grep -q "logging initialized" ~/.local/share/biuplayer/logs/biu.log.*
```

环境变量：`LIBGL_ALWAYS_SOFTWARE=1`、`WEBKIT_DISABLE_COMPOSITING_MODE=1`、`WEBKIT_DISABLE_DMABUF_RENDERER=1`、`GDK_BACKEND=x11`。

结果：run 45321447571 的 `check (ubuntu-22.04)` success；应用在 Xvfb 下保持运行至超时，日志目录与启动日志生成。

## 4. 结论与限制

已验证：

- Ubuntu 22.04 + WebKitGTK 4.1 可编译、可启动。
- 三件 Linux 产物均可构建。
- 启动路径使用 XDG 目录（`~/.local/share/biuplayer`、`~/.cache/biuplayer`）。

未验证/风险：

- **Ubuntu 20.04 不可用**：只有 WebKitGTK 4.0，Tauri 2 需要 4.1；Arch/PRD 已将 22.04 设为基线，20.04 仅探测。
- **Ubuntu 24.04** 未单独跑；其 WebKitGTK 4.1 版本较新，预计可用但未验证。
- **真实桌面/Wayland** 未测；Xvfb 是 X11 软渲染，不代表 GPU/合成器路径。
- **音频设备** 未测：CI 无声卡，`cpal` 设备枚举与播放需真机补测。
- AppImage 未在 CI 内以 `--appimage-extract-and-run` 启动；当前冒烟跑的是 release 二进制。
- 未测托盘、全局快捷键、文件关联（M2 范围）。
