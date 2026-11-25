# 设置分类口径与导航（F-SET-01）

日期：2026-11-25

## 口径调整

MVP 设置中心固定为 8 类，与页面导航一致：

1. 常规
2. 播放
3. 音频
4. 歌词
5. 界面
6. 媒体库
7. 数据
8. 关于

延后项不再作为空分类出现：

- 音效：归入独立 EQ/DSP 页面。
- 皮肤、插件、快捷键：M2/M3 能力，接入后再进入设置。
- 高级：由“数据/诊断”承接。

已同步 `docs/FeatureList.md` 的 `F-SET-01` 与 `docs/Prd.md` 的 P-04 / `FR-SET-01`。

## 实现

- `src/pages/Settings.tsx` 左侧新增分类导航，按钮跳转 `/settings/<section>`。
- 每个分类卡片设置 `id="settings-<section>"` 与 `scroll-margin-top`。
- 路由参数变化后 `scrollIntoView` 定位到对应卡片，并按当前 section 高亮导航。
- 页面顺序调整为：常规 → 播放 → 音频 → 歌词 → 界面 → 媒体库 → 数据 → 关于。
- 窄屏（≤900px）导航变为横向换行，不占用侧栏宽度。

## 验收

- `pnpm build` 通过：JS 447.51 kB / gzip 135.16 kB。
- 构建通过；未做浏览器自动化或人工点击验收，`scrollIntoView` 定位与导航高亮尚未在真实窗口中验证。
- 分类导航的 URL 同步逻辑基于 `useParams`：点击按钮写入 `/settings/<section>`，参数变化触发滚动与高亮。
