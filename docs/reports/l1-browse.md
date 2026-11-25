# 媒体库浏览视图：流派、年代、文件夹

## 范围

对应 F-LIB-04：歌曲/专辑/艺术家/专辑艺术家/流派/年代/文件夹视图。

本轮补齐流派、年代、文件夹；专辑与艺术家此前已有。专辑艺术家通过 `albums.artist_id` 归并到艺术家视图：艺术家详情页现在同时展示该艺术家名下的专辑。

## 后端

`crates/biu-library` 新增：

- `list_genres() -> Vec<(String, i64)>`：去空白、按名称 NOCASE 排序。
- `list_years() -> Vec<(i64, i64)>`：过滤 1–9999，按年代倒序。
- `list_folders() -> Vec<(i64, String, i64)>`：含未入库目录（count=0）。
- `browse_tracks(filter, value)`：白名单 `genre` / `year` / `folder`，按标题排序；非法 filter 返回 `InvalidArgument`。

命令：`library_genres`、`library_genre_tracks`、`library_years`、`library_year_tracks`、`library_folders`、`library_folder_tracks`。

## 前端

- `src/components/LibraryNav.tsx`：媒体库子导航（歌曲/专辑/艺术家/流派/年代/文件夹）。
- `src/pages/Browse.tsx`：Genres/GenreDetail/Years/YearDetail/Folders/FolderDetail。
- 歌曲页、专辑页、艺术家页均接入子导航。
- 艺术家详情页新增“专辑”网格，列出该艺术家作为专辑艺术家的专辑。
- 路由：`/library/genres/:name`、`/library/years/:year`、`/library/folders/:id`。
- 样式：`.library-nav`、`.library-tab`、`.tile-grid`、`.tile`、`.folder-row`。

## 验证

- `tests/views.rs::genres_years_and_folders_group_and_filter`：3 首跨 2 流派/2 年代/2 目录，分组计数与过滤正确；非法 filter 报错。
- `pnpm build`：JS 434.16 kB / gzip 131.33 kB。

## 限制

- 流派、年代、文件夹都一次性返回全部条目；10 万曲目下条目数受元数据基数限制，暂未分页。
- 专辑艺术家未做独立列表页，合并进艺术家视图。
- 文件夹视图显示入库根目录，不展示子目录层级。
