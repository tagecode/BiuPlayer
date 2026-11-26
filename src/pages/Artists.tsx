import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type ArtistDto } from "../bindings";
import { LibraryNav } from "../components/LibraryNav";
import { ShowMore, useLoadMore } from "../components/ShowMore";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function Artists() {
  const { t } = useI18n();
  const [artists, setArtists] = useState<ArtistDto[]>([]);
  const [message, setMessage] = useState("");
  const { visible, hasMore, loadMore } = useLoadMore(artists.length);

  useEffect(() => {
    commands
      .libraryArtists()
      .then(setArtists)
      .catch((error) => setMessage(formatError(error, t)));
  }, []);

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-05</p>
          <h1>{t("pages.artists.title")}</h1>
          <p className="muted">{t("pages.artists.desc")}</p>
        </div>
        <span className="pill">{artists.length} {t("common.artistsSuffix")}</span>
      </header>
      <LibraryNav />
      {message && <p className="notice">{message}</p>}
      {artists.length === 0 ? (
        <div className="empty-card">
          <strong>{t("artists.emptyTitle")}</strong>
          <span>{t("artists.emptyHint")}</span>
          <Link to="/library">{t("common.openLibrary")}</Link>
        </div>
      ) : (
        <>
          <ul className="artist-list">
            {artists.slice(0, visible).map((artist) => (
              <li key={artist.id}>
                <Link to={`/library/artists/${artist.id}`} className="artist-row">
                  <span className="artist-avatar">{artist.name.trim().charAt(0).toUpperCase() || "?"}</span>
                  <span>
                    <strong>{artist.name}</strong>
                    <span className="muted">{artist.track_count ?? 0} {t("common.tracksSuffix")}</span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
          <ShowMore hasMore={hasMore} onClick={loadMore} />
        </>
      )}
    </section>
  );
}
