# E1-05 路由、布局与主题令牌

日期：2026-11-24
任务：E1-05（2 人日）
状态：**完成（占位页 + 外壳 + 主题）**

## 1. 路由

使用 `createHashRouter`（ADR-011），入口 `src/router.tsx`：

| 路由 | 页面 |
|---|---|
| `/` | P-01 正在播放 |
| `/library` | P-02 曲目 |
| `/library/albums`、`/library/albums/:id` | P-03/P-04 专辑 |
| `/library/artists`、`/library/artists/:id` | P-05 艺术家 |
| `/library/genres` | P-06 流派 |
| `/playlists`、`/playlists/:id` | P-07 播放列表 |
| `/search` | P-08 搜索 |
| `/lyrics/:trackId` | P-09 歌词编辑 |
| `/equalizer` | P-10 均衡器 |
| `/skins` | P-11 皮肤 |
| `/plugins` | P-12 插件 |
| `/settings/:section` | P-13 设置 |
| `/tools` | P-14 音频工具 |
| `/m0` | M0 验证台 |

`/settings` 重定向 `/settings/general`；未知路由回 `/`。

## 2. 布局

`src/Shell.tsx`：

- 左侧栏：品牌、10 个一级导航、主题切换、版本。
- 顶部栏：状态文案、搜索框占位（禁用）。
- 主内容区：`<Outlet />`，独立滚动。
- 底部播放栏：封面占位、上一首/播放/下一首（禁用）、进度占位。
- 窄屏（<860px）侧栏收成 64px 图标栏，播放栏精简。

## 3. 主题

- CSS 变量在 `src/App.css`（深色默认）与 `src/shell.css`（`[data-theme="light"]` 覆盖）。
- 切换写入 `localStorage["biu.theme"]`。
- `index.html` 在样式加载前用内联脚本设置 `data-theme`，避免首屏闪烁。
- 令牌：`--bg`、`--panel`、`--panel-2`、`--line`、`--text`、`--muted`、`--accent`、`--accent-dim`、`--warn`、`--error`。

## 4. 验证

```bash
pnpm build
```

结果：TypeScript 通过，Vite 产出 `index-BrYlRqd4.js` 330.68 kB（gzip 105.72 kB）、CSS 9.17 kB（gzip 2.66 kB）。

本机启动 `target/debug/biuplayer` 正常；`screencapture` 因 macOS 屏幕录制权限不可用，未附截图。

## 5. 未完成

- 所有页面为占位，真实数据/交互在 E1-07、L1、P1、A1 补齐。
- 播放栏尚未接 `player:state`。
- 未做键盘焦点顺序与 `aria-current` 细化；E1-07/无障碍任务补。
- 主题只覆盖应用外壳与卡片，M0 验证台的局部颜色仍用硬编码。
