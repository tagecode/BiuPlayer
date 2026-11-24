import { useEffect, useState } from "react";
import { commands } from "../bindings";

const cache = new Map<number, string | null>();

export function AlbumCover({
  albumId,
  title,
  className = "album-cover",
}: {
  albumId: number | null;
  title: string;
  className?: string;
}) {
  const [src, setSrc] = useState<string | null | undefined>(
    albumId == null ? null : cache.get(albumId),
  );

  useEffect(() => {
    if (albumId == null) {
      setSrc(null);
      return;
    }
    if (cache.has(albumId)) {
      setSrc(cache.get(albumId));
      return;
    }
    let disposed = false;
    commands
      .libraryAlbumCover(albumId)
      .then((value) => {
        cache.set(albumId, value);
        if (!disposed) setSrc(value);
      })
      .catch(() => {
        cache.set(albumId, null);
        if (!disposed) setSrc(null);
      });
    return () => {
      disposed = true;
    };
  }, [albumId]);

  return (
    <div className={className} aria-hidden={src ? undefined : true}>
      {src ? (
        <img src={src} alt={`${title} 封面`} loading="lazy" />
      ) : (
        <span>{title.trim().charAt(0).toUpperCase() || "♪"}</span>
      )}
    </div>
  );
}
