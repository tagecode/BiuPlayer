import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type AlbumDto,
  type ArtistDto,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { TrackTable } from "../components/TrackTable";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function ArtistDetail() {
  const { t } = useI18n();
  const { id } = useParams();
  const artistId = Number(id);
  const [artist, setArtist] = useState<ArtistDto | null>(null);
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [albums, setAlbums] = useState<AlbumDto[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    const [all, rows, allAlbums] = await Promise.all([
      commands.libraryArtists(),
      commands.libraryArtistTracks(artistId),
      commands.libraryAlbums(),
    ]);
    const current = all.find((entry) => entry.id === artistId) ?? null;
    setArtist(current);
    setTracks(rows);
    setAlbums(current ? allAlbums.filter((album) => album.artist === current.name) : []);
  }, [artistId]);

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
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/library/artists">{t("pages.artists.title")}</Link> / {artist?.name ?? t("common.loading")}
          </p>
          <h1>{artist?.name ?? t("pages.artists.title")}</h1>
          <p className="muted">{tracks.length} {t("common.tracksSuffix")}</p>
        </div>
        <button onClick={() => play(0)} disabled={tracks.length === 0}>
          {t("common.playAll")}
        </button>
      </header>
      {message && <p className="notice">{message}</p>}
      {albums.length > 0 && (
        <>
          <h2 className="section-title">{t("nav.albums")}</h2>
          <div className="tile-grid">
            {albums.map((album) => (
              <Link key={album.id} to={`/library/albums/${album.id}`} className="tile">
                <strong>{album.title}</strong>
                <span className="muted">{album.year ?? "—"} · {album.track_count ?? 0} {t("common.tracksSuffix")}</span>
              </Link>
            ))}
          </div>
        </>
      )}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
