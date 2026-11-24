import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type PlaylistDto } from "../bindings";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function Playlists() {
  const { t } = useI18n();
  const [items, setItems] = useState<PlaylistDto[]>([]);
  const [name, setName] = useState("");
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    setItems(await commands.playlistList());
  }, []);

  useEffect(() => {
    refresh().catch((error) => setMessage(formatError(error, t)));
  }, [refresh]);

  async function create() {
    const value = name.trim();
    if (!value) {
      setMessage(t("playlists.namePrompt"));
      return;
    }
    try {
      await commands.playlistCreate(value);
      setName("");
      await refresh();
      setMessage(t("playlists.created"));
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function rename(item: PlaylistDto) {
    const value = window.prompt(t("playlists.renamePrompt"), item.name)?.trim();
    if (!value || value === item.name) return;
    try {
      if (item.id == null) return;
      await commands.playlistRename(item.id, value);
      await refresh();
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  async function remove(item: PlaylistDto) {
    if (!window.confirm(t("playlists.deleteConfirm", { name: item.name }))) return;
    try {
      if (item.id == null) return;
      await commands.playlistDelete(item.id);
      await refresh();
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <h1>{t("pages.playlists.title")}</h1>
          <p className="muted">{t("pages.playlists.desc")}</p>
        </div>
        <span className="pill">{items.length} {t("common.playlistsSuffix")}</span>
      </header>

      <div className="playlist-create">
        <input
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder={t("playlists.newPlaceholder")}
          onKeyDown={(event) => {
            if (event.key === "Enter") create();
          }}
        />
        <button onClick={create}>{t("common.create")}</button>
      </div>

      {message && <p className="notice">{message}</p>}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>{t("playlists.emptyTitle")}</strong>
          <span>{t("playlists.emptyHint")}</span>
        </div>
      ) : (
        <ul className="playlist-list">
          {items.map((item) => (
            <li key={item.id} className="playlist-row">
              <Link className="playlist-link" to={`/playlists/${item.id}`}>
                <strong>{item.name}</strong>
                <span className="muted">{item.track_count} {t("common.tracksSuffix")}</span>
              </Link>
              <div className="row-actions">
                <button className="ghost" onClick={() => rename(item)}>
                  {t("common.rename")}
                </button>
                <button className="ghost danger" onClick={() => remove(item)}>
                  {t("common.delete")}
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
