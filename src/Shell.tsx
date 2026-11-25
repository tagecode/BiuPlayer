import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { formatError } from "./errors";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  commands,
  type CrashReport,
  type PlayerStateDto_Serialize,
  type QueueSnapshotDto_Serialize,
} from "./bindings";
import { useI18n, type Lang, type MessageKey } from "./i18n";
import { applyFrameless, applyOpacity } from "./window";
import { applyTheme, resolveTheme } from "./theme";

const nav: { to: string; labelKey: MessageKey; icon: string; end?: boolean }[] = [
  { to: "/", labelKey: "nav.nowPlaying", icon: "▶", end: true },
  { to: "/library", labelKey: "nav.library", icon: "♪" },
  { to: "/playlists", labelKey: "nav.playlists", icon: "≡" },
  { to: "/search", labelKey: "nav.search", icon: "⌕" },
  { to: "/equalizer", labelKey: "nav.equalizer", icon: "♬" },
  { to: "/skins", labelKey: "nav.skins", icon: "◈" },
  { to: "/plugins", labelKey: "nav.plugins", icon: "⬡" },
  { to: "/tools", labelKey: "nav.tools", icon: "⚒" },
  { to: "/settings/general", labelKey: "nav.settings", icon: "⚙" },
  { to: "/m0", labelKey: "nav.m0", icon: "◎" },
];

const modes = ["sequential", "list_loop", "single_loop", "shuffle"] as const;
type PlayMode = (typeof modes)[number];

const modeKey: Record<PlayMode, MessageKey> = {
  sequential: "player.mode.sequential",
  list_loop: "player.mode.listLoop",
  single_loop: "player.mode.singleLoop",
  shuffle: "player.mode.shuffle",
};

type Theme = "dark" | "light";

function initialTheme(): Theme {
  // `initTheme()` already resolved the stored mode before React mounted; the
  // inline bootstrap in index.html did the same for the very first paint.
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

function formatTime(ms: number | null | undefined): string {
  if (!ms || ms <= 0) return "0:00";
  const total = Math.floor(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}



const AUDIO_EXTENSIONS = new Set([
  "mp3", "flac", "wav", "m4a", "aac", "ogg", "oga", "aiff", "aif",
]);

function isAudioPath(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return AUDIO_EXTENSIONS.has(ext);
}

function nameFromPath(path: string): string {
  return path.split(/[\\/]/).pop() || path;
}

export function Shell() {
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const { lang, setLang, t } = useI18n();
  const navigate = useNavigate();
  const [player, setPlayer] = useState<PlayerStateDto_Serialize | null>(null);
  const [queue, setQueue] = useState<QueueSnapshotDto_Serialize | null>(null);
  const [queueOpen, setQueueOpen] = useState(false);
  const [queueDrag, setQueueDrag] = useState<number | null>(null);
  const [notice, setNotice] = useState("");
  const [dragging, setDragging] = useState(false);
  const [topQuery, setTopQuery] = useState("");
  const [crash, setCrash] = useState<CrashReport | null>(null);
  const [frameless, setFrameless] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(220);
  const [collapsed, setCollapsed] = useState(false);
  const sidebarWidthRef = useRef(220);

  useEffect(() => {
    commands
      .settingsGet()
      .then((settings) => {
        const width = Math.min(400, Math.max(160, settings.sidebarWidth || 220));
        sidebarWidthRef.current = width;
        setSidebarWidth(width);
        setCollapsed(Boolean(settings.sidebarCollapsed));
        const opacity = Math.min(1, Math.max(0.7, settings.opacity ?? 1));
        document.documentElement.style.setProperty("--ui-opacity", String(opacity));
        void getCurrentWindow().setAlwaysOnTop(Boolean(settings.alwaysOnTop)).catch(() => {});
        // settings.json is the source of truth (portable installs keep it next
        // to the executable), so re-apply it instead of trusting localStorage.
        applyTheme(settings.theme);
        setTheme(
          resolveTheme(
            settings.theme,
            window.matchMedia("(prefers-color-scheme: dark)").matches,
          ),
        );
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    commands
      .settingsGet()
      .then((settings) => {
        setFrameless(settings.frameless);
        void applyFrameless(settings.frameless);
        void applyOpacity(settings.opacity ?? 1);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    const sync = (event: Event) => {
      const value = Boolean((event as CustomEvent).detail);
      setFrameless(value);
      void applyFrameless(value);
    };
    window.addEventListener("biu:frameless", sync);
    return () => window.removeEventListener("biu:frameless", sync);
  }, []);

  useEffect(() => {
    commands
      .crashReport()
      .then((report) => {
        if (!report) return;
        if (localStorage.getItem("biu.crash.dismissed") === report.file) return;
        setCrash(report);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    const sync = (event: Event) => setCollapsed(Boolean((event as CustomEvent).detail));
    window.addEventListener("biu:sidebar-collapsed", sync);
    return () => window.removeEventListener("biu:sidebar-collapsed", sync);
  }, []);

  function toggleSidebar() {
    const next = !collapsed;
    setCollapsed(next);
    void commands.settingsUpdate({ sidebarCollapsed: next }).catch((error) => {
      setCollapsed(!next);
      setNotice(formatError(error, t));
    });
  }

  function startSidebarResize(event: React.PointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sidebarWidthRef.current;
    const move = (moveEvent: PointerEvent) => {
      const width = Math.min(400, Math.max(160, startWidth + moveEvent.clientX - startX));
      sidebarWidthRef.current = width;
      setSidebarWidth(width);
    };
    const stop = () => {
      document.removeEventListener("pointermove", move);
      document.removeEventListener("pointerup", stop);
      void commands.settingsUpdate({ sidebarWidth: sidebarWidthRef.current }).catch(() => {});
    };
    document.addEventListener("pointermove", move);
    document.addEventListener("pointerup", stop);
  }

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("biu.theme", theme);
  }, [theme]);

  useEffect(() => {
    const sync = (event: Event) => {
      const value = (event as CustomEvent).detail;
      if (value === "light" || value === "dark") setTheme(value);
    };
    window.addEventListener("biu:theme", sync);
    return () => window.removeEventListener("biu:theme", sync);
  }, []);

  function toggleTheme() {
    const next: Theme = theme === "dark" ? "light" : "dark";
    applyTheme(next);
    setTheme(next);
    void commands.settingsUpdate({ theme: next }).catch((error) => {
      setNotice(formatError(error, t));
    });
  }

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen<string>("tray:action", async (event) => {
      try {
        switch (event.payload) {
          case "play_pause": {
            const snapshot = await commands.playerSnapshot();
            if (snapshot.state === "playing") await commands.playerPause();
            else await commands.playerPlay();
            break;
          }
          case "prev":
            await commands.playerPrev();
            break;
          case "next":
            await commands.playerNext();
            break;
        }
      } catch (error) {
        console.warn("tray action failed", error);
      }
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen<PlayerStateDto_Serialize>("player:state", (event) => {
      if (!disposed) setPlayer(event.payload);
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    Promise.all([commands.playerSnapshot(), commands.playerQueue()])
      .then(([snapshot, snapshotQueue]) => {
        if (disposed) return;
        setPlayer(snapshot);
        setQueue(snapshotQueue);
      })
      .catch((error) => setNotice(formatError(error, t)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  async function run(action: () => Promise<unknown>) {
    try {
      setNotice("");
      await action();
    } catch (error) {
      setNotice(formatError(error, t));
    }
  }

  async function refreshQueue() {
    setQueue(await commands.playerQueue());
  }

  async function moveQueueItem(from: number, to: number) {
    if (from === to) return;
    try {
      setQueue(await commands.playerQueueMove(from, to));
    } catch (error) {
      setNotice(formatError(error, t));
    }
  }

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    const acceptPaths = async (paths: string[]) => {
      const playlistMatch = /^#\/playlists\/(\d+)$/.exec(window.location.hash);
      if (playlistMatch) {
        await addPathsToPlaylist(paths, Number(playlistMatch[1]));
        return;
      }
      const audio = paths.filter(isAudioPath);
      const folders = paths.filter((path) => !isAudioPath(path));
      if (audio.length > 0) {
        try {
          await commands.playerPlayQueue(
            audio.map((path) => ({
              path,
              title: nameFromPath(path),
              artist: null,
              album: null,
              duration_ms: null,
            })),
            0,
            "list_loop",
          );
          setNotice(t("shell.playingNotice", { name: nameFromPath(audio[0]) }));
        } catch (error) {
          setNotice(formatError(error, t));
        }
      }
      if (folders.length > 0) {
        try {
          await commands.libraryScanStart(folders[0]);
          setNotice(t("shell.scanNotice", { path: folders[0] }));
        } catch (error) {
          setNotice(formatError(error, t));
        }
      }
    };

    const addPathsToPlaylist = async (paths: string[], playlistId: number) => {
      try {
        const added = await commands.playlistAddPaths(playlistId, paths);
        setNotice(t("shell.addedToPlaylist", { count: added ?? 0 }));
      } catch (error) {
        setNotice(formatError(error, t));
      }
    };

    const handleArgs = (args: string[]) => {
      let playlistId: number | null = null;
      const paths: string[] = [];
      for (let index = 0; index < args.length; index += 1) {
        const arg = args[index];
        if (arg === "--playlist") {
          const value = Number(args[index + 1]);
          if (Number.isFinite(value)) {
            playlistId = value;
            index += 1;
          }
          continue;
        }
        if (arg.startsWith("--")) continue;
        paths.push(arg);
      }
      if (paths.length === 0) return;
      if (playlistId != null) {
        void addPathsToPlaylist(paths, playlistId);
      } else {
        void acceptPaths(paths);
      }
    };

    listen<{ paths: string[] }>("tauri://drag-drop", (event) => {
      setDragging(false);
      void acceptPaths(event.payload.paths ?? []);
    }).then((stop) => (disposed ? stop() : unlisteners.push(stop)));
    listen("tauri://drag-enter", () => setDragging(true)).then((stop) =>
      disposed ? stop() : unlisteners.push(stop),
    );
    listen("tauri://drag-leave", () => setDragging(false)).then((stop) =>
      disposed ? stop() : unlisteners.push(stop),
    );
    listen<string[]>("single-instance:open", (event) => {
      handleArgs(event.payload ?? []);
    }).then((stop) => (disposed ? stop() : unlisteners.push(stop)));

    commands
      .startupArgs()
      .then((args) => {
        if (!disposed) handleArgs(args);
      })
      .catch(() => {});

    return () => {
      disposed = true;
      unlisteners.forEach((stop) => stop());
    };
  }, []);

  const mode = (modes.includes(player?.mode as PlayMode) ? player?.mode : "sequential") as PlayMode;
  const playing = player?.state === "playing";
  const hasTrack = Boolean(player?.path);
  const position = player?.position_ms ?? 0;
  const duration = player?.duration_ms ?? 0;

  return (
    <div
      className={collapsed ? "shell collapsed" : "shell"}
      style={{ "--sidebar-width": collapsed ? "64px" : `${sidebarWidth}px` } as React.CSSProperties}
    >
      {crash && (
        <div className="crash-banner" role="alert">
          <span>{t("crash.desc")}</span>
          <button
            type="button"
            className="ghost"
            onClick={() => {
              void commands.exportDiagnostics().then((path) => setNotice(t("crash.exported", { path })));
            }}
          >
            {t("crash.export")}
          </button>
          <button
            type="button"
            className="ghost"
            onClick={() => {
              localStorage.setItem("biu.crash.dismissed", crash.file);
              setCrash(null);
            }}
          >
            {t("crash.dismiss")}
          </button>
        </div>
      )}
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
              <span className="nav-icon" aria-hidden>
                {item.icon}
              </span>
              <span className="nav-label">{t(item.labelKey)}</span>
            </NavLink>
          ))}
        </nav>
        <div
          className="sidebar-resizer"
          role="separator"
          aria-orientation="vertical"
          aria-label={t("shell.resizer")}
          onPointerDown={startSidebarResize}
        />
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
          <button className="ghost theme-toggle" onClick={toggleTheme} title={theme === "dark" ? t("theme.toLight") : t("theme.toDark")}>
            <span aria-hidden>{theme === "dark" ? "☀" : "☾"}</span>
            <span className="theme-label">{theme === "dark" ? t("theme.toLight") : t("theme.toDark")}</span>
          </button>
          <span className="muted">{t("app.version")}</span>
        </div>
      </aside>

      <div className="main">
        <header className="titlebar" data-tauri-drag-region>
          <div className="titlebar-left">
            <button
              className="ghost icon-button"
              onClick={toggleSidebar}
              aria-label={t("shell.toggleSidebar")}
              title={t("shell.toggleSidebar")}
            >
              ☰
            </button>
            <span className="muted">{t("shell.offline")}</span>
          </div>
          <div className="titlebar-actions">
            {frameless && (
              <div className="window-controls">
                <button
                  type="button"
                  className="window-control"
                  aria-label={t("window.minimize")}
                  onClick={() => void commands.windowMinimize()}
                >
                  –
                </button>
                <button
                  type="button"
                  className="window-control"
                  aria-label={t("window.maximize")}
                  onClick={() => void commands.windowToggleMaximize()}
                >
                  □
                </button>
                <button
                  type="button"
                  className="window-control danger"
                  aria-label={t("window.close")}
                  onClick={() => void commands.windowClose()}
                >
                  ×
                </button>
              </div>
            )}
            <form
              onSubmit={(event) => {
                event.preventDefault();
                const value = topQuery.trim();
                navigate(value ? `/search?q=${encodeURIComponent(value)}` : "/search");
              }}
            >
              <input
                className="search"
                placeholder={t("shell.search")}
                value={topQuery}
                onChange={(event) => setTopQuery(event.target.value)}
              />
            </form>
          </div>
        </header>
        <main className="content">
          <Outlet />
        </main>

        {queueOpen && (
          <section className="queue-panel" aria-label={t("player.queue")}>
            <div className="queue-head">
              <strong>{t("player.queue")}</strong>
              <button
                type="button"
                className="ghost"
                onClick={() => run(async () => {
                  setPlayer(await commands.playerQueueClear());
                  await refreshQueue();
                })}
                disabled={(queue?.items.length ?? 0) === 0}
              >
                {t("player.clear")}
              </button>
            </div>
            {queue && queue.items.length > 0 ? (
              <ol className="queue-list">
                {queue.items.map((item, index) => (
                  <li
                    key={`${item.path}-${index}`}
                    className={`${index === queue.index ? "current" : ""}${queueDrag === index ? " dragging" : ""}`}
                    draggable
                    onDragStart={() => setQueueDrag(index)}
                    onDragOver={(event) => event.preventDefault()}
                    onDragEnd={() => setQueueDrag(null)}
                    onDrop={(event) => {
                      event.preventDefault();
                      if (queueDrag != null) void moveQueueItem(queueDrag, index);
                      setQueueDrag(null);
                    }}
                  >
                    <button
                      type="button"
                      className="queue-track"
                      onClick={() => run(async () => {
                        setPlayer(await commands.playerJump(index));
                        await refreshQueue();
                      })}
                    >
                      <span>{index + 1}</span>
                      <span>
                        <strong>{item.title}</strong>
                        <small>{item.artist ?? t("player.unknownArtist")}</small>
                      </span>
                    </button>
                    <button
                      type="button"
                      className="ghost"
                      aria-label={t("player.remove")}
                      onClick={() => run(async () => {
                        await commands.playerQueueRemove(index);
                        await refreshQueue();
                      })}
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ol>
            ) : (
              <p className="muted">{t("player.queueEmpty")}</p>
            )}
          </section>
        )}

        <footer className="playerbar">
          <div className="player-meta">
            <div className="cover-placeholder">♪</div>
            <div>
              <strong>{player?.title ?? t("player.none")}</strong>
              <p className="muted">
                {player?.device_lost
                  ? t("player.deviceLost")
                  : player?.error
                    ? `${t("player.error")}: ${player.error}`
                    : player?.artist ?? (hasTrack ? t("player.unknownArtist") : t("player.hint"))}
              </p>
            </div>
          </div>

          <div className="player-controls">
            <button
              className="ghost"
              aria-label={t("player.prev")}
              onClick={() => run(async () => {
                setPlayer(await commands.playerPrev());
                await refreshQueue();
              })}
              disabled={!hasTrack}
            >
              ⏮
            </button>
            <button
              className="play"
              aria-label={playing ? t("player.pause") : t("player.play")}
              onClick={() => run(async () => setPlayer(playing ? await commands.playerPause() : await commands.playerPlay()))}
              disabled={!hasTrack}
            >
              {playing ? "⏸" : "▶"}
            </button>
            <button
              type="button"
              className="ghost"
              aria-label={t("player.stop")}
              title={t("player.stop")}
              onClick={() => run(async () => setPlayer(await commands.playerStop()))}
              disabled={!hasTrack}
            >
              ⏹
            </button>
            <button
              className="ghost"
              aria-label={t("player.next")}
              onClick={() => run(async () => {
                setPlayer(await commands.playerNext());
                await refreshQueue();
              })}
              disabled={!hasTrack}
            >
              ⏭
            </button>
          </div>

          <div className="player-progress">
            <span className="mono">{formatTime(position)}</span>
            <input
              type="range"
              aria-label="Seek"
              min={0}
              max={Math.max(duration, 1)}
              value={Math.min(position, duration || 0)}
              onChange={(event) => run(async () => {
                setPlayer(await commands.playerSeek(Number(event.target.value)));
              })}
              disabled={!hasTrack}
            />
            <span className="mono">{formatTime(duration)}</span>
          </div>

          <div className="player-extra">
            <button
              type="button"
              className="ghost mode-button"
              aria-label={t("player.mode")}
              title={t(modeKey[mode])}
              onClick={() => run(async () => {
                const next = modes[(modes.indexOf(mode) + 1) % modes.length];
                setPlayer(await commands.playerSetMode(next));
              })}
            >
              {mode === "shuffle" ? "⤨" : mode === "single_loop" ? "🔂" : mode === "list_loop" ? "🔁" : "➡"}
            </button>
            <button
              type="button"
              className="ghost"
              aria-label={player?.muted ? t("player.unmute") : t("player.mute")}
              onClick={() => run(async () => setPlayer(await commands.playerSetMuted(!player?.muted)))}
            >
              {player?.muted ? "🔇" : "🔊"}
            </button>
            <input
              type="range"
              aria-label="Volume"
              min={0}
              max={1}
              step={0.01}
              value={player?.muted ? 0 : player?.volume ?? 1}
              onChange={(event) => run(async () => setPlayer(await commands.playerSetVolume(Number(event.target.value))))}
            />
            <button
              type="button"
              className="ghost"
              aria-pressed={queueOpen}
              onClick={() => setQueueOpen((open) => !open)}
            >
              {t("player.queue")} {queue?.items.length ?? 0}
            </button>
          </div>
        </footer>
        {notice && <p className="player-notice">{notice}</p>}
      </div>
      {dragging && <div className="drop-overlay">{t("shell.dropOverlay")}</div>}
    </div>
  );
}
