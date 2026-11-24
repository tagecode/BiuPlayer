# L1 曲目虚拟滚动与列排序

## 目标

补齐 F-PL-11 的 10 万条虚拟滚动基础，并为 F-PL-05 提供列排序。

## 后端

- `Database::list_tracks_sorted(limit, offset, sort, descending)`：
  - 支持 `title`、`artist`、`album`、`duration`、`added`、`path` 六种排序。
  - 排序列走白名单映射，不拼接用户输入；方向只允许 ASC/DESC。
  - 以 `t.id` 作为稳定次序，避免同名曲目翻页重复/丢失。
- `list_tracks(limit, offset)` 保留为按标题升序的兼容包装。
- `library_tracks(limit, offset, sort, descending)` 命令；`limit` 上限 1000。

## 前端

- `src/pages/Library.tsx` 改为窗口化列表：
  - 固定行高 36px，视口 480px，上下各 6 行 overscan。
  - 只为可见范围请求 200 条/页，页缓存保存在 `Record<index, Track>`。
  - 滚动时按页加载；`generation` 计数丢弃排序切换后的过期响应。
  - 列头点击切换排序，再次点击反向；切换时清空缓存并回到顶部。
  - 双击行播放；点击“播放”会取当前行附近 500 条作为队列。
- 搜索结果仍保持简单表格（最多 50 条）。

## 验证

- `pnpm build` 通过：JS 378.85 kB / gzip 117.75 kB，CSS 19.05 kB。
- `cargo test --workspace`：51 个测试通过。
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --check` 通过。
- 当前环境未生成 10 万条真实曲目做 55fps 实测；窗口化逻辑与分页查询已就位。

## 限制

- 行高固定，尚未支持紧凑/舒适密度切换。
- 手动排序（F-PL-05 后半）未实现；当前只支持列排序。
- 搜索高亮未实现。
- 队列只取点击位置附近 500 条，不是完整列表。
