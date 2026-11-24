import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type AlbumDto,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { TrackTable } from "../components/TrackTable";

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const value = error as { code?: string; message?: string };
    return value.code ? `${value.code}: ${value.message}` : String(value.message);
  }
  return String(error);
}

export function AlbumDetail() {
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
    refresh().catch((error) => setMessage(errorText(error)));
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
      setMessage(errorText(error));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/library/albums">专辑</Link> / {album?.title ?? "加载中"}
          </p>
          <h1>{album?.title ?? "专辑"}</h1>
          <p className="muted">
            {album?.artist ?? "未知艺术家"}
            {album?.year ? ` · ${album.year}` : ""} · {tracks.length} 首
          </p>
        </div>
        <button onClick={() => play(0)} disabled={tracks.length === 0}>
          播放专辑
        </button>
      </header>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
