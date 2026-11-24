import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { useI18n, type Lang, type MessageKey } from "./i18n";

const nav: { to: string; labelKey: MessageKey; end?: boolean }[] = [
  { to: "/", labelKey: "nav.nowPlaying", end: true },
  { to: "/library", labelKey: "nav.library" },
  { to: "/playlists", labelKey: "nav.playlists" },
  { to: "/search", labelKey: "nav.search" },
  { to: "/equalizer", labelKey: "nav.equalizer" },
  { to: "/skins", labelKey: "nav.skins" },
  { to: "/plugins", labelKey: "nav.plugins" },
  { to: "/tools", labelKey: "nav.tools" },
  { to: "/settings/general", labelKey: "nav.settings" },
  { to: "/m0", labelKey: "nav.m0" },
];

type Theme = "dark" | "light";

function initialTheme(): Theme {
  const saved = localStorage.getItem("biu.theme");
  return saved === "light" ? "light" : "dark";
}

export function Shell() {
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const { lang, setLang, t } = useI18n();

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("biu.theme", theme);
  }, [theme]);

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">Biu</span>
          <div>
            <strong>{t("app.name")}</strong>
            <p>{t("app.tagline")}</p>
          </div>
        </div>
        <nav className="nav">
          {nav.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              className={({ isActive }) => (isActive ? "nav-item active" : "nav-item")}
            >
              {t(item.labelKey)}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-foot">
          <select
            className="lang-select"
            aria-label={t("lang.label")}
            value={lang}
            onChange={(event) => setLang(event.target.value as Lang)}
          >
            <option value="zh-CN">简体中文</option>
            <option value="zh-TW">繁體中文</option>
            <option value="en">English</option>
          </select>
          <button className="ghost" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
            {theme === "dark" ? t("theme.toLight") : t("theme.toDark")}
          </button>
          <span className="muted">{t("app.version")}</span>
        </div>
      </aside>

      <div className="main">
        <header className="titlebar">
          <div>
            <span className="muted">{t("shell.offline")}</span>
          </div>
          <div className="titlebar-actions">
            <input className="search" placeholder={t("shell.search")} disabled />
          </div>
        </header>
        <main className="content">
          <Outlet />
        </main>
        <footer className="playerbar">
          <div className="player-meta">
            <div className="cover-placeholder">♪</div>
            <div>
              <strong>{t("player.none")}</strong>
              <p className="muted">{t("player.hint")}</p>
            </div>
          </div>
          <div className="player-controls">
            <button className="ghost" aria-label={t("player.prev")} disabled>⏮</button>
            <button className="play" aria-label={t("player.play")} disabled>▶</button>
            <button className="ghost" aria-label={t("player.next")} disabled>⏭</button>
          </div>
          <div className="player-progress">
            <span className="mono">0:00</span>
            <div className="progress"><span /></div>
            <span className="mono">0:00</span>
          </div>
        </footer>
      </div>
    </div>
  );
}
