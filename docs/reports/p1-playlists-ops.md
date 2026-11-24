# 播放列表完整性：复制、拖拽排序、多选、撤销、插播

## 范围

对应 MVP P0：F-PL-01 / F-PL-02 / F-PL-03 / F-PL-05 / F-PL-08 的补齐部分。

## 后端

`crates/biu-library` 新增：

- `copy_playlist(id, name)`：复制列表元信息与全部条目，位置保持；空名称拒绝。
- `remove_playlist_items(id, positions)`：一次事务删除多条，位置越界拒绝，剩余条目重建连续 position。
- `reorder_playlist_items(id, track_ids)`：要求请求集合与当前集合完全一致（排序后比较），避免误删/漏项；事务内重建 position。

命令：

- `playlist_copy(id, name)`
- `playlist_remove_items(id, positions)`
- `playlist_reorder(id, track_ids)`
- `player_queue_play_next(item)`：把曲目插入当前项之后；队列为空时追加但不伪造播放状态。

## 前端

`src/pages/PlaylistDetail.tsx`：

- **复制**：顶部“复制”按钮，输入新名称后跳转到新列表。
- **多选**：表头/行内复选框；工具栏显示已选数量；批量删除。
- **撤销**：删除/排序前保存快照；撤销时先补回缺失曲目，再按快照顺序重排；保留最近 20 步。
- **拖拽排序**：HTML5 drag 行，放下后乐观更新并调用 `playlistReorder`，失败回滚。
- **上下移**：保留原按钮，作为键盘/精确操作的补充。
- **下一首播放**：每行“下一首播放”调用 `playerQueuePlayNext`。
- **单条删除**：直接删除该行，同样进入撤销栈。

样式：`src/shell.css` 新增 `.playlist-toolbar`、`.checkbox-row`、`.selected-row`、`.drop-target`、`.drag-cell`。

## 验证

- `crates/biu-library/tests/playlists.rs` 新增 `playlist_copy_bulk_remove_and_reorder`：
  - 复制 3 首列表后条目与顺序一致；
  - 空名称复制被拒绝；
  - 逆序重排成功，非法集合被拒绝；
  - 批量删除 2 条后只剩 1 条且 position 从 0 开始。
- `cargo test -p biu-library --test playlists`：2/2 通过。
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all` 通过。
- `pnpm build`：JS 424.51 kB / gzip 129.14 kB。

## 未覆盖

- 1 万条拖拽 ≥55fps 未实测。
- 右键菜单、命令行参数添加到播放列表未做（F-PL-02 仍部分）。
- 播放列表本身（列表的列表）手动排序未做，当前按名称排序（F-PL-01 仍部分）。
