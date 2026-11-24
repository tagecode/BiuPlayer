# M0-05 打包冒烟报告

- 日期：2026-11-24
- 环境：macOS 26.5.1（x86_64）、Tauri 2.12.7、Rust 1.99.1
- 状态：**部分完成**——macOS x64 `.dmg` 已产出；Windows、Linux、Apple Silicon 未构建。

## 1. 已产出

| 产物 | 路径 | 体积 | 结果 |
|---|---|---|---|
| `BiuPlayer_0.1.0_x64.dmg` | `src-tauri/target/release/bundle/dmg/` | 3.76 MiB | 成功 |
| `BiuPlayer.app` | `src-tauri/target/release/bundle/macos/` | — | 成功（dmg 构建后已清理） |

命令：

```bash
CI=true pnpm tauri build --bundles dmg
```

说明：首次不带 `CI=true` 时，Tauri 的 `bundle_dmg.sh` 调用 Finder 设置窗口布局，本机 AppleEvent 超时（`-1712`）；加 `CI=true` 后跳过该步骤，dmg 正常生成。CI 环境应始终设置 `CI=true`。

## 2. 构建验证

| 项 | 结果 |
|---|---|
| `pnpm build` | 通过（JS 230 KB / gzip 72 KB） |
| `cargo test --workspace` | 通过 |
| `cargo clippy --workspace --all-targets -D warnings` | 通过 |
| `cargo fmt --all --check` | 通过 |
| `pnpm tauri build --no-bundle` | 通过 |
| `pnpm tauri build --bundles dmg` | 通过（3.76 MiB） |

## 3. 未完成

| 平台 | 产物 | 状态 |
|---|---|---|
| Windows x64 | `.zip` 绿色版 / NSIS | 未构建 |
| macOS arm64 | `.dmg` | 未构建（需 Apple Silicon 或交叉 target） |
| Linux x64 | AppImage / deb / rpm | 未构建 |

## 4. 下一步

1. 在 macOS 上挂载 dmg，拖入 Applications，首次启动确认 Gatekeeper 提示与“仍要打开”流程。
2. CI 加 release 矩阵：Windows NSIS+zip、macOS x64/arm64 dmg、Linux AppImage/deb/rpm；设置 `CI=true`。
3. 记录各产物 SHA256 与体积，确认 ≤25 MB 预算。
4. 在 Windows 10/11、Ubuntu 22.04 各做一次安装/启动/播放冒烟。

## 5. 风险

- 未签名：macOS Gatekeeper 会拦截，Windows SmartScreen 会提示；发布说明需写清绕过步骤。
- Windows 绿色版依赖 WebView2 Runtime，目标机缺失时需引导安装。
- Apple Silicon 需要 `aarch64-apple-darwin` target；未验证。
- Linux AppImage 在无 FUSE 环境需 `--appimage-extract-and-run`。
