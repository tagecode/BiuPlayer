import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type PlaylistDto,
  type PlaylistItemDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const value = error as { code?: string; message?: string };
    return value.code ? `${value.code}: ${value.message}` : String(value.message);
  }
  return String(error);
}

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
    refresh().catch((error) => setMessage(errorText(error)));
  }, [refresh]);

  const queueItems = useMemo(() => toQueue(items), [items]);

  async function play(startIndex = 0) {
    if (queueItems.length === 0) return;
    try {
      await commands.playerPlayQueue(queueItems, startIndex, "list_loop");
      setMessage("正在播放播放列表");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function move(from: number, to: number) {
    if (to < 0 || to >= items.length) return;
    try {
      await commands.playlistMoveItem(playlistId, from, to);
      await refresh();
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function remove(position: number) {
    try {
      await commands.playlistRemoveItem(playlistId, position);
      await refresh();
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/playlists">播放列表</Link> / {playlist?.name ?? "加载中"}
          </p>
          <h1>{playlist?.name ?? "播放列表"}</h1>
          <p className="muted">{items.length} 首曲目</p>
        </div>
        <div className="row-actions">
          <button onClick={() => play(0)} disabled={items.length === 0}>
            播放
          </button>
        </div>
      </header>

      {message && <p className="notice">{message}</p>}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>列表还是空的</strong>
          <span>去媒体库，把曲目加入这个播放列表。</span>
          <Link to="/library">打开媒体库</Link>
        </div>
      ) : (
        <table className="track-table">
          <thead>
            <tr>
              <th>#</th>
              <th>标题</th>
              <th>艺术家</th>
              <th>专辑</th>
              <th>时长</th>
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
                <td>{item.track.artist ?? "未知艺术家"}</td>
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
                      移除
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
