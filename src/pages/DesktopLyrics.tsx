import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands, type LyricsDto_Serialize, type PlayerStateDto_Serialize } from "../bindings";
import { useI18n } from "../i18n";

export function DesktopLyrics() {
  const { t } = useI18n();
  const [player, setPlayer] = useState<PlayerStateDto_Serialize | null>(null);
  const [lyrics, setLyrics] = useState<LyricsDto_Serialize | null>(null);
  const [offset, setOffset] = useState(0);
  const [fontSize, setFontSize] = useState(32);
  const [color, setColor] = useState("#f5f7fa");

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen<PlayerStateDto_Serialize>("player:state", (event) => {
      if (!disposed) setPlayer(event.payload);
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    commands.playerSnapshot().then((value) => {
      if (!disposed) setPlayer(value);
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const loadStyle = () => {
      commands
        .settingsGet()
        .then((settings) => {
          if (disposed) return;
          setOffset(settings.lyricsOffsetMs ?? 0);
          setFontSize(settings.desktopLyricsFontSize ?? 32);
          setColor(settings.desktopLyricsColor ?? "#f5f7fa");
        })
        .catch(() => {});
    };
    loadStyle();
    listen("settings:changed", loadStyle).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const path = player?.path;
    if (!path) {
      setLyrics(null);
      return;
    }
    let disposed = false;
    commands.lyricsLoad(path).then((value) => {
      if (!disposed) setLyrics(value);
    });
    return () => {
      disposed = true;
    };
  }, [player?.path]);

  const position = player?.position_ms ?? 0;
  const activeIndex = lyrics
    ? lyrics.lines.reduce(
        (current, line, index) =>
          (line.timeMs ?? 0) <= position + 80 + offset ? index : current,
        -1,
      )
    : -1;
  const active = activeIndex >= 0 ? lyrics?.lines[activeIndex] : null;
  const next = lyrics && activeIndex + 1 < lyrics.lines.length ? lyrics.lines[activeIndex + 1] : null;

  return (
    <main className="desktop-lyrics" data-tauri-drag-region>
      <div className="desktop-lyrics-inner" data-tauri-drag-region>
        <p className="desktop-lyrics-title">
          {player?.title ?? t("desktop.defaultTitle")}
          {player?.artist ? ` · ${player.artist}` : ""}
        </p>
        <p
          className="desktop-lyrics-current"
          style={{ fontSize: `${fontSize}px`, color }}
        >
          {active?.text || (lyrics ? "♪" : t("desktop.waiting"))}
        </p>
        <p className="desktop-lyrics-next">{next?.text ?? ""}</p>
      </div>
    </main>
  );
}
