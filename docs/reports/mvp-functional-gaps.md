# MVP 功能缺口收尾（搜索 / 崩溃提示 / 手动排序）

日期：2026-11-25

## 全局搜索页（P-08）

- `/search` 由占位页替换为真实搜索页，顶栏搜索框启用：回车跳转 `/search?q=...`。
- 页面 200ms 防抖调用 `library_search`，最多 100 条；使用 `TrackTable` 展示并高亮命中词。
- 双击/播放按钮把结果作为队列从当前行开始播放；空查询显示提示，无结果显示空态。
- 报告搜索命中数，支持标题/艺术家/专辑/拼音全拼/首字母（复用 FTS v2）。

## 崩溃后启动提示（F-QA-02）

- `diagnostics::latest_crash(log_dir)` 扫描 `crash-*.log`，返回最近文件的路径与时间；新命令 `crash_report`。
- Shell 启动时查询；若有未忽略的崩溃记录，显示顶部提示条，可“导出诊断”或“忽略”。
- 忽略按文件路径记入 `localStorage["biu.crash.dismissed"]`，新崩溃会再次提示；诊断包沿用 E1-03，不含曲库/音频。

## 曲目手动排序（F-PL-05）

- schema v4：`tracks.manual_order REAL` + 索引；`SCHEMA_VERSION=4`。
- `Database::move_track(track_id, before_id, after_id)` 用相邻项中点写入排序值；间隙过小时按全表 `ROW_NUMBER` 重排。
- `list_tracks_sorted` 支持 `manual`，按 `COALESCE(manual_order, id)` 排序。
- 媒体库表头新增“手动”列；手动模式下每行显示上移/下移按钮，移动后重新拉取当前窗口数据。
- 未做拖动排序；跨页移动需先滚动到相邻行，当前实现限制在已加载窗口内。

## 验证

- `cargo test --workspace`：59 passed / 0 failed。
- 新增 `manual_order_moves_and_persists`：a,b,c → c,a,b，重开数据库仍保持。
- clippy `-D warnings`、`cargo fmt --all --check`、`pnpm build` 通过。
- `pnpm tauri dev` 启动 25s 无 panic。

## 未完成

- 搜索页未按艺术家/专辑分组，仅为曲目列表；可后续补分组。
- 崩溃提示条未做截图/视觉回归，`crash_report` 正常路径未做自动化测试。
- 手动排序无拖拽、无跨页插入、无“恢复默认顺序”按钮。
