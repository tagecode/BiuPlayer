# 拖放与第二实例文件参数

## 拖放

- `src/Shell.tsx` 监听 Tauri 拖放事件：
  - `tauri://drag-enter` / `drag-leave`：显示/隐藏全屏虚线提示。
  - `tauri://drag-drop`：按扩展名分流。
- 音频文件（mp3/flac/wav/m4a/aac/ogg/oga/aiff/aif）→ `playerPlayQueue`，直接开始播放。
- 目录 → `libraryScanStart`，进入现有扫描流程。
- 混合拖入时先播放音频，再扫描第一个目录。
- 样式 `.drop-overlay`，不拦截鼠标事件。

## 第二实例

- `src-tauri/src/lib.rs` 的单实例插件回调保留显示/聚焦主窗口，并把 `args[1..]` 通过 `single-instance:open` 事件发给前端。
- `Shell.tsx` 订阅该事件，复用同一套拖放分流逻辑；音频文件在原窗口播放，目录启动扫描。

## 验证

- `pnpm build` 通过：JS 379.87 kB / gzip 118.09 kB，CSS 19.05 kB。
- `cargo test --workspace`：51 个测试通过；clippy/fmt 通过。
- 未做真实文件管理器拖放与双开参数的人工验收；macOS `open -a` 传参未测。

## 限制

- 一次拖入多个目录只扫描第一个。
- 拖入音频不读取标签，队列标题使用文件名。
- 第二实例在首实例前端未就绪时可能丢失事件。
