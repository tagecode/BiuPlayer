# 列表交互缺口收尾（F-PL-01/02/08）

日期：2026-11-25
对应：`docs/Mvp.md` F-PL-01、F-PL-02、F-PL-08

## 实现

### 播放列表手动排序（F-PL-01）

- schema v3：`playlists.sort_order INTEGER NOT NULL DEFAULT 0`，迁移时按 `id` 初始化；旧库迁移前仍复制 `library.db.bak`。
- `Database::reorder_playlists(&[i64])`：要求 id 集合与库内完全一致（无重复、无遗漏），事务内重写 `sort_order` 并刷新 `updated_at`。
- `create_playlist` / `copy_playlist` 追加到末尾。
- `list_playlists` 改为 `ORDER BY sort_order, id`。
- `/playlists` 行支持 HTML5 拖拽；松手后乐观更新并发起 `playlist_reorder_all`，失败时重新拉取。

### 拖放到指定播放列表（F-PL-02）

- 全局拖放处理在 `#/playlists/<id>` 路由下改为 `playlist_add_paths`，文件/目录都直接加入该列表，不触发播放或扫描。
- 其他路由保持原行为：音频播放、目录扫描。

### 队列内调整顺序（F-PL-08）

- `AudioQueue::move_item(from, to)`：移动后按规则修正当前索引；当前项被移动时跟随，其他项移动时保持当前曲目。
- 新命令 `player_queue_move`；队列面板行支持拖拽，拖拽后刷新快照。
- 新增单测覆盖“移动非当前项保持当前曲目”和越界拒绝。

## 验证

- `cargo test --workspace`：57 passed / 0 failed。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo fmt --check`：通过。
- `pnpm build`：通过，JS 435.32 kB / gzip 131.72 kB。

## 已知限制

- 拖拽排序未做 1 万条性能实测（按计划延后）。
- 拖放到 `/playlists` 列表页本身不加入某个列表，需进入具体列表详情页。
- 队列拖拽是 HTML5 行拖拽，未做触摸端与键盘排序。
