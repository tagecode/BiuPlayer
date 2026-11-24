# L1-03 增量扫描、暂停/取消与断点续扫

日期：2026-11-24

## 已实现

- `biu-library::ScanControl`：`pause/resume/cancel`，后台扫描在文件间检查标志；`pause_after(n)` 供测试/自动化使用。
- `biu-library::ScanProgress`：`seen/inserted/skipped` 原子计数，供 UI 轮询。
- `Database::scan_folder_controlled(folder_id, root, control, progress)`：
  - 每个文件一个事务，暂停/取消时已处理文件已提交。
  - 每 50 个文件把 `{last_path, seen, inserted, skipped}` 写入 `folders.scan_state`。
  - 恢复时重新遍历目录，跳过直到（含）`last_path`，再继续扫描；`ON CONFLICT(path)` 保证不重复。
  - 未变化文件先比较 `size_bytes + mtime`，命中则跳过且**不读取标签**。
  - 完成时 `scan_state='complete'` 并更新 `last_scan_at`。
- Tauri 命令：
  - `library_scan_start(path)`：后台线程扫描，立即返回 `scanning`。
  - `library_scan_status()`：返回状态与实时计数。
  - `library_scan_pause()` / `library_scan_resume()` / `library_scan_cancel()`。
  - 上一次线程尚未退出时拒绝启动新扫描（避免两个写者）。
- `/library` 页面：添加并扫描后显示状态与计数，提供暂停/继续/取消；扫描中每 500ms 轮询，完成后自动刷新列表。

## 验证

- `scan_pauses_and_resumes_without_duplicates`：20 个文件，`pause_after(5)` 暂停时 5 首，续扫后 20 首、无重复。
- `scan_inserts_and_searches`：第二次扫描 `inserted=0, skipped=2`。
- `library_scan_start_reports_progress_until_complete`：mock app 后台扫描 7 个夹具，最终 `completed`、`files_seen=7`。
- 10,000 个空 MP3：首次 6.3s；二次扫描 345ms（全部跳过）；FTS 1ms。
- `cargo test --workspace`、clippy、`pnpm build` 通过。

## 未完成 / 已知限制

- 断点只记录最后路径，恢复时对目录做一次完整遍历并按路径跳过；10 万文件时遍历本身约数百毫秒。
- 若文件在检查点之前被修改，恢复会跳过它；下一次完整扫描会通过 mtime/size 修正。
- 暂停/取消是“当前文件完成后生效”，不是立即中断解码/标签读取。
- 目录选择器、并发扫描、缺失文件标记仍未做（L1-03 其余部分/L1-05）。
