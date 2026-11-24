import { useCallback, useEffect, useState } from "react";
import {
  commands,
  type LibraryScanStatusDto_Serialize,
  type LibraryTrackDto_Serialize,
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

export function Library() {
  const [folder, setFolder] = useState("");
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [hits, setHits] = useState<LibraryTrackDto_Serialize[]>([]);
  const [query, setQuery] = useState("");
  const [count, setCount] = useState(0);
  const [scan, setScan] = useState<LibraryScanStatusDto_Serialize | null>(null);
  const [message, setMessage] = useState("");
  const busy = scan?.state === "scanning" || scan?.state === "cancelling";

  const refresh = useCallback(async () => {
    const [list, total] = await Promise.all([
      commands.libraryTracks(500, 0),
      commands.libraryTrackCount(),
    ]);
    setTracks(list);
    setCount(total ?? 0);
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

  async function play(track: LibraryTrackDto_Serialize) {
    const list = query.trim() ? hits : tracks;
    if (list.length === 0) return;
    const items: QueueItemDto_Serialize[] = list.map((entry) => ({
      path: entry.path,
      title: entry.title,
      artist: entry.artist ?? null,
      album: entry.album ?? null,
      duration_ms: entry.duration_ms ?? null,
    }));
    const index = Math.max(0, list.findIndex((entry) => entry.path === track.path));
    try {
      await commands.playerPlayQueue(items, index, "list_loop");
      setMessage(`正在播放：${track.title}`);
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  const visible = query.trim() ? hits : tracks;

  return (
    <section className="page">
      <div className="page-head">
        <span className="page-kicker">P-02</span>
        <h2>媒体库</h2>
        <p>{count} 首曲目 · 当前显示前 500 首</p>
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
        {visible.length === 0 ? (
          <p className="muted">
            {query.trim()
              ? "没有匹配的曲目，试试标题、艺术家、专辑或拼音首字母。"
              : "还没有曲目。添加一个包含音频文件的目录开始扫描。"}
          </p>
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
              {visible.map((track) => (
                <tr key={track.path}>
                  <td title={track.path}>{track.title}</td>
                  <td>{track.artist ?? "未知艺术家"}</td>
                  <td>{track.album ?? "未知专辑"}</td>
                  <td>{formatDuration(track.duration_ms)}</td>
                  <td className="mono">{track.format?.toUpperCase() ?? "-"}</td>
                  <td>
                    <button type="button" className="ghost" onClick={() => play(track)}>
                      播放
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </section>
  );
}
