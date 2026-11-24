# D1 数据层

日期：2026-11-24
任务：D1-01..04（5 人日）
状态：**完成**（MVP schema、备份、损坏检测、迁移测试）

## 1. Schema（D1-01）

`src-tauri/crates/biu-library/src/lib.rs`，`SCHEMA_VERSION = 1`：

- `schema_migrations(version, applied_at)`
- `folders(id, path, added_at, last_scan_at, scan_state)`
- `artists(id, name, sort_name)`
- `albums(id, title, artist_id, year, cover_path)`
- `tracks(...)` 含 `folder_id`、`artist_id`、`album_id`、时长/采样率/声道/码率、`mtime`、`missing`
- `playlists`、`playlist_items`
- `lyrics`
- `play_history`
- `tracks_fts`（FTS5 外部内容，`unicode61`，含 INSERT/DELETE/UPDATE 触发器）
- 索引：`idx_tracks_folder/album/artist/mtime`

## 2. 打开、PRAGMA 与备份（D1-02）

`Database::open` 顺序：

1. 创建父目录。
2. `prepare_existing`：
   - 空文件/不存在：跳过。
   - `PRAGMA integrity_check` 非 `ok` → `LibraryError::Corrupt`。
   - 无 `schema_migrations` 或版本 `< 1` → 复制为 `library.db.bak`。
   - 版本 `> 1` → `SchemaTooNew { found, supported }`，拒绝降级。
3. 打开连接并设置 `WAL`、`synchronous=NORMAL`、`foreign_keys=ON`、`busy_timeout=5000`、`temp_store=MEMORY`。
4. 事务内建表、FTS5、触发器，写入 `schema_migrations(1)`。

`backup_path(path)` 返回 `library.db.bak`，供诊断/恢复使用。

## 3. 配置（D1-03）

已在 E1-02 实现（`src-tauri/crates/biu-core/src/settings.rs`）：

- 临时文件 + `fsync` + `rename` 原子写。
- 损坏 JSON 隔离为 `settings.corrupt.<unix>.json`，启动用默认值。
- 未知字段保留（`#[serde(flatten)] extra`）。
- 高 `schemaVersion` 只读，不覆盖。

## 4. 测试（D1-04）

```bash
cd src-tauri
cargo test -p biu-library
cargo clippy --workspace --all-targets -- -D warnings
```

`crates/biu-library/tests/migrations.rs`：

- `fresh_database_is_version_one_and_idempotent`：新建为版本 1，重开不重复迁移、不产生备份。
- `legacy_database_is_backed_up_before_migration`：无迁移表的老库先备份，备份保留旧表。
- `newer_schema_is_rejected_not_downgraded`：版本 99 返回 `SchemaTooNew`，不修改原库。
- `corrupt_file_fails_without_panic`：非 SQLite 文件返回错误，不 panic。

结果：4 passed；`scan.rs` 1 passed；clippy 通过。

## 5. 未完成/风险

- 未做 10 万曲目容量实测；M0 的 1 万空文件数据仍有效。
- 备份是整库复制，迁移前会短暂占用与 DB 等大的磁盘；大库后续可换 `VACUUM INTO` 或页级备份。
- WAL 文件未一起备份；备份时要求无其他连接（当前仅启动期调用）。
- schema 仍是 MVP 最小集，P1 的 `cue_sheets`、`track_stats`、`plugins`、`skins` 等表未加入。
