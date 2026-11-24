import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { commands, type PlaylistDto } from "../bindings";

function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    const value = error as { code?: string; message?: string };
    return value.code ? `${value.code}: ${value.message}` : String(value.message);
  }
  return String(error);
}

export function Playlists() {
  const [items, setItems] = useState<PlaylistDto[]>([]);
  const [name, setName] = useState("");
  const [message, setMessage] = useState("");

  const refresh = useCallback(async () => {
    setItems(await commands.playlistList());
  }, []);

  useEffect(() => {
    refresh().catch((error) => setMessage(errorText(error)));
  }, [refresh]);

  async function create() {
    const value = name.trim();
    if (!value) {
      setMessage("请输入播放列表名称");
      return;
    }
    try {
      await commands.playlistCreate(value);
      setName("");
      await refresh();
      setMessage("已创建");
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function rename(item: PlaylistDto) {
    const value = window.prompt("新的名称", item.name)?.trim();
    if (!value || value === item.name) return;
    try {
      if (item.id == null) return;
      await commands.playlistRename(item.id, value);
      await refresh();
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  async function remove(item: PlaylistDto) {
    if (!window.confirm(`删除播放列表“${item.name}”？曲目不会从媒体库删除。`)) return;
    try {
      if (item.id == null) return;
      await commands.playlistDelete(item.id);
      await refresh();
    } catch (error) {
      setMessage(errorText(error));
    }
  }

  return (
    <section className="page">
      <header className="page-head">
        <div>
          <h1>播放列表</h1>
          <p className="muted">创建、重命名、删除，并进入列表调整曲目顺序。</p>
        </div>
        <span className="pill">{items.length} 个</span>
      </header>

      <div className="playlist-create">
        <input
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="新播放列表名称"
          onKeyDown={(event) => {
            if (event.key === "Enter") create();
          }}
        />
        <button onClick={create}>创建</button>
      </div>

      {message && <p className="notice">{message}</p>}

      {items.length === 0 ? (
        <div className="empty-card">
          <strong>还没有播放列表</strong>
          <span>输入名称创建一个，然后从媒体库添加曲目。</span>
        </div>
      ) : (
        <ul className="playlist-list">
          {items.map((item) => (
            <li key={item.id} className="playlist-row">
              <Link className="playlist-link" to={`/playlists/${item.id}`}>
                <strong>{item.name}</strong>
                <span className="muted">{item.track_count} 首</span>
              </Link>
              <div className="row-actions">
                <button className="ghost" onClick={() => rename(item)}>
                  重命名
                </button>
                <button className="ghost danger" onClick={() => remove(item)}>
                  删除
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
