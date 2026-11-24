import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type ArtistDto,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
} from "../bindings";
import { TrackTable } from "../components/TrackTable";

export function ArtistDetail() {
  const { id } = useParams();
  const artistId = Number(id);
  const [artist, setArtist] = useState<ArtistDto | null>(null);
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    const [all, rows] = await Promise.all([commands.libraryArtists(), commands.libraryArtistTracks(artistId)]);
    setArtist(all.find((entry) => entry.id === artistId) ?? null);
    setTracks(rows);
  }, [artistId]);

  useEffect(() => {
    refresh().catch((error) => setMessage(String(error)));
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
      setMessage(String(error));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <p className="breadcrumb">
            <Link to="/library/artists">艺术家</Link> / {artist?.name ?? "加载中"}
          </p>
          <h1>{artist?.name ?? "艺术家"}</h1>
          <p className="muted">{tracks.length} 首曲目</p>
        </div>
        <button onClick={() => play(0)} disabled={tracks.length === 0}>
          播放全部
        </button>
      </header>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
