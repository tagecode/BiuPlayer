import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
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

const ROW_HEIGHT = 38;
const VIEWPORT_HEIGHT = 520;
const OVERSCAN = 8;

function toQueueItem(item: PlaylistItemDto_Serialize): QueueItemDto_Serialize {
  return {
    path: item.track.path,
    title: item.track.title,
    artist: item.track.artist ?? null,
    album: item.track.album ?? null,
    duration_ms: item.track.duration_ms ?? null,
  };
}

function toQueue(items: PlaylistItemDto_Serialize[]): QueueItemDto_Serialize[] {
  return items.map(toQueueItem);
}

export function PlaylistDetail() {
  const { t } = useI18n();
  const { id } = useParams();
  const navigate = useNavigate();
  const playlistId = Number(id);
  const [playlist, setPlaylist] = useState<PlaylistDto | null>(null);
  const [items, setItems] = useState<PlaylistItemDto_Serialize[]>([]);
  const [message, setMessage] = useState("");
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [history, setHistory] = useState<PlaylistItemDto_Serialize[][]>([]);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dragOver, setDragOver] = useState<number | null>(null);
  const [scrollTop, setScrollTop] = useState(0);

  const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const end = Math.min(items.length, start + Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT) + OVERSCAN * 2);
  const visible = items.slice(start, end);

  const refresh = useCallback(async () => {
    if (!Number.isFinite(playlistId)) return;
    const [all, rows] = await Promise.all([
      commands.playlistList(),
      commands.playlistItems(playlistId),
    ]);
    setPlaylist(all.find((entry) => entry.id === playlistId) ?? null);
    setItems(rows);
    setSelected(new Set());
  }, [playlistId]);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh, t]);

  useEffect(() => {
    setScrollTop(0);
  }, [playlistId]);

  const queueItems = useMemo(() => toQueue(items), [items]);

  function remember(snapshot: PlaylistItemDto_Serialize[]) {
    setHistory((current) => [...current.slice(-19), snapshot]);
  }

  async function play(startIndex = 0) {
    if (queueItems.length === 0) return;
    try {
      await commands.playerPlayQueue(queueItems, startIndex, "list_loop");
      setMessage(t("playlist.playing"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function reorder(next: PlaylistItemDto_Serialize[]) {
    const previous = items;
    setItems(next);
    remember(previous);
    try {
      await commands.playlistReorder(
        playlistId,
        next.map((item) => item.track.id ?? 0),
      );
      await refresh();
    } catch (error) {
      setItems(previous);
      setMessage(formatError(error, t));
    }
  }

  async function move(from: number, to: number) {
    if (to < 0 || to >= items.length) return;
    const next = [...items];
    const [value] = next.splice(from, 1);
    next.splice(to, 0, value);
    await reorder(next);
  }

  async function removeSelected() {
    if (selected.size === 0) return;
    const positions = [...selected].sort((a, b) => a - b);
    remember(items);
    try {
      await commands.playlistRemoveItems(playlistId, positions);
      await refresh();
      setMessage(t("playlist.removed", { count: positions.length }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function removeOne(index: number) {
    remember(items);
    try {
      await commands.playlistRemoveItems(playlistId, [index]);
      await refresh();
      setMessage(t("playlist.removed", { count: 1 }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function undo() {
    const snapshot = history[history.length - 1];
    if (!snapshot) return;
    setHistory((current) => current.slice(0, -1));
    try {
      const ids = snapshot.map((item) => item.track.id ?? 0).filter((value) => value > 0);
      await commands.playlistAddTracks(playlistId, ids);
      await commands.playlistReorder(playlistId, ids);
      await refresh();
      setMessage(t("playlist.restored"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function copy() {
    if (!playlist) return;
    const name = window.prompt(t("playlist.copyPrompt"), `${playlist.name} copy`)?.trim();
    if (!name) return;
    try {
      const newId = await commands.playlistCopy(playlistId, name);
      if (newId != null) navigate(`/playlists/${newId}`);
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function playNext(item: PlaylistItemDto_Serialize) {
    try {
      await commands.playerQueuePlayNext(toQueueItem(item));
      setMessage(t("playlist.playNextQueued", { title: item.track.title }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  function toggle(index: number) {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(index)) next.delete(index);
      else next.add(index);
      return next;
    });
  }

  function toggleAll() {
    setSelected((current) =>
      current.size === items.length ? new Set() : new Set(items.map((_, index) => index)),
    );
  }

  async function dropAt(index: number) {
    if (dragIndex == null || dragIndex === index) return;
    const next = [...items];
    const [value] = next.splice(dragIndex, 1);
    next.splice(index, 0, value);
    setDragIndex(null);
    setDragOver(null);
    await reorder(next);
  }

  const allSelected = items.length > 0 && selected.size === items.length;

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
          <button className="ghost" onClick={copy}>
            {t("playlist.copy")}
          </button>
          <button onClick={() => play(0)} disabled={items.length === 0}>
            {t("common.play")}
          </button>
        </div>
      </header>

      {message && <p className="notice">{message}</p>}

      {items.length > 0 && (
        <div className="playlist-toolbar">
          <label className="checkbox-row">
            <input type="checkbox" checked={allSelected} onChange={toggleAll} />
            {t("playlist.selectAll")}
          </label>
          <span className="muted">{t("playlist.selected", { count: selected.size })}</span>
          <button className="ghost" onClick={undo} disabled={history.length === 0}>
            {t("playlist.undo")}
          </button>
          <button className="ghost danger" onClick={removeSelected} disabled={selected.size === 0}>
            {t("playlist.removeSelected")}
          </button>
          <span className="muted">{t("playlist.dragHint")}</span>
        </div>
      )}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>{t("playlist.emptyTitle")}</strong>
          <span>{t("playlist.emptyHint")}</span>
          <Link to="/library">{t("common.openLibrary")}</Link>
        </div>
      ) : (
        <div
          className="playlist-scroll"
          style={{ height: Math.min(VIEWPORT_HEIGHT, items.length * ROW_HEIGHT + 44) }}
          onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
        >
        <table className="track-table playlist-table">
          <thead>
            <tr>
              <th className="checkbox-cell">
                <input type="checkbox" checked={allSelected} onChange={toggleAll} />
              </th>
              <th>#</th>
              <th>{t("common.title")}</th>
              <th>{t("common.artist")}</th>
              <th>{t("common.album")}</th>
              <th>{t("common.duration")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {start > 0 && (
              <tr className="spacer-row" aria-hidden="true">
                <td colSpan={7} style={{ height: start * ROW_HEIGHT }} />
              </tr>
            )}
            {visible.map((item, offset) => {
              const index = start + offset;
              return (
              <tr
                key={`${item.track.id}-${index}`}
                className={`${dragOver === index ? "drop-target" : ""} ${selected.has(index) ? "selected-row" : ""}`}
                draggable
                onDragStart={() => setDragIndex(index)}
                onDragOver={(event) => {
                  event.preventDefault();
                  setDragOver(index);
                }}
                onDragLeave={() => setDragOver((current) => (current === index ? null : current))}
                onDrop={(event) => {
                  event.preventDefault();
                  void dropAt(index);
                }}
                onDragEnd={() => {
                  setDragIndex(null);
                  setDragOver(null);
                }}
                onDoubleClick={() => play(index)}
              >
                <td className="checkbox-cell">
                  <input
                    type="checkbox"
                    checked={selected.has(index)}
                    onChange={() => toggle(index)}
                    onClick={(event) => event.stopPropagation()}
                  />
                </td>
                <td className="muted drag-cell" title={t("playlist.dragHint")}>⋮⋮</td>
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
                    <button className="ghost" onClick={() => playNext(item)}>
                      {t("playlist.playNext")}
                    </button>
                    <button className="ghost" disabled={index === 0} onClick={() => move(index, index - 1)}>
                      ↑
                    </button>
                    <button className="ghost" disabled={index === items.length - 1} onClick={() => move(index, index + 1)}>
                      ↓
                    </button>
                    <button className="ghost danger" onClick={() => void removeOne(index)}>
                      {t("common.remove")}
                    </button>
                  </div>
                </td>
              </tr>
              );
            })}
            {end < items.length && (
              <tr className="spacer-row" aria-hidden="true">
                <td colSpan={7} style={{ height: (items.length - end) * ROW_HEIGHT }} />
              </tr>
            )}
          </tbody>
        </table>
        </div>
      )}
    </section>
  );
}
