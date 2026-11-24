import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  commands,
  type LyricsDto_Serialize,
  type PlayerStateDto_Serialize,
  type QueueSnapshotDto_Serialize,
} from "../bindings";
import { useI18n, type MessageKey } from "../i18n";
import { formatError } from "../errors";

const modes = ["sequential", "list_loop", "single_loop", "shuffle"] as const;
type PlayMode = (typeof modes)[number];
const modeKey: Record<PlayMode, MessageKey> = {
  sequential: "player.mode.sequential",
  list_loop: "player.mode.listLoop",
  single_loop: "player.mode.singleLoop",
  shuffle: "player.mode.shuffle",
};

function formatTime(ms: number | null | undefined): string {
  if (!ms || ms <= 0) return "0:00";
  const total = Math.floor(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function NowPlaying() {
  const { t } = useI18n();
  const [player, setPlayer] = useState<PlayerStateDto_Serialize | null>(null);
  const [queue, setQueue] = useState<QueueSnapshotDto_Serialize | null>(null);
  const [message, setMessage] = useState("");
  const [lyrics, setLyrics] = useState<LyricsDto_Serialize | null>(null);
  const [lyricsLoading, setLyricsLoading] = useState(false);
  const [lyricsOffset, setLyricsOffset] = useState(0);
  const lyricsRef = useRef<HTMLDivElement>(null);

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
      .catch((error) => setMessage(formatError(error, t)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    commands
      .settingsGet()
      .then((settings) => setLyricsOffset(settings.lyricsOffsetMs ?? 0))
      .catch(() => setLyricsOffset(0));
  }, []);

  useEffect(() => {
    const path = player?.path;
    if (!path) {
      setLyrics(null);
      return;
    }
    let disposed = false;
    setLyricsLoading(true);
    commands
      .lyricsLoad(path)
      .then((value) => {
        if (!disposed) setLyrics(value);
      })
      .catch(() => {
        if (!disposed) setLyrics(null);
      })
      .finally(() => {
        if (!disposed) setLyricsLoading(false);
      });
    return () => {
      disposed = true;
    };
  }, [player?.path]);

  const positionForLyrics = player?.position_ms ?? 0;
  const activeLine = lyrics
    ? lyrics.lines.reduce(
        (current, line, index) => ((line.timeMs ?? 0) <= positionForLyrics + 80 ? index : current),
        -1,
      )
    : -1;

  useEffect(() => {
    const active = lyricsRef.current?.querySelector<HTMLElement>(".active");
    active?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [activeLine]);

  async function run(action: () => Promise<unknown>) {
    try {
      setMessage("");
      await action();
      setQueue(await commands.playerQueue());
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  const hasTrack = Boolean(player?.path);
  const playing = player?.state === "playing";
  const mode = (modes.includes(player?.mode as PlayMode) ? player?.mode : "sequential") as PlayMode;
  const duration = player?.duration_ms ?? 0;
  const position = Math.min(player?.position_ms ?? 0, duration || Number.MAX_SAFE_INTEGER);
  const title = player?.title ?? t("player.none");
  const artist = player?.artist ?? (hasTrack ? t("player.unknownArtist") : t("np.selectTrack"));
  const initial = title.trim().charAt(0).toUpperCase() || "♪";

  return (
    <section className="now-playing">
      <div className="np-stage">
        <div className="np-cover" aria-hidden="true">
          <span>{initial}</span>
        </div>
        <div className="np-meta">
          <p className="page-kicker">P-01 · NOW PLAYING</p>
          <h1>{title}</h1>
          <p className="np-artist">{artist}</p>
          <p className="muted">{player?.album ?? "—"}</p>

          <div className="np-progress">
            <input
              aria-label={t("np.seek")}
              type="range"
              min={0}
              max={Math.max(duration, 1)}
              value={position}
              onChange={(event) => run(() => commands.playerSeek(Number(event.target.value)))}
              disabled={!hasTrack}
            />
            <div className="np-times">
              <span>{formatTime(position)}</span>
              <span>{formatTime(duration)}</span>
            </div>
          </div>

          <div className="np-controls">
            <button className="ghost" onClick={() => run(() => commands.playerPrev())} disabled={!hasTrack}>
              {t("player.prev")}
            </button>
            <button
              className="np-play"
              onClick={() => run(() => (playing ? commands.playerPause() : commands.playerPlay()))}
              disabled={!hasTrack}
            >
              {playing ? t("player.pause") : t("player.play")}
            </button>
            <button className="ghost" onClick={() => run(() => commands.playerNext())} disabled={!hasTrack}>
              {t("player.next")}
            </button>
            <button
              className="ghost"
              onClick={() => {
                if (!player) return;
                const index = modes.indexOf(mode);
                run(() => commands.playerSetMode(modes[(index + 1) % modes.length]));
              }}
              disabled={!hasTrack}
            >
              {t(modeKey[mode])}
            </button>
            <input
              className="np-volume"
              aria-label={t("np.volume")}
              type="range"
              min={0}
              max={1}
              step={0.01}
              value={player?.volume ?? 1}
              onChange={(event) => run(() => commands.playerSetVolume(Number(event.target.value)))}
            />
          </div>

          {message && <p className="notice">{message}</p>}
          {player?.device_lost && <p className="notice">{t("player.deviceLost")}</p>}
        </div>
      </div>

      <div className="np-columns">
        <div className="np-card np-lyrics-card">
          <div className="card-head">
            <h3>{t("np.lyrics")}</h3>
            <span className="pill">
              {lyrics
                ? t("np.lyricsMeta", {
                    count: lyrics.lines.length,
                    source: lyrics.source === "embedded" ? t("np.embedded") : "LRC",
                  })
                : lyricsLoading
                  ? t("common.loading")
                  : t("np.none")}
              {lyrics && lyricsOffset !== 0 ? ` · ${lyricsOffset > 0 ? "+" : ""}${lyricsOffset}ms` : ""}
            </span>
            <button className="ghost small" onClick={() => run(() => commands.desktopLyricsToggle())}>
              {t("np.desktopLyrics")}
            </button>
          </div>
          {!hasTrack && <p className="muted">{t("np.lyricsHint")}</p>}
          {hasTrack && !lyricsLoading && !lyrics && (
            <p className="muted">{t("np.lyricsMissing")}</p>
          )}
          {lyrics && (
            <div className="np-lyrics" ref={lyricsRef}>
              {lyrics.lines.map((line, index) => (
                <button
                  key={`${line.timeMs}-${index}`}
                  className={index === activeLine ? "active" : ""}
                  onClick={() => run(() => commands.playerSeek(line.timeMs ?? 0))}
                >
                  {line.text || "♪"}
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="np-card">
          <div className="card-head">
            <h3>{t("np.queue")}</h3>
            <span className="pill">{t("np.count", { count: queue?.items.length ?? 0 })}</span>
          </div>
          <ol className="np-queue">
            {(queue?.items ?? []).slice(0, 30).map((item, index) => (
              <li
                key={`${item.path}-${index}`}
                className={queue?.index === index ? "active" : ""}
                onClick={() => run(() => commands.playerJump(index))}
              >
                <span className="muted">{index + 1}</span>
                <span>{item.title}</span>
                <span className="muted">{item.artist ?? t("player.unknownArtist")}</span>
              </li>
            ))}
            {(!queue || queue.items.length === 0) && <li className="muted">{t("np.queueEmpty")}</li>}
          </ol>
        </div>
      </div>
    </section>
  );
}
