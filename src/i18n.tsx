import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

export type Lang = "zh-CN" | "zh-TW" | "en";

const zhCN = {
  "app.name": "博音",
  "app.tagline": "BiuPlayer",
  "app.version": "v0.1.0 · M0",
  "nav.nowPlaying": "正在播放",
  "nav.library": "媒体库",
  "nav.playlists": "播放列表",
  "nav.search": "搜索",
  "nav.equalizer": "均衡器",
  "nav.skins": "皮肤",
  "nav.plugins": "插件",
  "nav.tools": "音频工具",
  "nav.settings": "设置",
  "nav.m0": "M0 验证台",
  "theme.toLight": "浅色主题",
  "theme.toDark": "深色主题",
  "lang.label": "语言",
  "shell.offline": "本地优先 · 离线可用",
  "shell.search": "搜索曲目、专辑、艺术家（⌘K）",
  "player.none": "未在播放",
  "player.hint": "选择一个文件夹开始",
  "player.prev": "上一首",
  "player.play": "播放",
  "player.next": "下一首",
  "page.todoTitle": "本页待实现",
  "page.todoHint": "占位页 · 后续任务填充",
  "page.todo.0": "接入数据与 IPC 命令",
  "page.todo.1": "补齐加载、空、错误状态",
  "page.todo.2": "补充键盘与无障碍行为",
  "pages.nowPlaying.title": "正在播放",
  "pages.nowPlaying.desc": "封面、曲目信息、歌词、可视化与队列的主视图。",
  "pages.library.title": "曲目",
  "pages.library.desc": "本地曲库的默认视图，虚拟列表承载 10 万级曲目。",
  "pages.albums.title": "专辑",
  "pages.albums.desc": "按专辑聚合，支持封面网格与详情。",
  "pages.album.title": "专辑详情",
  "pages.album.desc": "单张专辑的曲目、封面与元数据。",
  "pages.artists.title": "艺术家",
  "pages.artists.desc": "按艺术家聚合，支持专辑与曲目下钻。",
  "pages.artist.title": "艺术家详情",
  "pages.artist.desc": "艺术家的专辑、曲目与简介。",
  "pages.genres.title": "流派",
  "pages.genres.desc": "按流派浏览曲库。",
  "pages.playlists.title": "播放列表",
  "pages.playlists.desc": "手动与智能播放列表管理。",
  "pages.playlist.title": "播放列表详情",
  "pages.playlist.desc": "列表内曲目与批量操作。",
  "pages.search.title": "搜索",
  "pages.search.desc": "曲目、专辑、艺术家、歌词的统一搜索。",
  "pages.lyrics.title": "歌词编辑",
  "pages.lyrics.desc": "逐行打轴、偏移与保存。",
  "pages.equalizer.title": "均衡器",
  "pages.equalizer.desc": "10 段均衡器与预设。",
  "pages.skins.title": "皮肤",
  "pages.skins.desc": "皮肤预览、安装与切换。",
  "pages.plugins.title": "插件",
  "pages.plugins.desc": "WASM/原生插件管理、权限与日志。",
  "pages.settings.title": "设置",
  "pages.settings.desc": "常规、播放、音频、音效、歌词、皮肤、插件、快捷键、媒体库、高级。",
  "pages.tools.title": "音频工具",
  "pages.tools.desc": "格式转换、抓轨、文件整理与校验（P2）。",
} as const;

export type MessageKey = keyof typeof zhCN;

const zhTW: Record<MessageKey, string> = {
  "app.name": "博音",
  "app.tagline": "BiuPlayer",
  "app.version": "v0.1.0 · M0",
  "nav.nowPlaying": "正在播放",
  "nav.library": "媒體庫",
  "nav.playlists": "播放清單",
  "nav.search": "搜尋",
  "nav.equalizer": "等化器",
  "nav.skins": "面板",
  "nav.plugins": "外掛",
  "nav.tools": "音訊工具",
  "nav.settings": "設定",
  "nav.m0": "M0 驗證台",
  "theme.toLight": "淺色主題",
  "theme.toDark": "深色主題",
  "lang.label": "語言",
  "shell.offline": "本地優先 · 離線可用",
  "shell.search": "搜尋曲目、專輯、演出者（⌘K）",
  "player.none": "未在播放",
  "player.hint": "選擇一個資料夾開始",
  "player.prev": "上一首",
  "player.play": "播放",
  "player.next": "下一首",
  "page.todoTitle": "本頁待實作",
  "page.todoHint": "佔位頁 · 後續任務填充",
  "page.todo.0": "接入資料與 IPC 命令",
  "page.todo.1": "補齊載入、空、錯誤狀態",
  "page.todo.2": "補充鍵盤與無障礙行為",
  "pages.nowPlaying.title": "正在播放",
  "pages.nowPlaying.desc": "封面、曲目資訊、歌詞、視覺化與佇列的主視圖。",
  "pages.library.title": "曲目",
  "pages.library.desc": "本地曲庫的預設視圖，虛擬列表承載 10 萬級曲目。",
  "pages.albums.title": "專輯",
  "pages.albums.desc": "按專輯聚合，支援封面網格與詳情。",
  "pages.album.title": "專輯詳情",
  "pages.album.desc": "單張專輯的曲目、封面與中繼資料。",
  "pages.artists.title": "演出者",
  "pages.artists.desc": "按演出者聚合，支援專輯與曲目下鑽。",
  "pages.artist.title": "演出者詳情",
  "pages.artist.desc": "演出者的專輯、曲目與簡介。",
  "pages.genres.title": "曲風",
  "pages.genres.desc": "按曲風瀏覽曲庫。",
  "pages.playlists.title": "播放清單",
  "pages.playlists.desc": "手動與智慧播放清單管理。",
  "pages.playlist.title": "播放清單詳情",
  "pages.playlist.desc": "清單內曲目與批次操作。",
  "pages.search.title": "搜尋",
  "pages.search.desc": "曲目、專輯、演出者、歌詞的統一搜尋。",
  "pages.lyrics.title": "歌詞編輯",
  "pages.lyrics.desc": "逐行打軸、偏移與儲存。",
  "pages.equalizer.title": "等化器",
  "pages.equalizer.desc": "10 段等化器與預設。",
  "pages.skins.title": "面板",
  "pages.skins.desc": "面板預覽、安裝與切換。",
  "pages.plugins.title": "外掛",
  "pages.plugins.desc": "WASM/原生外掛管理、權限與日誌。",
  "pages.settings.title": "設定",
  "pages.settings.desc": "一般、播放、音訊、音效、歌詞、面板、外掛、快速鍵、媒體庫、進階。",
  "pages.tools.title": "音訊工具",
  "pages.tools.desc": "格式轉換、抓軌、檔案整理與校驗（P2）。",
};

const en: Record<MessageKey, string> = {
  "app.name": "BiuPlayer",
  "app.tagline": "BiuPlayer",
  "app.version": "v0.1.0 · M0",
  "nav.nowPlaying": "Now Playing",
  "nav.library": "Library",
  "nav.playlists": "Playlists",
  "nav.search": "Search",
  "nav.equalizer": "Equalizer",
  "nav.skins": "Skins",
  "nav.plugins": "Plugins",
  "nav.tools": "Audio Tools",
  "nav.settings": "Settings",
  "nav.m0": "M0 Bench",
  "theme.toLight": "Light theme",
  "theme.toDark": "Dark theme",
  "lang.label": "Language",
  "shell.offline": "Local-first · Offline-ready",
  "shell.search": "Search tracks, albums, artists (⌘K)",
  "player.none": "Nothing playing",
  "player.hint": "Choose a folder to start",
  "player.prev": "Previous",
  "player.play": "Play",
  "player.next": "Next",
  "page.todoTitle": "To be implemented",
  "page.todoHint": "Placeholder · filled by later tasks",
  "page.todo.0": "Wire data and IPC commands",
  "page.todo.1": "Add loading, empty and error states",
  "page.todo.2": "Add keyboard and accessibility behavior",
  "pages.nowPlaying.title": "Now Playing",
  "pages.nowPlaying.desc": "Cover, track info, lyrics, visualizer and queue.",
  "pages.library.title": "Tracks",
  "pages.library.desc": "Default library view; a virtual list for 100k tracks.",
  "pages.albums.title": "Albums",
  "pages.albums.desc": "Grouped by album with cover grid and details.",
  "pages.album.title": "Album",
  "pages.album.desc": "Tracks, cover and metadata of one album.",
  "pages.artists.title": "Artists",
  "pages.artists.desc": "Grouped by artist with album and track drill-down.",
  "pages.artist.title": "Artist",
  "pages.artist.desc": "Albums, tracks and biography of one artist.",
  "pages.genres.title": "Genres",
  "pages.genres.desc": "Browse the library by genre.",
  "pages.playlists.title": "Playlists",
  "pages.playlists.desc": "Manual and smart playlist management.",
  "pages.playlist.title": "Playlist",
  "pages.playlist.desc": "Tracks and bulk actions inside a playlist.",
  "pages.search.title": "Search",
  "pages.search.desc": "Unified search for tracks, albums, artists and lyrics.",
  "pages.lyrics.title": "Lyrics Editor",
  "pages.lyrics.desc": "Line-by-line timing, offset and save.",
  "pages.equalizer.title": "Equalizer",
  "pages.equalizer.desc": "10-band equalizer and presets.",
  "pages.skins.title": "Skins",
  "pages.skins.desc": "Preview, install and switch skins.",
  "pages.plugins.title": "Plugins",
  "pages.plugins.desc": "WASM/native plugins, permissions and logs.",
  "pages.settings.title": "Settings",
  "pages.settings.desc": "General, playback, audio, DSP, lyrics, skins, plugins, shortcuts, library, advanced.",
  "pages.tools.title": "Audio Tools",
  "pages.tools.desc": "Convert, rip, organize and verify (P2).",
};

const messages: Record<Lang, Record<MessageKey, string>> = { "zh-CN": zhCN, "zh-TW": zhTW, en };

type I18nValue = {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (key: MessageKey) => string;
};

const I18nContext = createContext<I18nValue | null>(null);

function initialLang(): Lang {
  const saved = localStorage.getItem("biu.lang");
  return saved === "zh-TW" || saved === "en" ? saved : "zh-CN";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>(initialLang);

  useEffect(() => {
    document.documentElement.lang = lang;
    localStorage.setItem("biu.lang", lang);
  }, [lang]);

  const t = useCallback((key: MessageKey) => messages[lang][key], [lang]);
  const value = useMemo(() => ({ lang, setLang, t }), [lang, t]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be used inside I18nProvider");
  return ctx;
}
