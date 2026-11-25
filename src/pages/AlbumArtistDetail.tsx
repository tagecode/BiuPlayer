import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { TrackTable } from "../components/TrackTable";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function AlbumArtistDetail() {
  const { t } = useI18n();
  const { name } = useParams();
  const albumArtist = name ? decodeURIComponent(name) : "";
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    const rows = await commands.libraryAlbumArtistTracks(albumArtist);
    setTracks(rows);
  }, [albumArtist]);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh, t]);

  async function play(index: number) {
    const items: QueueItemDto_Serialize[] = tracks.map((track) => ({
      path: track.path,
      title: track.title,
      artist: track.artist ?? null,
      album: track.album ?? null,
      duration_ms: track.duration_ms ?? null,
    }));
    try {
      await commands.playerPlayQueue(items, index, "list_loop");
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/library/album-artists">{t("nav.albumArtists")}</Link> /{" "}
            {albumArtist || t("common.loading")}
          </p>
          <h1>{albumArtist || t("nav.albumArtists")}</h1>
          <p className="muted">
            {tracks.length} {t("common.tracksSuffix")}
          </p>
        </div>
        <button onClick={() => play(0)} disabled={tracks.length === 0}>
          {t("common.playAll")}
        </button>
      </header>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
