import { useCallback, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type PlaylistDto } from "../bindings";
import { useI18n } from "../i18n";
import { formatError } from "../errors";

export function Playlists() {
  const { t, lang } = useI18n();
  const [items, setItems] = useState<PlaylistDto[]>([]);
  const [name, setName] = useState("");
  const [message, setMessage] = useState("");
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const originalOrder = useRef<number[]>([]);

  const refresh = useCallback(async () => {
    const list = await commands.playlistList();
    setItems(list);
    if (originalOrder.current.length === 0) {
      originalOrder.current = list.map((item) => item.id ?? 0);
    }
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

  async function move(from: number, to: number) {
    if (from === to) return;
    const next = [...items];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    setItems(next);
    try {
      await commands.playlistReorderAll(next.map((item) => item.id ?? 0));
    } catch (error) {
      setMessage(formatError(error, t));
      await refresh();
    }
  }

  async function sortBy(mode: "name" | "nameDesc" | "default") {
    const next = [...items];
    if (mode === "default") {
      next.sort(
        (a, b) =>
          originalOrder.current.indexOf(a.id ?? 0) - originalOrder.current.indexOf(b.id ?? 0),
      );
    } else {
      const direction = mode === "name" ? 1 : -1;
      next.sort((a, b) => a.name.localeCompare(b.name, lang) * direction);
    }
    setItems(next);
    try {
      await commands.playlistReorderAll(next.map((item) => item.id ?? 0));
    } catch (error) {
      setMessage(formatError(error, t));
      await refresh();
    }
  }

  async function remove(item: PlaylistDto) {    if (!window.confirm(t("playlists.deleteConfirm", { name: item.name }))) return;
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
      {items.length > 1 && (
        <div className="playlist-sort">
          <span className="muted">{t("playlists.reorderHint")}</span>
          <button className="ghost" onClick={() => void sortBy("name")}>
            {t("playlists.sortName")}
          </button>
          <button className="ghost" onClick={() => void sortBy("nameDesc")}>
            {t("playlists.sortNameDesc")}
          </button>
          <button className="ghost" onClick={() => void sortBy("default")}>
            {t("playlists.sortDefault")}
          </button>
        </div>
      )}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>{t("playlists.emptyTitle")}</strong>
          <span>{t("playlists.emptyHint")}</span>
        </div>
      ) : (
        <ul className="playlist-list">
          {items.map((item) => (
            <li
              key={item.id}
              className={`playlist-row${dragIndex === items.indexOf(item) ? " dragging" : ""}`}
              draggable
              onDragStart={() => setDragIndex(items.indexOf(item))}
              onDragOver={(event) => event.preventDefault()}
              onDragEnd={() => setDragIndex(null)}
              onDrop={(event) => {
                event.preventDefault();
                if (dragIndex != null) void move(dragIndex, items.indexOf(item));
                setDragIndex(null);
              }}
            >
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
