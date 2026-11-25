# F-PL-02：右键菜单与命令行添加入口

## 右键菜单

`src/pages/Library.tsx` 的曲目行支持 `onContextMenu`：

- 列出全部播放列表及曲目数，点击即加入；已存在曲目不重复插入，提示“已在列表中”。
- 末尾“新建播放列表”，输入名称后创建并立即加入当前曲目。
- 点击空白、按 Esc、再次右键关闭；菜单定位限制在视口内。
- 仍保留工具栏的“目标播放列表 + 加入”路径。

## 命令行

```bash
BiuPlayer --playlist <playlist-id> <file-or-dir>...
```

- `--playlist` 后必须是数字 ID；其余非 `--` 参数视为文件/目录。
- 文件：扫描其父目录以入库（标签/时长等），但只把该文件加入目标列表。
- 目录：递归扫描并把该目录下全部曲目加入列表。
- 重复项由 `add_tracks_to_playlist` 去重，返回新增数量。
- 已运行的实例：`single-instance` 插件把参数转发给前端，复用同一处理逻辑。
- 首次启动：`startup_args` 命令把参数交给 `Shell.tsx`，在挂载后执行。

实现位置：

- `src-tauri/src/commands/system.rs::startup_args`
- `src-tauri/src/commands/library.rs::playlist_add_paths`
- `src-tauri/crates/biu-library/src/lib.rs::{track_id_by_path,track_ids_under}`
- `src/Shell.tsx::handleArgs`

## 验证

- `track_lookup_by_path_and_root`：根目录/子目录路径查找、`track_ids_under` 递归范围。
- `playlist_copy_bulk_remove_and_reorder`：复制、批量删除、重排。
- `cargo test -p biu-library --test playlists`：3/3。
- `cargo clippy --workspace --all-targets -- -D warnings`、`pnpm build` 通过。

## 限制

- 命令行只接受播放列表 ID，不提供按名称查找；ID 需从 UI 或数据库获取。
- 单文件会触发父目录扫描，目录很大时首次较慢。
- 不支持 `--playlist` 与普通播放参数混用；带 `--playlist` 时不会自动播放。
