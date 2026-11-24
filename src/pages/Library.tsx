import { useCallback, useEffect, useState } from "react";
import { commands, type LibraryTrackDto_Serialize } from "../bindings";

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
  const [count, setCount] = useState(0);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");

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

  async function scan() {
    const path = folder.trim();
    if (!path) {
      setMessage("先填写一个绝对路径");
      return;
    }
    setBusy(true);
    setMessage("扫描中…");
    try {
      const report = await commands.libraryAddFolder(path);
      await refresh();
      setMessage(
        `扫描完成：发现 ${report.files_seen ?? 0}，新增 ${report.inserted ?? 0}，跳过 ${report.skipped ?? 0}，用时 ${report.elapsed_ms ?? 0}ms`,
      );
    } catch (error) {
      setMessage(errorText(error));
    } finally {
      setBusy(false);
    }
  }

  async function play(track: LibraryTrackDto_Serialize) {
    try {
      await commands.playerOpen(track.path);
      setMessage(`正在播放：${track.title}`);
    } catch (error) {
      setMessage(errorText(error));
    }
  }

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
          <button type="button" onClick={scan} disabled={busy}>
            {busy ? "扫描中…" : "添加并扫描"}
          </button>
        </div>
        {message && <p className="library-message">{message}</p>}
      </div>

      <div className="card">
        <div className="card-head">
          <h3>曲目</h3>
          <button type="button" className="ghost" onClick={() => refresh()} disabled={busy}>
            刷新
          </button>
        </div>
        {tracks.length === 0 ? (
          <p className="muted">还没有曲目。添加一个包含音频文件的目录开始扫描。</p>
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
              {tracks.map((track) => (
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
