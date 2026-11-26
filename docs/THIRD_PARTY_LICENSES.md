# 第三方组件与许可（MVP）

BiuPlayer 本体以 MIT OR Apache-2.0 发布。应用分发的二进制包含下列第三方组件；
本文件按运行时依赖列出主要项目与许可证，完整依赖树与版本以仓库中的
`package.json`、`pnpm-lock.yaml`、`src-tauri/Cargo.lock` 为准。

> 生成/复核方式：`pnpm licenses list --prod`（npm 侧）与
> `cargo metadata --format-version 1` + `cargo deny`（Rust 侧，尚未接入 CI）。

## 前端运行时

| 组件 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| react / react-dom | 19.x | MIT | UI 渲染 |
| react-router-dom | 7.x | MIT | 路由 |
| @tauri-apps/api | 2.x | MIT OR Apache-2.0 | IPC 绑定 |
| @tauri-apps/plugin-dialog | 2.x | MIT OR Apache-2.0 | 文件/目录选择框 |
| @tauri-apps/plugin-opener | 2.x | MIT OR Apache-2.0 | 打开外部链接 |

## Rust 运行时

| 组件 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| tauri | 2.x | MIT OR Apache-2.0 | 窗口、IPC、打包 |
| tauri-plugin-* | 2.x | MIT OR Apache-2.0 | opener / dialog / single-instance / window-state |
| symphonia | 0.5.4 | MPL-2.0 | 音频解码 |
| cpal | 0.15.3 | Apache-2.0 | 音频输出 |
| rusqlite（bundled SQLite） | 0.32 | MIT | 媒体库存储（SQLite 为 Public Domain） |
| lofty | 0.22 | MIT | 标签读写 |
| pinyin | 0.10 | MIT OR Apache-2.0 | 拼音索引 |
| blake3 | 1.x | CC0-1.0 OR Apache-2.0 | 封面内容哈希 |
| walkdir | 2.x | MIT OR Unlicense | 目录遍历 |
| encoding_rs | 0.8 | Apache-2.0 OR MIT | GBK/UTF-16 歌词解码 |
| serde / serde_json | 1.x | MIT OR Apache-2.0 | 序列化 |
| specta / tauri-specta / specta-typescript | rc | MIT | 类型导出 |
| tracing / tracing-subscriber / tracing-appender | 0.1–0.3 | MIT | 日志 |
| zip | 2.x | MIT | 诊断包 |
| thiserror | 2.x | MIT OR Apache-2.0 | 错误类型 |

## 需要注意的许可证

- **symphonia（MPL-2.0）**：文件级弱 copyleft。BiuPlayer 未修改其源码，
  仅以未修改的 crate 形式链接；若将来修改 symphonia 源文件，需要公开这些
  修改文件的源码。MPL-2.0 不要求公开 BiuPlayer 自身的源代码。
- **SQLite（Public Domain）**：通过 rusqlite 的 `bundled` feature 静态编译。
- **WebView2（Microsoft）**：Windows 上由系统/安装器提供，遵循 Microsoft 的
  WebView2 Runtime 许可，不随本仓库分发。

## 字体与素材

MVP 未内置第三方字体或图标字体；界面图标为内联 SVG/字符。
音频夹具（`fixtures/audio`）由项目自行生成或使用无版权测试音频，
不得替换为受版权保护的音乐。
