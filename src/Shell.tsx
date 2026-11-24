import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  commands,
  type PlayerStateDto_Serialize,
  type QueueSnapshotDto_Serialize,
} from "./bindings";
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
  const saved = localStorage.getItem("biu.theme");
  return saved === "light" ? "light" : "dark";
}

function formatTime(ms: number | null | undefined): string {
  if (!ms || ms <= 0) return "0:00";
  const total = Math.floor(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const value = error as { code?: string; message?: string };
    return value.code ? `${value.code}: ${value.message}` : String(value.message);
  }
  return String(error);
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
  const [player, setPlayer] = useState<PlayerStateDto_Serialize | null>(null);
  const [queue, setQueue] = useState<QueueSnapshotDto_Serialize | null>(null);
  const [queueOpen, setQueueOpen] = useState(false);
  const [notice, setNotice] = useState("");
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("biu.theme", theme);
  }, [theme]);

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
      .catch((error) => setNotice(errorText(error)));
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
      setNotice(errorText(error));
    }
  }

  async function refreshQueue() {
    setQueue(await commands.playerQueue());
  }

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    const acceptPaths = async (paths: string[]) => {
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
          setNotice(`正在播放：${nameFromPath(audio[0])}`);
        } catch (error) {
          setNotice(errorText(error));
        }
      }
      if (folders.length > 0) {
        try {
          await commands.libraryScanStart(folders[0]);
          setNotice(`已开始扫描：${folders[0]}`);
        } catch (error) {
          setNotice(errorText(error));
        }
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
      void acceptPaths(event.payload ?? []);
    }).then((stop) => (disposed ? stop() : unlisteners.push(stop)));

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
                  <li key={`${item.path}-${index}`} className={index === queue.index ? "current" : ""}>
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
      {dragging && <div className="drop-overlay">释放以播放音频，或扫描目录</div>}
    </div>
  );
}
