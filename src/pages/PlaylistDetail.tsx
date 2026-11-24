import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type PlaylistDto,
  type PlaylistItemDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

function formatDuration(ms: number | null): string {
  if (!ms || ms <= 0) return "--:--";
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function toQueue(items: PlaylistItemDto_Serialize[]): QueueItemDto_Serialize[] {
  return items.map((item) => ({
    path: item.track.path,
    title: item.track.title,
    artist: item.track.artist ?? null,
    album: item.track.album ?? null,
    duration_ms: item.track.duration_ms ?? null,
  }));
}

export function PlaylistDetail() {
  const { t } = useI18n();
  const { id } = useParams();
  const playlistId = Number(id);
  const [playlist, setPlaylist] = useState<PlaylistDto | null>(null);
  const [items, setItems] = useState<PlaylistItemDto_Serialize[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    if (!Number.isFinite(playlistId)) return;
    const [all, rows] = await Promise.all([
      commands.playlistList(),
      commands.playlistItems(playlistId),
    ]);
    setPlaylist(all.find((entry) => entry.id === playlistId) ?? null);
    setItems(rows);
  }, [playlistId]);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh]);

  const queueItems = useMemo(() => toQueue(items), [items]);

  async function play(startIndex = 0) {
    if (queueItems.length === 0) return;
    try {
      await commands.playerPlayQueue(queueItems, startIndex, "list_loop");
      setMessage(t("playlist.playing"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function move(from: number, to: number) {
    if (to < 0 || to >= items.length) return;
    try {
      await commands.playlistMoveItem(playlistId, from, to);
      await refresh();
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function remove(position: number) {
    try {
      await commands.playlistRemoveItem(playlistId, position);
      await refresh();
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/playlists">{t("pages.playlists.title")}</Link> / {playlist?.name ?? t("common.loading")}
          </p>
          <h1>{playlist?.name ?? t("pages.playlist.title")}</h1>
          <p className="muted">{items.length} {t("common.tracksSuffix")}</p>
        </div>
        <div className="row-actions">
          <button onClick={() => play(0)} disabled={items.length === 0}>
            {t("common.play")}
          </button>
        </div>
      </header>

      {message && <p className="notice">{message}</p>}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>{t("playlist.emptyTitle")}</strong>
          <span>{t("playlist.emptyHint")}</span>
          <Link to="/library">{t("common.openLibrary")}</Link>
        </div>
      ) : (
        <table className="track-table">
          <thead>
            <tr>
              <th>#</th>
              <th>{t("common.title")}</th>
              <th>{t("common.artist")}</th>
              <th>{t("common.album")}</th>
              <th>{t("common.duration")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {items.map((item, index) => (
              <tr key={`${item.track.id}-${item.position}`} onDoubleClick={() => play(index)}>
                <td className="muted">{index + 1}</td>
                <td>
                  <button className="link-button" onClick={() => play(index)}>
                    {item.track.title}
                  </button>
                </td>
                <td>{item.track.artist ?? t("player.unknownArtist")}</td>
                <td>{item.track.album ?? "—"}</td>
                <td>{formatDuration(item.track.duration_ms)}</td>
                <td>
                  <div className="row-actions">
                    <button className="ghost" disabled={index === 0} onClick={() => move(index, index - 1)}>
                      ↑
                    </button>
                    <button className="ghost" disabled={index === items.length - 1} onClick={() => move(index, index + 1)}>
                      ↓
                    </button>
                    <button className="ghost danger" onClick={() => remove(index)}>
                      {t("common.remove")}
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
