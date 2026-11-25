# 集成测试与 E2E 验收（macOS）

日期：2026-11-25
环境：macOS 26.5.1（25F80）、Chromium Headless Shell 159、Rust 1.99.1、Node 22.18、pnpm 10.11

## 结论

本轮在 macOS 上完成两类自动化验收：

| 层级 | 命令 | 结果 |
|---|---|---|
| Rust 全量测试（含集成） | `cargo test --workspace` | **61 passed / 0 failed** |
| 新增跨模块集成 | `cargo test --test mvp_workflow` | **2 passed** |
| 浏览器 E2E | `pnpm test:e2e` | **5 passed**，约 4.8s |
| 原生启动冒烟 | `pnpm tauri dev` | 启动成功，无 panic |

## 集成测试覆盖

新增 `src-tauri/tests/mvp_workflow.rs`：

1. `decodes_every_mvp_format`
   - 对 7 个 MVP 夹具逐一调用 `biu_audio::probe_path`；
   - 断言 44.1kHz、双声道、时长 > 1s。

2. `scan_search_playlist_and_manual_order_flow`
   - 扫描 `fixtures/audio/m0` 7 个文件，断言 `files_seen/inserted = 7`；
   - FTS 搜索 `tone` 命中 7 条；
   - 创建播放列表、加入 3 首、反向重排、复制、批量删除；
   - 手动排序把末项移到首位，重开数据库后顺序保持；
   - 移除目录后 `track_count = 0`，验证外键级联清理。

这覆盖了 `F-AUD-01`、`F-LIB-02`、`F-LIB-04`、`F-LIB-06`、`F-PL-01`、`F-PL-05`、`F-PL-08` 的核心链路。

## E2E 覆盖

`e2e/app.spec.ts`，通过 `e2e/mock-tauri.js` 在 Chromium 中模拟 Tauri IPC：

1. Shell 加载：导航、空闲播放栏。
2. 媒体库：3 条曲目、搜索“晴天”收敛到 1 条、播放后播放栏显示标题/艺术家。
3. 设置：点击“音频”“媒体库”，URL 变为 `/settings/audio`、`/settings/library`，对应卡片可见，导航 `.active` 高亮。
4. i18n：切到英文后侧栏变 `Library`，设置导航变 `Audio`。
5. 播放列表：种子列表“通勤”详情显示 2 首曲目。

覆盖 `F-SET-01`（分类导航/URL/高亮）、`F-I18N-01`（运行时切换）、`F-PL-06`（搜索）及播放栏基本联动。

## 原生启动冒烟

`pnpm tauri dev` 启动 Vite 1420 与 `target/debug/biuplayer`，日志显示：

```
BiuPlayer logging initialized
application paths resolved portable=false
window decorations updated frameless=false
```

进程保持运行，无 panic。测试时窗口以 `frameless=false` 启动。

## 限制（必须明确）

- **macOS 没有可用的 WKWebView WebDriver**，`tauri-driver` 官方仅支持 Windows/Linux，因此浏览器 E2E 使用 mock IPC，不是对原生 WebView 的点击测试。
- E2E 未覆盖：原生窗口拖拽/置顶/透明度、托盘菜单、系统文件拖放、真实音频输出、桌面歌词窗口、安装包。
- 原生冒烟只验证启动与日志，不做 UI 断言。
- 浏览器 mock 位于 `e2e/mock-tauri.js`，不进入生产包。

## 复现

```bash
# 集成 + 单元
cargo test --workspace

# 浏览器 E2E（首次先装 Chromium）
pnpm exec playwright install chromium
pnpm test:e2e
```

CI 新增 `e2e (browser mock)` 作业，在 ubuntu-22.04 上安装 Chromium 并执行同一套用例。
