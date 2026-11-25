import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  commands,
  type LibraryScanStatusDto_Serialize,
  type LibraryTrackDto_Serialize,
  type PlaylistDto,
  type QueueItemDto_Serialize,
} from "../bindings";
import { LibraryNav } from "../components/LibraryNav";
import { useI18n } from "../i18n";
import { formatError } from "../errors";
import { HighlightText } from "../components/TrackTable";

const PAGE_SIZE = 200;
const ROW_HEIGHT = 36;
const VIEWPORT_HEIGHT = 480;
const OVERSCAN = 6;

type TrackSort = "title" | "artist" | "album" | "duration" | "added" | "manual";

function formatDuration(ms: number | null): string {
  if (!ms || ms <= 0) return "--:--";
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function toQueueItem(track: LibraryTrackDto_Serialize): QueueItemDto_Serialize {
  return {
    path: track.path,
    title: track.title,
    artist: track.artist ?? null,
    album: track.album ?? null,
    duration_ms: track.duration_ms ?? null,
  };
}

export function Library() {
  const { t } = useI18n();
  const [folder, setFolder] = useState("");
  const [tracks, setTracks] = useState<Record<number, LibraryTrackDto_Serialize>>({});
  const [hits, setHits] = useState<LibraryTrackDto_Serialize[]>([]);
  const [query, setQuery] = useState("");
  const [count, setCount] = useState(0);
  const [sort, setSort] = useState<TrackSort>("title");
  const [descending, setDescending] = useState(false);
  const [scrollTop, setScrollTop] = useState(0);
  const [playlists, setPlaylists] = useState<PlaylistDto[]>([]);
  const [targetPlaylist, setTargetPlaylist] = useState("");
  const [scan, setScan] = useState<LibraryScanStatusDto_Serialize | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const [message, setMessage] = useState("");
  const [menu, setMenu] = useState<{ x: number; y: number; track: LibraryTrackDto_Serialize } | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inFlight = useRef(new Set<number>());
  const generation = useRef(0);
  const busy = scan?.state === "scanning" || scan?.state === "cancelling";
  const searching = query.trim().length > 0;

  const refresh = useCallback(async () => {
    const [total, lists] = await Promise.all([
      commands.libraryTrackCount(),
      commands.playlistList(),
    ]);
    setCount(total ?? 0);
    setTracks({});
    generation.current += 1;
    inFlight.current.clear();
    setPlaylists(lists);
    setTargetPlaylist((current) => current || String(lists[0]?.id ?? ""));
  }, []);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh]);

  useEffect(() => {
    const value = query.trim();
    if (!value) {
      setHits([]);
      return;
    }
    const timer = window.setTimeout(() => {
      commands
        .librarySearch(value, 50)
        .then(setHits)
        .catch((error) => setMessage(formatError(error, t)));
    }, 200);
    return () => window.clearTimeout(timer);
  }, [query]);

  useEffect(() => {
    if (scan?.state !== "scanning") return;
    const timer = window.setInterval(() => {
      commands
        .libraryScanStatus()
        .then((status) => {
          setScan(status);
          if (status.state === "completed") {
            refresh().catch((error) => setMessage(formatError(error, t)));
          }
        })
        .catch((error) => setMessage(formatError(error, t)));
    }, 500);
    return () => window.clearInterval(timer);
  }, [scan?.state, refresh]);

  const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const end = Math.min(
    count,
    start + Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT) + OVERSCAN * 2,
  );

  useEffect(() => {
    if (searching || count === 0) return;
    const firstPage = Math.floor(start / PAGE_SIZE) * PAGE_SIZE;
    const requestGeneration = generation.current;
    for (let offset = firstPage; offset < end; offset += PAGE_SIZE) {
      if (inFlight.current.has(offset)) continue;
      inFlight.current.add(offset);
      commands
        .libraryTracks(PAGE_SIZE, offset, sort, descending)
        .then((list) => {
          if (requestGeneration !== generation.current) return;
          setTracks((previous) => {
            const next = { ...previous };
            list.forEach((track, index) => {
              next[offset + index] = track;
            });
            return next;
          });
        })
        .catch((error) => setMessage(formatError(error, t)))
        .finally(() => inFlight.current.delete(offset));
    }
  }, [start, end, sort, descending, count, searching, reloadKey]);

  function changeSort(next: TrackSort) {
    setScrollTop(0);
    setTracks({});
    generation.current += 1;
    inFlight.current.clear();
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
    if (next === "manual") {
      setSort("manual");
      setDescending(false);
      return;
    }
    if (sort === next) {
      setDescending((value) => !value);
    } else {
      setSort(next);
      setDescending(false);
    }
  }

  async function moveTrack(index: number, delta: number) {
    const track = tracks[index];
    const target = tracks[index + delta];
    if (!track?.id || !target?.id) return;
    const before = delta < 0 ? tracks[index - 2] : target;
    const after = delta < 0 ? target : tracks[index + 2];
    if (delta < 0 && index - 2 >= 0 && !before?.id) return;
    if (delta > 0 && index + 2 < count && !after?.id) return;
    try {
      await commands.libraryMoveTrack(track.id, before?.id ?? null, after?.id ?? null);
      setTracks({});
      generation.current += 1;
      inFlight.current.clear();
      setReloadKey((value) => value + 1);
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function startScan(pathOverride?: string) {
    const path = (pathOverride ?? folder).trim();
    if (!path) {
      setMessage(t("library.pathRequired"));
      return;
    }
    try {
      const status = await commands.libraryScanStart(path);
      setScan(status);
      setMessage(t("library.scanning"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function pickFolder() {
    try {
      const picked = await open({ directory: true, multiple: false, title: t("library.addHeading") });
      if (typeof picked === "string" && picked.length > 0) {
        setFolder(picked);
        await startScan(picked);
      }
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function pauseScan() {
    try {
      setScan(await commands.libraryScanPause());
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function resumeScan() {
    try {
      setScan(await commands.libraryScanResume());
      setMessage(t("library.resuming"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function cancelScan() {
    try {
      setScan(await commands.libraryScanCancel());
      setMessage(t("library.cancelled"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function playAt(index: number) {
    try {
      const offset = Math.max(0, index - 250);
      const list = await commands.libraryTracks(500, offset, sort, descending);
      if (list.length === 0) return;
      const startIndex = Math.min(index - offset, list.length - 1);
      await commands.playerPlayQueue(list.map(toQueueItem), startIndex, "list_loop");
      setMessage(t("library.nowPlaying", { title: list[startIndex]?.title ?? "" }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function playSingle(track: LibraryTrackDto_Serialize) {
    try {
      await commands.playerPlayQueue([toQueueItem(track)], 0, "list_loop");
      setMessage(t("library.nowPlaying", { title: track.title }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function addTrackTo(track: LibraryTrackDto_Serialize, playlistId: number) {
    try {
      const added = await commands.playlistAddTracks(playlistId, [track.id]);
      setMessage((added ?? 0) > 0 ? t("library.added", { title: track.title }) : t("library.alreadyAdded"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function addToPlaylist(track: LibraryTrackDto_Serialize) {
    if (!targetPlaylist) {
      setMessage(t("library.noPlaylist"));
      return;
    }
    await addTrackTo(track, Number(targetPlaylist));
  }

  async function createPlaylistWithTrack(track: LibraryTrackDto_Serialize) {
    const name = window.prompt(t("playlists.newPlaceholder"))?.trim();
    if (!name) return;
    try {
      const id = await commands.playlistCreate(name);
      if (id != null) {
        await addTrackTo(track, id);
        setPlaylists(await commands.playlistList());
      }
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  useEffect(() => {
    if (!menu) return;
    const close = () => setMenu(null);
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") close();
    };
    window.addEventListener("click", close);
    window.addEventListener("contextmenu", close);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("contextmenu", close);
      window.removeEventListener("keydown", onKey);
    };
  }, [menu]);

  const sortMark = (column: TrackSort) =>
    sort === column ? (descending ? " ↓" : " ↑") : "";

  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">P-02</span>
        <h2>{t("nav.library")}</h2>
        <p>{t("library.count", { count: count.toLocaleString() })}</p>
      </div>
      <LibraryNav />

      <div className="card">
        <div className="card-head">
          <h3>{t("library.addHeading")}</h3>
          <span className="muted">{t("library.addHint")}</span>
        </div>
        <div className="library-add">
          <input
            aria-label={t("library.addPlaceholder")}
            placeholder="/Users/you/Music"
            value={folder}
            onChange={(event) => setFolder(event.target.value)}
            disabled={busy}
          />
          <button type="button" className="ghost" onClick={pickFolder} disabled={busy}>
            {t("library.pickFolder")}
          </button>
          <button type="button" onClick={() => void startScan()} disabled={busy || scan?.state === "paused"}>
            {busy ? t("library.scanning") : t("library.addScan")}
          </button>
          {scan?.state === "scanning" && (
            <button type="button" className="ghost" onClick={pauseScan}>
              {t("library.pause")}
            </button>
          )}
          {scan?.state === "paused" && (
            <button type="button" className="ghost" onClick={resumeScan}>
              {t("library.resume")}
            </button>
          )}
          {(scan?.state === "scanning" || scan?.state === "paused") && (
            <button type="button" className="ghost" onClick={cancelScan}>
              {t("common.cancel")}
            </button>
          )}
        </div>
        {message && <p className="library-message">{message}</p>}
        {scan && scan.state !== "idle" && (
          <p className="library-message">
            {t("library.scanStatus", {
              state: scan.state,
              seen: scan.files_seen ?? 0,
              inserted: scan.inserted ?? 0,
              skipped: scan.skipped ?? 0,
            })}
            {scan.error ? ` · ${scan.error}` : ""}
          </p>
        )}
      </div>

      <div className="card">
        <div className="card-head">
          <h3>{t("library.tracksHeading")}</h3>
          <div className="library-search">
            <input
              aria-label={t("library.searchPlaceholder")}
              placeholder={t("library.searchPlaceholder")}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <button type="button" className="ghost" onClick={() => refresh()} disabled={busy}>
              {t("common.refresh")}
            </button>
          </div>
        </div>
        <div className="library-playlist">
          <label>
            {t("library.targetPlaylist")}
            <select
              value={targetPlaylist}
              onChange={(event) => setTargetPlaylist(event.target.value)}
              disabled={playlists.length === 0}
            >
              {playlists.length === 0 ? (
                <option value="">{t("library.createFirst")}</option>
              ) : (
                playlists.map((item) => (
                  <option key={item.id} value={String(item.id)}>
                    {t("library.playlistOption", { name: item.name, count: item.track_count ?? 0 })}
                  </option>
                ))
              )}
            </select>
          </label>
          <span className="muted">{t("library.addHint2")}</span>
        </div>

        {searching ? (
          hits.length === 0 ? (
            <p className="muted">{t("library.noResults")}</p>
          ) : (
            <table className="track-table">
              <thead>
                <tr>
                  <th>{t("common.title")}</th>
                  <th>{t("common.artist")}</th>
                  <th>{t("common.album")}</th>
                  <th>{t("common.duration")}</th>
                  <th>{t("common.format")}</th>
                  <th aria-label={t("common.actions")} />
                </tr>
              </thead>
              <tbody>
                {hits.map((track) => (
                  <tr key={track.path}>
                    <td title={track.path}>
                      <HighlightText text={track.title} query={query} />
                    </td>
                    <td>
                      <HighlightText text={track.artist ?? t("player.unknownArtist")} query={query} />
                    </td>
                    <td>
                      <HighlightText text={track.album ?? t("common.unknownAlbum")} query={query} />
                    </td>
                    <td>{formatDuration(track.duration_ms)}</td>
                    <td className="mono">{track.format?.toUpperCase() ?? "-"}</td>
                    <td>
                      <div className="row-actions">
                        <button type="button" className="ghost" onClick={() => playSingle(track)}>
                          {t("common.play")}
                        </button>
                        <button
                          type="button"
                          className="ghost"
                          onClick={() => addToPlaylist(track)}
                          disabled={playlists.length === 0}
                        >
                          {t("common.add")}
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        ) : count === 0 ? (
          <p className="muted">{t("library.empty")}</p>
        ) : (
          <div className="track-virtual">
            <div className="track-head">
              {(
                [
                  ["title", t("common.title")],
                  ["artist", t("common.artist")],
                  ["album", t("common.album")],
                  ["added", t("common.addedAt")],
                  ["duration", t("common.duration")],
                  ["path", t("common.path")],
                  ["manual", t("library.manualSort")],
                ] as Array<[TrackSort, string]>
              ).map(([column, label]) => (
                <button
                  key={column}
                  type="button"
                  className={`track-sort sort-${column}`}
                  onClick={() => changeSort(column)}
                >
                  {label}
                  {sortMark(column)}
                </button>
              ))}
              <span className="mono track-format-head">{t("common.format")}</span>
            </div>
            <div
              className="track-scroll"
              ref={scrollRef}
              style={{ height: VIEWPORT_HEIGHT }}
              onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
            >
              <div
                className="track-spacer"
                style={{ height: count * ROW_HEIGHT }}
              >
                {Array.from({ length: end - start }, (_, offset) => {
                  const index = start + offset;
                  const track = tracks[index];
                  return (
                    <div
                      key={index}
                      className="track-row"
                      style={{ top: index * ROW_HEIGHT, height: ROW_HEIGHT }}
                      onDoubleClick={() => playAt(index)}
                      onContextMenu={(event) => {
                        if (!track) return;
                        event.preventDefault();
                        event.stopPropagation();
                        setMenu({
                          x: Math.min(event.clientX, window.innerWidth - 240),
                          y: Math.min(event.clientY, window.innerHeight - 280),
                          track,
                        });
                      }}
                    >
                      {track ? (
                        <>
                          <span className="track-title" title={track.path}>
                            <HighlightText text={track.title} query={query} />
                          </span>
                          <span>
                            <HighlightText text={track.artist ?? t("player.unknownArtist")} query={query} />
                          </span>
                          <span>
                            <HighlightText text={track.album ?? t("common.unknownAlbum")} query={query} />
                          </span>
                          <span className="mono track-added">
                            {track.added_at ? track.added_at.slice(0, 10) : "-"}
                          </span>
                          <span className="mono">{formatDuration(track.duration_ms)}</span>
                          <span className="mono">
                            {track.format?.toUpperCase() ?? "-"}
                          </span>
                          <span className="mono track-path" title={track.path}>
                            {track.path}
                          </span>
                          <span className="row-actions">
                            {sort === "manual" && (
                              <>
                                <button
                                  type="button"
                                  className="ghost"
                                  disabled={index === 0}
                                  aria-label={t("common.moveUp")}
                                  onClick={() => moveTrack(index, -1)}
                                >
                                  ↑
                                </button>
                                <button
                                  type="button"
                                  className="ghost"
                                  disabled={index >= count - 1}
                                  aria-label={t("common.moveDown")}
                                  onClick={() => moveTrack(index, 1)}
                                >
                                  ↓
                                </button>
                              </>
                            )}
                            <button
                              type="button"
                              className="ghost"
                              onClick={() => playAt(index)}
                            >
                              {t("common.play")}
                            </button>
                            <button
                              type="button"
                              className="ghost"
                              onClick={() => addToPlaylist(track)}
                              disabled={playlists.length === 0}
                            >
                              {t("common.add")}
                            </button>
                          </span>
                        </>
                      ) : (
                        <span className="track-placeholder">{t("common.loading")}…</span>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>
      {menu && (
        <div
          className="context-menu"
          style={{ left: menu.x, top: menu.y }}
          onClick={(event) => event.stopPropagation()}
        >
          <p className="context-title">{t("library.addToPlaylist")}</p>
          <p className="muted context-track" title={menu.track.title}>{menu.track.title}</p>
          {playlists.map((item) => (
            <button
              key={item.id}
              type="button"
              className="context-item"
              onClick={() => {
                if (item.id != null) void addTrackTo(menu.track, item.id);
                setMenu(null);
              }}
            >
              <span>{item.name}</span>
              <span className="muted">{item.track_count ?? 0}</span>
            </button>
          ))}
          <button
            type="button"
            className="context-item"
            onClick={() => {
              void createPlaylistWithTrack(menu.track);
              setMenu(null);
            }}
          >
            + {t("library.newPlaylist")}
          </button>
        </div>
      )}
    </section>
  );
}
