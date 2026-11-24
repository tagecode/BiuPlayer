import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type AlbumDto } from "../bindings";
import { AlbumCover } from "../components/AlbumCover";

export function Albums() {
  const [albums, setAlbums] = useState<AlbumDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryAlbums()
      .then(setAlbums)
      .catch((error) => setMessage(String(error)));
  }, []);

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-03</p>
          <h1>专辑</h1>
          <p className="muted">按专辑浏览媒体库。</p>
        </div>
        <span className="pill">{albums.length} 张</span>
      </header>
      {message && <p className="notice">{message}</p>}
      {albums.length === 0 ? (
        <div className="empty-card">
          <strong>还没有专辑</strong>
          <span>扫描带专辑标签的音乐目录后，这里会自动聚合。</span>
          <Link to="/library">打开媒体库</Link>
        </div>
      ) : (
        <div className="album-grid">
          {albums.map((album) => (
            <Link key={album.id} to={`/library/albums/${album.id}`} className="album-card">
              <AlbumCover albumId={album.id} title={album.title} />
              <strong>{album.title}</strong>
              <span className="muted">{album.artist ?? "未知艺术家"}</span>
              <span className="muted">
                {album.year ? `${album.year} · ` : ""}
                {album.track_count ?? 0} 首
              </span>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}
