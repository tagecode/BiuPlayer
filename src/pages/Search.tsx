import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { commands, type LibraryTrackDto_Serialize } from "../bindings";
import { TrackTable } from "../components/TrackTable";
import { formatError } from "../errors";
import { useI18n } from "../i18n";

export function Search() {
  const { t } = useI18n();
  const [params, setParams] = useSearchParams();
  const [query, setQuery] = useState(params.get("q") ?? "");
  const [hits, setHits] = useState<LibraryTrackDto_Serialize[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    setQuery(params.get("q") ?? "");
  }, [params]);

  useEffect(() => {
    const value = query.trim();
    if (!value) {
      setHits([]);
      setMessage("");
      return;
    }
    setLoading(true);
    const timer = window.setTimeout(() => {
      commands
        .librarySearch(value, 100)
        .then((list) => {
          setHits(list);
          setMessage("");
        })
        .catch((error) => setMessage(formatError(error, t)))
        .finally(() => setLoading(false));
    }, 200);
    return () => window.clearTimeout(timer);
  }, [query]);

  function submit(value: string) {
    const next = value.trim();
    if (next) setParams({ q: next });
    else setParams({});
  }

  async function play(index: number) {
    try {
      await commands.playerPlayQueue(
        hits.map((track) => ({
          path: track.path,
          title: track.title,
          artist: track.artist ?? null,
          album: track.album ?? null,
          duration_ms: track.duration_ms ?? null,
        })),
        index,
        "list_loop",
      );
      setMessage(t("library.nowPlaying", { title: hits[index]?.title ?? "" }));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  const trimmed = query.trim();

  return (
    <section>
      <div className="page-head">
        <div>
          <h2>{t("pages.search.title")}</h2>
          <p className="muted">{t("pages.search.desc")}</p>
        </div>
      </div>

      <form
        className="search-page-form"
        onSubmit={(event) => {
          event.preventDefault();
          submit(query);
        }}
      >
        <input
          autoFocus
          aria-label={t("library.searchPlaceholder")}
          placeholder={t("library.searchPlaceholder")}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
        <button type="submit" className="primary">
          {t("common.search")}
        </button>
      </form>

      {message && <p className="notice">{message}</p>}
      {!trimmed ? (
        <p className="muted">{t("search.hint")}</p>
      ) : loading ? (
        <p className="muted">{t("common.loading")}…</p>
      ) : hits.length === 0 ? (
        <p className="muted">{t("library.noResults")}</p>
      ) : (
        <>
          <p className="muted">{t("search.resultCount", { count: hits.length })}</p>
          <TrackTable tracks={hits} onPlay={play} highlight={trimmed} />
        </>
      )}
    </section>
  );
}
