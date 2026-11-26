import type { LibraryTrackDto_Serialize } from "../bindings";
import { useI18n } from "../i18n";

export function formatDuration(ms: number | null | undefined): string {
  if (!ms || ms <= 0) return "--:--";
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function HighlightText({
  text,
  query,
  markAllIfMiss,
}: {
  text: string;
  query?: string;
  /** Pinyin queries never match literally; mark the whole value instead. */
  markAllIfMiss?: boolean;
}) {
  const needle = query?.trim().toLowerCase();
  if (!needle) return <>{text}</>;
  const index = text.toLowerCase().indexOf(needle);
  if (index < 0) {
    return markAllIfMiss ? <mark className="search-hit">{text}</mark> : <>{text}</>;
  }
  return (
    <>
      {text.slice(0, index)}
      <mark className="search-hit">{text.slice(index, index + needle.length)}</mark>
      {text.slice(index + needle.length)}
    </>
  );
}

/** Latin-only queries may be pinyin; CJK queries always match literally. */
export function isPinyinQuery(query: string | undefined): boolean {
  const value = query?.trim() ?? "";
  return value.length > 0 && /^[a-z0-9\s]+$/i.test(value);
}

export function TrackTable({
  tracks,
  onPlay,
  highlight,
}: {
  tracks: LibraryTrackDto_Serialize[];
  onPlay: (index: number) => void;
  highlight?: string;
}) {
  const { t } = useI18n();
  const pinyin = isPinyinQuery(highlight);
  if (tracks.length === 0) {
    return <p className="muted">{t("common.emptyTracks")}</p>;
  }
  return (
    <table className="track-table">
      <thead>
        <tr>
          <th>#</th>
          <th>{t("common.title")}</th>
          <th>{t("common.artist")}</th>
          <th>{t("common.album")}</th>
          <th>{t("common.duration")}</th>
          <th>{t("common.format")}</th>
        </tr>
      </thead>
      <tbody>
        {tracks.map((track, index) => (
          <tr key={track.path} onDoubleClick={() => onPlay(index)}>
            <td className="muted">{index + 1}</td>
            <td>
              <button className="link-button" onClick={() => onPlay(index)}>
                <HighlightText text={track.title} query={highlight} markAllIfMiss={pinyin} />
              </button>
            </td>
            <td>
              <HighlightText text={track.artist ?? t("player.unknownArtist")} query={highlight} />
            </td>
            <td>
              <HighlightText text={track.album ?? "—"} query={highlight} />
            </td>
            <td>{formatDuration(track.duration_ms)}</td>
            <td className="mono">{track.format?.toUpperCase() ?? "-"}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
