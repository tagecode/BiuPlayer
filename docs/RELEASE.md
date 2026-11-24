# 发布流程

## 版本号

- 应用版本在 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 三处保持一致。
- 本文以 `0.1.0` 为例；发布新版本时同步替换。

## 发布前检查

```bash
pnpm install --frozen-lockfile
pnpm build
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- 更新 `CHANGELOG.md`，把 `[Unreleased]` 改为新版本并补日期。
- 更新 `docs/release-notes/vX.Y.Z.md`。
- 确认 `release.yml` 中矩阵与产物名仍匹配。
- 人工冒烟：安装/绿色版启动、扫描、播放、歌词、托盘。

## 触发

```bash
git tag v0.1.0
git push origin v0.1.0
```

或在 Actions 页面手动运行 `release`。手动运行只构建产物和校验文件，不创建 Release。

## 工作流

`release.yml`：

1. `build`：Windows NSIS + 绿色 zip、macOS x64/arm64 dmg、Linux AppImage/deb/rpm。
2. `checksums`：汇总所有产物，生成 `SHA256SUMS.txt`。
3. `release`：仅在 tag 上运行，创建 **draft** Release，附全部产物与校验文件。

## 发布草稿

- 打开 GitHub Releases，检查 draft 的产物、标题、自动生成的 release notes。
- 把 `docs/release-notes/vX.Y.Z.md` 的内容合并进说明。
- 确认 `SHA256SUMS.txt` 与产物一一对应。
- 手动 Publish。

## 未签名说明

本项目不签名、不公证：

- macOS Gatekeeper 会拦截，需要右键打开或 `xattr -dr com.apple.quarantine`。
- Windows SmartScreen 会提示，需要“更多信息 → 仍要运行”。
- 发布说明必须保留上述提示。

## 回滚

- 不删除已发布 tag；如需撤回，把 Release 改为 draft 或标记 prerelease。
- 修复后发新 patch 版本，不覆盖旧产物。
