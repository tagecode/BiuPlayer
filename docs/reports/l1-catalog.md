# L1-05/06 专辑与艺术家视图

日期：2026-11-24

## 范围

- 专辑列表与专辑详情。
- 艺术家列表与艺术家详情。
- 点击曲目/播放专辑/播放全部时，把当前列表作为播放队列。

## 数据层

`crates/biu-library` 新增：

- `AlbumSummary { id, title, artist, year, track_count }`
- `ArtistSummary { id, name, track_count }`
- `Database::list_albums`
- `Database::album_tracks`
- `Database::list_artists`
- `Database::artist_tracks`

补充：扫描时把标签年份写入 `albums.year`（原先只写 title/artist_id），首次插入后仅用 `COALESCE(year, ?)` 补齐，不覆盖已有值。

测试 `tests/catalog.rs`：3 首带标签文件聚合为 2 张专辑、2 位艺术家，专辑曲目 2 首、艺术家曲目 2 首，年份 2024，1 个测试通过。

## 命令层

- `library_albums`
- `library_album_tracks(id)`
- `library_artists`
- `library_artist_tracks(id)`

DTO：`AlbumDto { id, title, artist, year, track_count }`、`ArtistDto { id, name, track_count }`。

## 前端

- `/library/albums`：专辑网格，封面占位、标题、艺术家、年份、曲目数。
- `/library/albums/:id`：专辑头部、播放专辑、曲目表。
- `/library/artists`：艺术家列表。
- `/library/artists/:id`：艺术家头部、播放全部、曲目表。
- 新增 `src/components/TrackTable.tsx` 复用曲目表。

## 验证

- Rust 测试 44 个通过（新增 catalog 1 个）。
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --check` 通过。
- `pnpm build` 通过。

## 已知限制

- 封面仍为占位，`albums.cover_path` 尚未写入。
- 专辑/艺术家列表未分页、无虚拟滚动。
- 艺术家详情只列曲目，未按专辑分组。
- 未做 E2E。
