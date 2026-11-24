import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type AlbumDto,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { AlbumCover } from "../components/AlbumCover";
import { TrackTable } from "../components/TrackTable";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function AlbumDetail() {
  const { t } = useI18n();
  const { id } = useParams();
  const albumId = Number(id);
  const [album, setAlbum] = useState<AlbumDto | null>(null);
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    const [all, rows] = await Promise.all([commands.libraryAlbums(), commands.libraryAlbumTracks(albumId)]);
    setAlbum(all.find((entry) => entry.id === albumId) ?? null);
    setTracks(rows);
  }, [albumId]);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh]);

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
      <header className="page-head album-head">
        <AlbumCover albumId={albumId} title={album?.title ?? t("common.album")} className="album-cover cover-large" />
        <div>
          <p className="breadcrumb">
            <Link to="/library/albums">{t("common.album")}</Link> / {album?.title ?? t("common.loading")}
          </p>
          <h1>{album?.title ?? t("common.album")}</h1>
          <p className="muted">
            {album?.artist ?? t("player.unknownArtist")}
            {album?.year ? ` · ${album.year}` : ""} · {tracks.length} {t("common.tracksSuffix")}
          </p>
        </div>
        <button onClick={() => play(0)} disabled={tracks.length === 0}>
          {t("album.play")}
        </button>
      </header>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
