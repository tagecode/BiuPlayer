import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  commands,
  type PlayerStateDto_Serialize,
  type QueueSnapshotDto_Serialize,
} from "../bindings";
import { useI18n, type MessageKey } from "../i18n";

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

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const value = error as { code?: string; message?: string };
    return value.code ? `${value.code}: ${value.message}` : String(value.message);
  }
  return String(error);
}

export function NowPlaying() {
  const { t } = useI18n();
  const [player, setPlayer] = useState<PlayerStateDto_Serialize | null>(null);
  const [queue, setQueue] = useState<QueueSnapshotDto_Serialize | null>(null);
  const [message, setMessage] = useState("");

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
      .catch((error) => setMessage(errorText(error)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  async function run(action: () => Promise<unknown>) {
    try {
      setMessage("");
      await action();
      setQueue(await commands.playerQueue());
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  const hasTrack = Boolean(player?.path);
  const playing = player?.state === "playing";
  const mode = (modes.includes(player?.mode as PlayMode) ? player?.mode : "sequential") as PlayMode;
  const duration = player?.duration_ms ?? 0;
  const position = Math.min(player?.position_ms ?? 0, duration || Number.MAX_SAFE_INTEGER);
  const title = player?.title ?? "未在播放";
  const artist = player?.artist ?? (hasTrack ? "未知艺术家" : "从媒体库选择一首歌");
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
              aria-label="播放进度"
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
              上一首
            </button>
            <button
              className="np-play"
              onClick={() => run(() => (playing ? commands.playerPause() : commands.playerPlay()))}
              disabled={!hasTrack}
            >
              {playing ? t("player.pause") : t("player.play")}
            </button>
            <button className="ghost" onClick={() => run(() => commands.playerNext())} disabled={!hasTrack}>
              下一首
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
              aria-label="音量"
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
        <div className="np-card">
          <div className="card-head">
            <h3>歌词</h3>
            <span className="pill">即将支持</span>
          </div>
          <p className="muted">歌词窗口与 LRC 解析将在后续版本接入。</p>
        </div>
        <div className="np-card">
          <div className="card-head">
            <h3>队列</h3>
            <span className="pill">{queue?.items.length ?? 0} 首</span>
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
                <span className="muted">{item.artist ?? "未知艺术家"}</span>
              </li>
            ))}
            {(!queue || queue.items.length === 0) && <li className="muted">队列为空</li>}
          </ol>
        </div>
      </div>
    </section>
  );
}
