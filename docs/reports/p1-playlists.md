# P1-01..04 播放列表

日期：2026-11-24

## 范围

- P1-01 创建、重命名、删除播放列表。
- P1-02 从媒体库添加曲目、移除曲目。
- P1-03 播放列表内上移/下移排序。
- P1-04 播放列表页与详情页；列表可直接作为播放队列播放。

数据表沿用 D1 的 `playlists` / `playlist_items`，未新增迁移。

## 数据层（`crates/biu-library`）

新增：

- `PlaylistSummary { id, name, track_count }`
- `PlaylistItem { position, track: TrackListItem }`
- `Database::list_playlists`
- `Database::create_playlist`
- `Database::rename_playlist`
- `Database::delete_playlist`
- `Database::playlist_items`
- `Database::add_tracks_to_playlist`
- `Database::remove_playlist_item`
- `Database::move_playlist_item`

实现细节：

- 创建/重命名会 trim 名称并拒绝空名；删除依赖 `ON DELETE CASCADE` 清空条目。
- 添加曲目忽略重复项，追加到 `MAX(position)+1`；不存在的 track id 被跳过。
- 移除与排序都采用“读取有序 track_id → 重建 position”的方式，避免 SQLite 主键 `(playlist_id, position)` 在逐行更新时冲突。
- 每个写操作使用事务并更新 `playlists.updated_at`。

测试 `crates/biu-library/tests/playlists.rs`：创建、空名拒绝、重复添加、排序、移除后重排、重命名、删除级联，1 个测试通过。

## 命令层（`src-tauri/src/commands/library.rs`）

新增命令：

- `playlist_list`
- `playlist_create(name)`
- `playlist_rename(id, name)`
- `playlist_delete(id)`
- `playlist_items(id)`
- `playlist_add_tracks(id, track_ids)`
- `playlist_remove_item(id, position)`
- `playlist_move_item(id, from, to)`

DTO：`PlaylistDto { id, name, track_count }`、`PlaylistItemDto { position, track }`，已注册到 specta 并生成 bindings。

## 前端

- `/playlists`：创建、重命名（prompt）、删除（confirm）、空状态、进入详情。
- `/playlists/:id`：曲目表、播放整张列表、双击播放、上移/下移、移除、空状态引导去媒体库。
- `/library`：新增“目标播放列表”选择器；每行“加入”按钮调用 `playlistAddTracks`，重复添加给出提示。
- 样式：`.playlist-list`、`.playlist-row`、`.empty-card`、`.breadcrumb`、`.library-playlist` 等加入 `src/shell.css`。

## 验证

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm build
```

- Rust 测试 43 个通过（新增播放列表 1 个）。
- 前端构建通过：JS 358.05 kB / gzip 113.10 kB，CSS 13.93 kB。
- `pnpm tauri dev` 启动正常，bindings 已包含 8 个 playlist 命令。

## 已知限制

- 重命名使用 `window.prompt`，未做内联编辑。
- 添加曲目为逐行操作，未做多选/拖拽到播放列表。
- 播放列表排序为上移/下移按钮，未做拖拽。
- 播放列表不显示封面与总时长。
- 没有删除确认的撤销；删除列表不可恢复。
- 未做 E2E 点击验证。
