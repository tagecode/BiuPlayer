import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  commands,
  type FolderDto,
  type GenreDto,
  type LibraryTrackDto_Serialize,
  type QueueItemDto_Serialize,
  type YearDto,
} from "../bindings";
import { LibraryNav } from "../components/LibraryNav";
import { TrackTable } from "../components/TrackTable";
import { formatError } from "../errors";
import { useI18n } from "../i18n";

function toQueue(tracks: LibraryTrackDto_Serialize[]): QueueItemDto_Serialize[] {
  return tracks.map((track) => ({
    path: track.path,
    title: track.title,
    artist: track.artist ?? null,
    album: track.album ?? null,
    duration_ms: track.duration_ms ?? null,
  }));
}

function useTracks(load: () => Promise<LibraryTrackDto_Serialize[]>) {
  const { t } = useI18n();
  const [tracks, setTracks] = useState<LibraryTrackDto_Serialize[]>([]);
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    setTracks(await load());
  }, [load]);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh]);

  async function play(index: number) {
    try {
      await commands.playerPlayQueue(toQueue(tracks), index, "list_loop");
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  return { tracks, message, play };
}

function Header({ kicker, title, desc }: { kicker: string; title: string; desc: string }) {
  return (
    <header className="page-head">
      <div>
        <p className="page-kicker">{kicker}</p>
        <h1>{title}</h1>
        <p className="muted">{desc}</p>
      </div>
    </header>
  );
}

export function Genres() {
  const { t } = useI18n();
  const [rows, setRows] = useState<GenreDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryGenres()
      .then(setRows)
      .catch((error) => setMessage(formatError(error, t)));
  }, []);

  return (
    <section className="page">
      <Header kicker="P-06" title={t("pages.genres.title")} desc={t("pages.genres.desc")} />
      <LibraryNav />
      {message && <p className="notice">{message}</p>}
      {rows.length === 0 ? (
        <p className="muted">{t("genres.empty")}</p>
      ) : (
        <div className="tile-grid">
          {rows.map((row) => (
            <Link key={row.name} to={`/library/genres/${encodeURIComponent(row.name)}`} className="tile">
              <strong>{row.name}</strong>
              <span className="muted">{row.track_count ?? 0} {t("common.tracksSuffix")}</span>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}

export function GenreDetail() {
  const { t } = useI18n();
  const { name = "" } = useParams();
  const load = useCallback(() => commands.libraryGenreTracks(name), [name]);
  const { tracks, message, play } = useTracks(load);

  return (
    <section className="page">
      <Header kicker="P-06" title={name} desc={t("genres.detailDesc")} />
      <p className="breadcrumb">
        <Link to="/library/genres">{t("pages.genres.title")}</Link> / {name}
      </p>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}

export function Years() {
  const { t } = useI18n();
  const [rows, setRows] = useState<YearDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryYears()
      .then(setRows)
      .catch((error) => setMessage(formatError(error, t)));
  }, []);

  return (
    <section className="page">
      <Header kicker="P-06" title={t("pages.years.title")} desc={t("pages.years.desc")} />
      <LibraryNav />
      {message && <p className="notice">{message}</p>}
      {rows.length === 0 ? (
        <p className="muted">{t("years.empty")}</p>
      ) : (
        <div className="tile-grid">
          {rows.map((row) => (
            <Link key={row.year} to={`/library/years/${row.year}`} className="tile">
              <strong>{row.year}</strong>
              <span className="muted">{row.track_count ?? 0} {t("common.tracksSuffix")}</span>
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}

export function YearDetail() {
  const { t } = useI18n();
  const { year = "0" } = useParams();
  const load = useCallback(() => commands.libraryYearTracks(Number(year)), [year]);
  const { tracks, message, play } = useTracks(load);

  return (
    <section className="page">
      <Header kicker="P-06" title={year} desc={t("years.detailDesc")} />
      <p className="breadcrumb">
        <Link to="/library/years">{t("pages.years.title")}</Link> / {year}
      </p>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}

export function Folders() {
  const { t } = useI18n();
  const [rows, setRows] = useState<FolderDto[]>([]);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .libraryFolders()
      .then(setRows)
      .catch((error) => setMessage(formatError(error, t)));
  }, []);

  return (
    <section className="page">
      <Header kicker="P-06" title={t("pages.folders.title")} desc={t("pages.folders.desc")} />
      <LibraryNav />
      {message && <p className="notice">{message}</p>}
      {rows.length === 0 ? (
        <p className="muted">{t("folders.empty")}</p>
      ) : (
        <ul className="folder-list">
          {rows.map((row) => (
            <li key={row.id}>
              <Link to={`/library/folders/${row.id}`} className="folder-row">
                <span className="mono">{row.path}</span>
                <span className="muted">{row.track_count ?? 0} {t("common.tracksSuffix")}</span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

export function FolderDetail() {
  const { t } = useI18n();
  const { id = "0" } = useParams();
  const load = useCallback(() => commands.libraryFolderTracks(Number(id)), [id]);
  const { tracks, message, play } = useTracks(load);

  return (
    <section className="page">
      <Header kicker="P-06" title={t("pages.folders.title")} desc={t("folders.detailDesc")} />
      <p className="breadcrumb">
        <Link to="/library/folders">{t("pages.folders.title")}</Link> / #{id}
      </p>
      {message && <p className="notice">{message}</p>}
      <TrackTable tracks={tracks} onPlay={play} />
    </section>
  );
}
