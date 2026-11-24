import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type AlbumDto } from "../bindings";
import { AlbumCover } from "../components/AlbumCover";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function Albums() {
  const { t } = useI18n();
  const [albums, setAlbums] = useState<AlbumDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryAlbums()
      .then(setAlbums)
      .catch((error) => setMessage(formatError(error, t)));
  }, []);

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-03</p>
          <h1>{t("pages.albums.title")}</h1>
          <p className="muted">{t("pages.albums.desc")}</p>
        </div>
        <span className="pill">{albums.length} {t("common.albumsSuffix")}</span>
      </header>
      {message && <p className="notice">{message}</p>}
      {albums.length === 0 ? (
        <div className="empty-card">
          <strong>{t("albums.emptyTitle")}</strong>
          <span>{t("albums.emptyHint")}</span>
          <Link to="/library">{t("common.openLibrary")}</Link>
        </div>
      ) : (
        <div className="album-grid">
          {albums.map((album) => (
            <Link key={album.id} to={`/library/albums/${album.id}`} className="album-card">
              <AlbumCover albumId={album.id} title={album.title} />
              <strong>{album.title}</strong>
              <span className="muted">{album.artist ?? t("player.unknownArtist")}</span>
              <span className="muted">
                {album.year ? `${album.year} · ` : ""}
                {album.track_count ?? 0} {t("common.tracksSuffix")}
              </span>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}
