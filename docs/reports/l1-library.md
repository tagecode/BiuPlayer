# L1 媒体库（L1-01/02 首版）

日期：2026-11-24

## 已实现

- `crates/biu-library` 扫描时用 lofty 0.22 读取标签：标题、艺术家、专辑、音轨号、碟号、流派、年份、时长、采样率、声道、码率。
- 艺术家/专辑按名称去重写入 `artists`/`albums`，扫描内使用 HashMap 缓存，避免 1 万文件逐条查库。
- 标签缺失时回退文件名；读取失败不阻塞扫描。
- `Database::list_tracks(limit, offset)` 左连接艺术家/专辑，按标题排序，供媒体库列表使用。
- Tauri 命令：
  - `library_add_folder(path)`：校验目录、入库、同步扫描，返回 `files_seen/inserted/skipped/elapsed_ms/total_tracks`。
  - `library_tracks(limit, offset)`：列表。
  - `library_track_count()`：总数。
  - `library_search(query, limit)`：复用 FTS5。
- 应用 `setup` 打开 `paths.database_file`，失败只记日志不阻塞启动。
- 前端 `/library`：绝对路径输入 + 添加并扫描、曲目表（标题/艺术家/专辑/时长/格式）、行内播放按钮调用 `player_open`。

## 验证

- `scan_reads_tags_and_lists_tracks`：复制 `fixtures/audio/m0/tone.mp3`，用 lofty 写入中文标题/艺术家/专辑/音轨号/年份，扫描后断言字段与时长。
- `library_add_folder_scans_fixtures`：mock app + 内存库扫描 `fixtures/audio/m0`，7 个文件全部入库，列表含 FLAC。
- 10,000 个空 `.mp3`：生成 575ms、扫描 3014ms、FTS 1ms（`scan_bench`）。
- `cargo test --workspace`、clippy、`pnpm build` 通过。

## 未完成

- 目录选择器（当前手填绝对路径；P1-02 做文件/文件夹/拖放入口）。
- 增量扫描已按 mtime+size 跳过，但暂停/取消/断点、并发扫描未做（L1-03）。
- 搜索索引仅 `title/path`，尚未加入艺术家/专辑与拼音（L1-04）。
- 专辑/艺术家/流派/年代视图、封面、分页虚拟滚动未做（L1-05/06）。
