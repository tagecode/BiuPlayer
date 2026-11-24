# E1-07 基础组件与状态页

日期：2026-11-24
任务：E1-07（1.5 人日）
状态：**基础组件完成**（未接 Storybook，使用开发路由代替）

## 1. 组件

- `src/components/StatusView.tsx`：`loading | empty | error | offline` 四态；`role="status"`/`alert`；可选 action。
- `src/components/Dialog.tsx`：原生 `<dialog>` 封装；`showModal()`/`close()`；Escape 与遮罩关闭；标题 + body + 调用方 footer。
- 样式在 `src/shell.css`：`.status-view`、`.status-icon`、`.dialog`、`.dialog-actions`，使用主题令牌。

## 2. 预览路由

`/dev/states`（不在一级导航）：

- 四态卡片：Loading / No tracks yet / Playback failed / Offline mode。
- Dialog 打开/关闭、Escape 关闭。
- 该路由用于设计与无障碍走查，后续可替换为 Storybook 或截图基线。

## 3. 验证

```bash
pnpm build
```

结果：TypeScript 通过；Vite 产出 `index-CsedrW2L.js` 340.31 kB（gzip 108.41 kB）、CSS 10.67 kB。

手工走查清单：

- [x] 四态在深色主题下对比度可读。
- [x] 错误态使用 `role="alert"`，其余 `role="status"`。
- [x] Dialog 打开后焦点进入 dialog，Escape 关闭。
- [x] 关闭按钮有 `aria-label`。
- [ ] 浅色主题截图（本机屏幕录制权限受限，未附）。
- [ ] 移动/窄屏状态网格（<860px 单列）未截图。

## 4. 未完成

- 未引入 Storybook，避免 MVP 增加构建依赖。
- 缺少按钮/输入框/列表的独立组件封装；当前复用全局 CSS 类，等 L1 列表与 P1 播放页出现重复时再抽象。
- 未做 axe/自动化无障碍扫描。
