# 错误码文案映射与全前端本地化

## 目标

1. 后端 `CommandError` 的 `code` 不再直接暴露给用户，按当前界面语言给出可读文案。
2. 清理设置页之外剩余的硬编码中文，覆盖主界面、媒体库、播放列表、播放页、桌面歌词与 M0 验证台。

## 错误码映射

新增 `src/errors.ts`：

```ts
formatError(error: unknown, t: Translate): string
```

- `E_INVALID_ARGUMENT` → `error.E_INVALID_ARGUMENT`
- `E_AUDIO_DEVICE` → `error.E_AUDIO_DEVICE`
- `E_INTERNAL` → `error.E_INTERNAL`
- `E_WINAMP` → `error.E_WINAMP`
- 未识别 code：保留 `code: message`，避免吞掉诊断信息
- 非对象/无 message：回退 `error.unknown`

所有页面（`Library`、`Playlists`、`Albums`、`Artists`、`AlbumDetail`、`ArtistDetail`、`NowPlaying`、`Settings`、`Shell`、`App`）统一改用 `formatError(error, t)`，删除各自的 `errorText`/`String(error)`。

## i18n 能力

`t(key, vars?)` 支持 `{name}` 占位符插值，使用 `split/join` 实现，兼容当前 TS `target`（无 `replaceAll`）。

## 本地化范围

- `src/components/TrackTable.tsx`：表头、空态、未知艺术家
- `src/pages/Albums.tsx` / `Artists.tsx`：标题、计数、空态、未知艺术家
- `src/pages/AlbumDetail.tsx` / `ArtistDetail.tsx`：面包屑、标题、曲目数、播放全部/专辑
- `src/pages/Playlists.tsx` / `PlaylistDetail.tsx`：创建/重命名/删除确认、空态、表头、移除
- `src/pages/Library.tsx`：扫描状态、暂停/继续/取消、搜索、加入播放列表、虚拟列表行操作
- `src/pages/NowPlaying.tsx`：标题/艺术家、歌词、队列、桌面歌词按钮
- `src/pages/DesktopLyrics.tsx`：默认标题、等待歌词
- `src/Shell.tsx`：拖放提示、播放/扫描通知、侧栏宽度 aria
- `src/components/AlbumCover.tsx`：封面 alt
- `src/App.tsx`：M0 验证台全部文案

新增 `common.*`、`albums.*`、`artists.*`、`playlist.*`、`library.*`、`np.*`、`desktop.*`、`shell.*`、`cover.*`、`error.*`、`m0.*` 三语键。

## 验证

- `pnpm build` 通过：JS 420.28 kB / gzip 127.90 kB。
- `cargo fmt --all --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 通过。
- 全前端扫描仅剩语言选择原生名称“简体中文 / 繁體中文”，符合语言选择器惯例。

## 限制

- 语言切换后，`App.tsx` 的初始 `eventStatus` 不会重新计算（仅影响 M0 验证台首次提示）。
- 后端日志与 `CommandError.message` 仍是英文；界面显示走映射，诊断包保留原文。
