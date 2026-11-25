# 媒体库缺口收尾（F-LIB-01/06）

日期：2026-11-25

## 实现

### 目录排除与隐藏文件（F-LIB-01）

- 扫描使用 `WalkDir::filter_entry`，深度 0 之外：
  - 任何以 `.` 开头的文件/目录直接跳过。
  - 命中 `library.excludeGlobs` 的路径跳过，且不向下遍历。
- 模式匹配规则：
  - 绝对路径按前缀匹配；
  - 目录名大小写不敏感匹配；
  - `**/name/**`、`*name*` 等包装会先去掉通配符再匹配。
- 默认排除 `**/@eaDir/**`（Synology 缩略图目录）。
- 设置页新增“媒体库”卡片，多行文本框编辑排除目录；失焦保存到 `settings.json`，下次扫描生效。

### 移除媒体库目录（F-LIB-01）

- `Database::remove_folder(id)` 删除 `folders` 行，依靠外键级联清理 `tracks`、`playlist_items` 等；音频文件本身不删除。
- 新命令 `library_remove_folder`；文件夹详情页新增“从媒体库移除”，确认后回到文件夹列表。

### 搜索结果高亮（F-LIB-06）

- 新增 `HighlightText` 组件，大小写不敏感匹配搜索词并用 `<mark class="search-hit">` 标出。
- 曲目表的标题/艺术家/专辑列，以及媒体库虚拟列表都接入；未搜索时不渲染标记。

## 验证

- 新增 `scan_skips_hidden_and_excluded_folders_and_removes_folder`：`.hidden/`、`skip/` 被跳过，`keep/` 与根目录文件入库；移除目录后曲目数归零。
- `cargo test --workspace`：58 passed / 0 failed。
- clippy `-D warnings`、`cargo fmt --check`、`pnpm build` 通过。

## 未完成

- `kill -9` 后 WAL 恢复实测（F-LIB-11）仍待补。
- 目录级“清理失效曲目”批量动作未做；当前通过删除目录手动清理。
- 排除规则未做 glob 完整语法（仅目录名/绝对前缀）。
