import { useCallback, useEffect, useRef, useState } from "react";
import {
  commands,
  type LibraryScanStatusDto_Serialize,
  type LibraryTrackDto_Serialize,
  type PlaylistDto,
  type QueueItemDto_Serialize,
} from "../bindings";

const PAGE_SIZE = 200;
const ROW_HEIGHT = 36;
const VIEWPORT_HEIGHT = 480;
const OVERSCAN = 6;

type TrackSort = "title" | "artist" | "album" | "duration" | "added";

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
  const [message, setMessage] = useState("");
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
    refresh().catch((error) => setMessage(errorText(error)));
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
        .catch((error) => setMessage(errorText(error)));
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
            refresh().catch((error) => setMessage(errorText(error)));
          }
        })
        .catch((error) => setMessage(errorText(error)));
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
        .catch((error) => setMessage(errorText(error)))
        .finally(() => inFlight.current.delete(offset));
    }
  }, [start, end, sort, descending, count, searching]);

  function changeSort(next: TrackSort) {
    setScrollTop(0);
    setTracks({});
    generation.current += 1;
    inFlight.current.clear();
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
    if (sort === next) {
      setDescending((value) => !value);
    } else {
      setSort(next);
      setDescending(false);
    }
  }

  async function startScan() {
    const path = folder.trim();
    if (!path) {
      setMessage("先填写一个绝对路径");
      return;
    }
    try {
      const status = await commands.libraryScanStart(path);
      setScan(status);
      setMessage("扫描中…");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function pauseScan() {
    try {
      setScan(await commands.libraryScanPause());
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function resumeScan() {
    try {
      setScan(await commands.libraryScanResume());
      setMessage("继续扫描…");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function cancelScan() {
    try {
      setScan(await commands.libraryScanCancel());
      setMessage("已取消，可稍后继续");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function playAt(index: number) {
    try {
      const offset = Math.max(0, index - 250);
      const list = await commands.libraryTracks(500, offset, sort, descending);
      if (list.length === 0) return;
      const startIndex = Math.min(index - offset, list.length - 1);
      await commands.playerPlayQueue(list.map(toQueueItem), startIndex, "list_loop");
      setMessage(`正在播放：${list[startIndex]?.title ?? ""}`);
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function playSingle(track: LibraryTrackDto_Serialize) {
    try {
      await commands.playerPlayQueue([toQueueItem(track)], 0, "list_loop");
      setMessage(`正在播放：${track.title}`);
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function addToPlaylist(track: LibraryTrackDto_Serialize) {
    if (!targetPlaylist) {
      setMessage("先创建一个播放列表");
      return;
    }
    try {
      const added = await commands.playlistAddTracks(Number(targetPlaylist), [track.id]);
      setMessage((added ?? 0) > 0 ? `已加入：${track.title}` : "该曲目已在播放列表中");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  const sortMark = (column: TrackSort) =>
    sort === column ? (descending ? " ↓" : " ↑") : "";

  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">P-02</span>
        <h2>媒体库</h2>
        <p>{count.toLocaleString()} 首曲目 · 虚拟滚动</p>
      </div>

      <div className="card">
        <div className="card-head">
          <h3>添加音乐目录</h3>
          <span className="muted">仅扫描，不移动或删除源文件</span>
        </div>
        <div className="library-add">
          <input
            aria-label="音乐目录绝对路径"
            placeholder="/Users/you/Music"
            value={folder}
            onChange={(event) => setFolder(event.target.value)}
            disabled={busy}
          />
          <button type="button" onClick={startScan} disabled={busy || scan?.state === "paused"}>
            {busy ? "扫描中…" : "添加并扫描"}
          </button>
          {scan?.state === "scanning" && (
            <button type="button" className="ghost" onClick={pauseScan}>
              暂停
            </button>
          )}
          {scan?.state === "paused" && (
            <button type="button" className="ghost" onClick={resumeScan}>
              继续
            </button>
          )}
          {(scan?.state === "scanning" || scan?.state === "paused") && (
            <button type="button" className="ghost" onClick={cancelScan}>
              取消
            </button>
          )}
        </div>
        {message && <p className="library-message">{message}</p>}
        {scan && scan.state !== "idle" && (
          <p className="library-message">
            扫描状态：{scan.state} · 发现 {scan.files_seen ?? 0} · 新增{" "}
            {scan.inserted ?? 0} · 跳过 {scan.skipped ?? 0}
            {scan.error ? ` · ${scan.error}` : ""}
          </p>
        )}
      </div>

      <div className="card">
        <div className="card-head">
          <h3>曲目</h3>
          <div className="library-search">
            <input
              aria-label="搜索曲目"
              placeholder="搜索标题、艺术家、专辑或拼音"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <button type="button" className="ghost" onClick={() => refresh()} disabled={busy}>
              刷新
            </button>
          </div>
        </div>
        <div className="library-playlist">
          <label>
            目标播放列表
            <select
              value={targetPlaylist}
              onChange={(event) => setTargetPlaylist(event.target.value)}
              disabled={playlists.length === 0}
            >
              {playlists.length === 0 ? (
                <option value="">先创建播放列表</option>
              ) : (
                playlists.map((item) => (
                  <option key={item.id} value={String(item.id)}>
                    {item.name}（{item.track_count} 首）
                  </option>
                ))
              )}
            </select>
          </label>
          <span className="muted">在曲目行点“加入”追加到所选列表</span>
        </div>

        {searching ? (
          hits.length === 0 ? (
            <p className="muted">没有匹配的曲目，试试标题、艺术家、专辑或拼音首字母。</p>
          ) : (
            <table className="track-table">
              <thead>
                <tr>
                  <th>标题</th>
                  <th>艺术家</th>
                  <th>专辑</th>
                  <th>时长</th>
                  <th>格式</th>
                  <th aria-label="操作" />
                </tr>
              </thead>
              <tbody>
                {hits.map((track) => (
                  <tr key={track.path}>
                    <td title={track.path}>{track.title}</td>
                    <td>{track.artist ?? "未知艺术家"}</td>
                    <td>{track.album ?? "未知专辑"}</td>
                    <td>{formatDuration(track.duration_ms)}</td>
                    <td className="mono">{track.format?.toUpperCase() ?? "-"}</td>
                    <td>
                      <div className="row-actions">
                        <button type="button" className="ghost" onClick={() => playSingle(track)}>
                          播放
                        </button>
                        <button
                          type="button"
                          className="ghost"
                          onClick={() => addToPlaylist(track)}
                          disabled={playlists.length === 0}
                        >
                          加入
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        ) : count === 0 ? (
          <p className="muted">还没有曲目。添加一个包含音频文件的目录开始扫描。</p>
        ) : (
          <div className="track-virtual">
            <div className="track-head">
              {(
                [
                  ["title", "标题"],
                  ["artist", "艺术家"],
                  ["album", "专辑"],
                  ["duration", "时长"],
                ] as Array<[TrackSort, string]>
              ).map(([column, label]) => (
                <button
                  key={column}
                  type="button"
                  className="track-sort"
                  onClick={() => changeSort(column)}
                >
                  {label}
                  {sortMark(column)}
                </button>
              ))}
              <span className="mono">格式</span>
              <span aria-hidden="true" />
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
                    >
                      {track ? (
                        <>
                          <span className="track-title" title={track.path}>
                            {track.title}
                          </span>
                          <span>{track.artist ?? "未知艺术家"}</span>
                          <span>{track.album ?? "未知专辑"}</span>
                          <span className="mono">{formatDuration(track.duration_ms)}</span>
                          <span className="mono">
                            {track.format?.toUpperCase() ?? "-"}
                          </span>
                          <span className="row-actions">
                            <button
                              type="button"
                              className="ghost"
                              onClick={() => playAt(index)}
                            >
                              播放
                            </button>
                            <button
                              type="button"
                              className="ghost"
                              onClick={() => addToPlaylist(track)}
                              disabled={playlists.length === 0}
                            >
                              加入
                            </button>
                          </span>
                        </>
                      ) : (
                        <span className="track-placeholder">加载中…</span>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
