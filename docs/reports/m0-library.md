# M0-04 媒体库与扫描报告

- 日期：2026-11-24
- 环境：macOS 26.5.1（x86_64）、rusqlite 0.32.1（bundled SQLite）、walkdir 2
- 结论：**建库、迁移、扫描入库、FTS5 搜索可用；1 万文件扫描 2.4 s，满足 M0 冒烟目标。**

## 1. Schema

`Database::open` 执行 PRAGMA：`journal_mode=WAL`、`synchronous=NORMAL`、`foreign_keys=ON`、`busy_timeout=5000`。

表：

| 表 | 用途 |
|---|---|
| `schema_migrations` | 迁移版本 |
| `folders` | 音乐文件夹与扫描状态 |
| `artists` / `albums` | 艺术家/专辑聚合 |
| `tracks` | 曲目主表（路径、标题、时长、格式、mtime、size、quick_hash） |
| `playlists` / `playlist_items` | 播放列表 |
| `lyrics` | 歌词缓存/关联 |
| `play_history` | 播放历史 |
| `tracks_fts` | FTS5 外部内容表（title/artist/album），带 INSERT/UPDATE/DELETE 触发器 |

M0 只建最小字段；完整 DDL（cue_sheets、track_stats、plugins、winamp_dsp 等）在 E1/D1 按 Arch §12 迁移补齐。

## 2. 功能验证

`src-tauri/crates/biu-library/tests/scan.rs`：

- 临时目录写入 3 个文件（2 个 `.mp3`、1 个 `.txt`）。
- `add_folder` + `scan_folder` 后 `tracks` 为 2，`.txt` 被忽略。
- 二次扫描 `inserted=0`，不重复入库。
- `search("tone")` 通过 FTS5 命中。

`examples/scan_bench.rs`（1 万空 `.mp3`）：

```text
generated 10000 files in 518 ms
scan: seen=10000 inserted=10000 skipped=0 elapsed=2385ms tracks=10000
fts: hits=1 elapsed=1ms
```

## 3. 结论与偏差

- 扫描 1 万文件 2.4 s，远低于 MVP “1 万 ≤60 s”；但夹具是空文件、无标签、无封面，真实曲库会更慢。
- 当前 `inserted` 统计把“新插入”与“更新”合并；正式实现需区分 `inserted/updated/skipped`。
- 未接 lofty 标签读取，`title` 暂用文件名；TAG-01 在 E1 实现。
- 未做增量扫描（mtime/size 已入库，但未跳过未变文件）、未做 rayon 并行、未做取消；属 LIB-02/LIB-03。
- 10 万曲库未测；目标 ≤500MB 内存、扫描预算另测。

## 4. 复现

```bash
cd src-tauri
cargo test -p biu-library -- --nocapture
cargo run -p biu-library --example scan_bench -- 10000
```
