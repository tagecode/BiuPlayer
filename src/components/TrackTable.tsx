import type { LibraryTrackDto_Serialize } from "../bindings";

export function formatDuration(ms: number | null | undefined): string {
  if (!ms || ms <= 0) return "--:--";
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function TrackTable({
  tracks,
  onPlay,
}: {
  tracks: LibraryTrackDto_Serialize[];
  onPlay: (index: number) => void;
}) {
  if (tracks.length === 0) {
    return <p className="muted">这里还没有曲目。</p>;
  }
  return (
    <table className="track-table">
      <thead>
        <tr>
          <th>#</th>
          <th>标题</th>
          <th>艺术家</th>
          <th>专辑</th>
          <th>时长</th>
          <th>格式</th>
        </tr>
      </thead>
      <tbody>
        {tracks.map((track, index) => (
          <tr key={track.path} onDoubleClick={() => onPlay(index)}>
            <td className="muted">{index + 1}</td>
            <td>
              <button className="link-button" onClick={() => onPlay(index)}>
                {track.title}
              </button>
            </td>
            <td>{track.artist ?? "未知艺术家"}</td>
            <td>{track.album ?? "—"}</td>
            <td>{formatDuration(track.duration_ms)}</td>
            <td className="mono">{track.format?.toUpperCase() ?? "-"}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
