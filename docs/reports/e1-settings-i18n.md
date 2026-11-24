# 设置页 i18n 清理

## 范围

- `src/pages/Settings.tsx` 中所有面向用户的硬编码中文（分节标题、字段名、选项、提示、按钮）改为 `t(...)`。
- 仅保留语言选择里的“简体中文 / 繁體中文 / English”原生名称，符合语言选择器惯例。

## 新增键

共新增 41 个 `settings.*` 键，三份资源同步：

- 分节：`settings.section.{general,audio,playback,lyrics,interface,about,data}`
- 常规：`settings.language`、`theme*`、`startMinimized`
- 音频：`outputDevice`、`defaultDevice`、`defaultTag`、`deviceHint`、`bitPerfectHint`
- 播放：`resumeOnStart`、`gapless`、`crossfade`、`defaultMode`、`mode.*`
- 歌词：`lyricsOffset`、`desktopLyricsSize`、`desktopLyricsColor`
- 界面：`density`、`density.*`、`reduceMotion`、`alwaysOnTop`、`opacity`
- 关于/数据：`checkUpdates`、`openLicense`、`licenseLine`、`dataNote`、`savedNote`、`loading`、`deviceChanged`、`schema`

## 验证

- `pnpm build` 通过：JS 404.19 kB / gzip 124.11 kB。
- `MessageKey` 由 `zhCN` 推导，`zhTW`/`en` 用 `Record<MessageKey,string>`；缺键会在编译期报错。
- 未做三种语言的逐页截图；语言切换后设置页文案随 `I18nProvider` 重渲染。

## 限制

- 后端错误码仍直接显示 `code: message`，未翻译。
- 语言名称保持原生写法，不随界面语言变化。
