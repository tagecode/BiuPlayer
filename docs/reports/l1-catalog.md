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

## 封面提取（后续补充）

- 扫描时读取内嵌封面（≤8 MiB），用 BLAKE3 哈希命名写入 `<data_dir>/covers/<hash>.<ext>`，并更新 `albums.cover_path`；已有封面不重复写。
- 新增 `Database::album_cover_path` 与命令 `library_album_cover(id)`，返回 `data:image/...;base64,...`，前端按需加载。
- 前端新增 `src/components/AlbumCover.tsx`，专辑列表与专辑详情显示真实封面，失败回退首字母。
- 测试 `embedded_cover_is_cached_once`：1×1 PNG 写入 ID3，扫描后文件存在且二次扫描不重复，通过。
- 已知限制：仅取第一张图；不生成缩略图，直接返回原图 base64；大图会增加 IPC 负载，后续做多尺寸缓存。
