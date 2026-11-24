import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";

const nav = [
  { to: "/", label: "正在播放", end: true },
  { to: "/library", label: "媒体库" },
  { to: "/playlists", label: "播放列表" },
  { to: "/search", label: "搜索" },
  { to: "/equalizer", label: "均衡器" },
  { to: "/skins", label: "皮肤" },
  { to: "/plugins", label: "插件" },
  { to: "/tools", label: "音频工具" },
  { to: "/settings/general", label: "设置" },
  { to: "/m0", label: "M0 验证台" },
];

type Theme = "dark" | "light";

function initialTheme(): Theme {
  const saved = localStorage.getItem("biu.theme");
  return saved === "light" ? "light" : "dark";
}

export function Shell() {
  const [theme, setTheme] = useState<Theme>(initialTheme);

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
            <strong>博音</strong>
            <p>BiuPlayer</p>
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
              {item.label}
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-foot">
          <button className="ghost" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
            {theme === "dark" ? "浅色主题" : "深色主题"}
          </button>
          <span className="muted">v0.1.0 · M0</span>
        </div>
      </aside>

      <div className="main">
        <header className="titlebar">
          <div>
            <span className="muted">本地优先 · 离线可用</span>
          </div>
          <div className="titlebar-actions">
            <input className="search" placeholder="搜索曲目、专辑、艺术家（⌘K）" disabled />
          </div>
        </header>
        <main className="content">
          <Outlet />
        </main>
        <footer className="playerbar">
          <div className="player-meta">
            <div className="cover-placeholder">♪</div>
            <div>
              <strong>未在播放</strong>
              <p className="muted">选择一个文件夹开始</p>
            </div>
          </div>
          <div className="player-controls">
            <button className="ghost" disabled>⏮</button>
            <button className="play" disabled>▶</button>
            <button className="ghost" disabled>⏭</button>
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
