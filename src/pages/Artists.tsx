import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type ArtistDto } from "../bindings";

export function Artists() {
  const [artists, setArtists] = useState<ArtistDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryArtists()
      .then(setArtists)
      .catch((error) => setMessage(String(error)));
  }, []);

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-05</p>
          <h1>艺术家</h1>
          <p className="muted">按艺术家浏览媒体库。</p>
        </div>
        <span className="pill">{artists.length} 位</span>
      </header>
      {message && <p className="notice">{message}</p>}
      {artists.length === 0 ? (
        <div className="empty-card">
          <strong>还没有艺术家</strong>
          <span>扫描带艺术家标签的音乐目录后，这里会自动聚合。</span>
          <Link to="/library">打开媒体库</Link>
        </div>
      ) : (
        <ul className="artist-list">
          {artists.map((artist) => (
            <li key={artist.id}>
              <Link to={`/library/artists/${artist.id}`} className="artist-row">
                <span className="artist-avatar">{artist.name.trim().charAt(0).toUpperCase() || "?"}</span>
                <span>
                  <strong>{artist.name}</strong>
                  <span className="muted">{artist.track_count ?? 0} 首</span>
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
