/* Minimal Tauri IPC mock for browser E2E. Not shipped with the app. */
/* global window */
(() => {
  const tracks = [
    { id: 1, title: "晴天", artist: "周杰伦", album: "叶惠美", duration_ms: 269000, format: "MP3", path: "/music/qing tian.mp3" },
    { id: 2, title: "七里香", artist: "周杰伦", album: "七里香", duration_ms: 299000, format: "FLAC", path: "/music/qi li xiang.flac" },
    { id: 3, title: "Yellow", artist: "Coldplay", album: "Parachutes", duration_ms: 266000, format: "OGG", path: "/music/yellow.ogg" },
  ];
  const settings = {
    schemaVersion: 4, language: "zh-CN", theme: "dark", startMinimized: false,
    mode: "list_loop", crossfadeMs: 0, gapless: true, resumeOnStart: false,
    listDensity: "comfortable", reduceMotion: false, lyricsOffsetMs: 0,
    desktopLyricsFontSize: 28, desktopLyricsColor: "#ffffff", audioDevice: "",
    bitPerfect: false, sidebarWidth: 220, sidebarCollapsed: false,
    libraryExcludes: [], alwaysOnTop: false, opacity: 1, frameless: false,
  };
  const playlists = [{ id: 1, name: "通勤", track_count: 2 }];
  let queue = { items: [], index: -1, mode: "list_loop", muted: false };
  let seq = 0;
  let player = {
    state: "idle", path: null, position_ms: 0, duration_ms: 0,
    sample_rate: 44100, channels: 2, volume: 1, error: null,
    title: null, artist: null, album: null, muted: false,
    mode: "list_loop", queue_index: null, queue_len: 0,
    finished: false, device_lost: false, seq: 0,
  };
  const callbacks = new Map();
  const listeners = new Map();
  let nextCallback = 1;
  const clone = (value) => JSON.parse(JSON.stringify(value));

  function emit(event, payload) {
    for (const [id, listener] of listeners) {
      if (listener.event !== event) continue;
      listener.cb({ event, id, payload: clone(payload) });
    }
  }

  function pushPlayer(patch) {
    player = { ...player, ...patch, seq: ++seq };
    emit("player:state", player);
    return clone(player);
  }

  function queueSnapshot() {
    return clone(queue);
  }

  function setQueue(items, index, mode) {
    queue = { items: clone(items), index, mode, muted: false };
    const item = queue.items[index];
    if (item) {
      pushPlayer({
        state: "playing", path: item.path, title: item.title,
        artist: item.artist ?? null, album: item.album ?? null,
        duration_ms: item.duration_ms ?? 0, position_ms: 0,
        queue_index: index, queue_len: queue.items.length, mode,
      });
    }
  }

  const handlers = {
    app_info: () => ({
      name: "BiuPlayer", version: "0.1.0", tauri_version: "2.9.0",
      os: "macos", arch: "aarch64", log_dir: "/tmp/biu-logs",
      build_commit: "abc1234",
    }),
    app_paths: () => ({
      data_dir: "/tmp/biu-data", cache_dir: "/tmp/biu-cache", log_dir: "/tmp/biu-logs",
      settings_file: "/tmp/biu-data/settings.json", database_file: "/tmp/biu-data/library.db",
      portable: false,
    }),
    crash_report: () => null,
    startup_args: () => [],
    audio_devices: () => [
      { name: "MacBook Pro 扬声器", is_default: true, channels: 2, sample_rates: [44100, 48000], sample_formats: ["f32"] },
    ],
    settings_get: () => clone(settings),
    settings_update: ({ patch }) => {
      Object.assign(settings, patch ?? {});
      return clone(settings);
    },
    player_snapshot: () => clone(player),
    player_queue: () => queueSnapshot(),
    player_play_queue: ({ items, startIndex, mode }) => {
      const index = startIndex ?? 0;
      setQueue(items, index, mode);
      return clone(player);
    },
    player_play: () => pushPlayer({ state: "playing" }),
    player_pause: () => pushPlayer({ state: "paused" }),
    player_stop: () => pushPlayer({ state: "stopped", position_ms: 0 }),
    player_seek: ({ positionMs }) => pushPlayer({ position_ms: positionMs ?? 0 }),
    player_set_volume: ({ volume }) => pushPlayer({ volume: volume ?? 1 }),
    player_set_muted: ({ muted }) => pushPlayer({ muted: Boolean(muted) }),
    player_set_device: () => pushPlayer({ device_lost: false }),
    player_next: () => {
      const next = Math.min(queue.items.length - 1, queue.index + 1);
      setQueue(queue.items, next, queue.mode);
      return clone(player);
    },
    player_prev: () => {
      const prev = Math.max(0, queue.index - 1);
      setQueue(queue.items, prev, queue.mode);
      return clone(player);
    },
    player_jump: ({ index }) => { setQueue(queue.items, index ?? 0, queue.mode); return clone(player); },
    player_set_mode: ({ mode }) => { queue.mode = mode; return pushPlayer({ mode }); },
    player_queue_move: ({ from, to }) => {
      const items = queue.items.slice();
      const [moved] = items.splice(from, 1);
      items.splice(to, 0, moved);
      queue.items = items;
      if (queue.index === from) queue.index = to;
      return queueSnapshot();
    },
    player_queue_remove: ({ index }) => {
      queue.items.splice(index, 1);
      if (queue.index >= queue.items.length) queue.index = queue.items.length - 1;
      return queueSnapshot();
    },
    player_queue_clear: () => {
      queue = { items: [], index: -1, mode: queue.mode, muted: false };
      return pushPlayer({ state: "stopped", path: null, title: null, artist: null, queue_index: null, queue_len: 0 });
    },
    player_queue_add: ({ item }) => { queue.items.push(item); return queueSnapshot(); },
    player_queue_play_next: ({ item }) => { queue.items.splice(queue.index + 1, 0, item); return queueSnapshot(); },
    library_track_count: () => tracks.length,
    library_tracks: ({ limit, offset }) => clone(tracks.slice(offset ?? 0, (offset ?? 0) + (limit ?? 500))),
    library_search: ({ query, limit }) => {
      const q = String(query ?? "").trim().toLowerCase();
      if (!q) return [];
      return clone(tracks.filter((t) =>
        [t.title, t.artist, t.album].some((v) => String(v ?? "").toLowerCase().includes(q))
      ).slice(0, limit ?? 100));
    },
    library_add_folder: ({ path }) => ({
      files_seen: tracks.length, inserted: tracks.length, updated: 0, skipped: 0,
      elapsed_ms: 12, total_tracks: tracks.length, path,
    }),
    library_scan_start: ({ path }) => ({
      state: "completed", root: path, files_seen: tracks.length, inserted: 0,
      skipped: tracks.length, total_tracks: tracks.length, error: null,
    }),
    library_scan_start_many: ({ paths }) => ({
      state: "completed", root: (paths ?? [])[0] ?? null, files_seen: tracks.length, inserted: 0,
      skipped: tracks.length, total_tracks: tracks.length, error: null,
    }),
    library_import_files: ({ paths }) => (paths ?? []).length,
    portable_flag_set: ({ enabled }) => Boolean(enabled),
    library_scan_status: () => ({
      state: "completed", root: "/music", files_seen: tracks.length, inserted: 0,
      skipped: tracks.length, total_tracks: tracks.length, error: null,
    }),
    library_scan_pause: () => ({ state: "paused", root: "/music", files_seen: 0, inserted: 0, skipped: 0, total_tracks: tracks.length, error: null }),
    library_scan_resume: () => ({ state: "scanning", root: "/music", files_seen: 0, inserted: 0, skipped: 0, total_tracks: tracks.length, error: null }),
    library_scan_cancel: () => ({ state: "cancelled", root: "/music", files_seen: 0, inserted: 0, skipped: 0, total_tracks: tracks.length, error: null }),
    library_move_track: () => null,
    library_albums: () => [{ id: 1, title: "叶惠美", artist: "周杰伦", year: 2003, track_count: 1 }, { id: 2, title: "Parachutes", artist: "Coldplay", year: 2000, track_count: 1 }],
    library_album_tracks: () => clone(tracks.slice(0, 1)),
    library_artists: () => [{ id: 1, name: "周杰伦", track_count: 2 }, { id: 2, name: "Coldplay", track_count: 1 }],
    library_artist_tracks: () => clone(tracks.slice(0, 2)),
    library_album_artists: () => [
      { name: "周杰伦", track_count: 2, album_count: 2 },
      { name: "Coldplay", track_count: 1, album_count: 1 },
    ],
    library_album_artist_tracks: ({ name }) => clone(tracks.filter((track) => track.artist === name)),
    library_album_cover: () => null,
    library_genres: () => [{ name: "流行", track_count: 2 }],
    library_genre_tracks: () => clone(tracks.slice(0, 2)),
    library_years: () => [{ year: 2003, track_count: 1 }],
    library_year_tracks: () => clone(tracks.slice(0, 1)),
    library_folders: () => [{ id: 1, path: "/music", track_count: 3 }],
    library_folder_tracks: () => clone(tracks),
    library_remove_folder: () => null,
    lyrics_load: () => ({ title: "晴天", artist: "周杰伦", offset_ms: 0, lines: [{ timeMs: 0, text: "故事的小黄花" }, { timeMs: 5000, text: "从出生那年就飘着" }] }),
    desktop_lyrics_toggle: () => true,
    playlist_list: () => clone(playlists),
    playlist_items: () => clone(tracks.slice(0, 2).map((track, position) => ({ position, track }))),
    playlist_create: ({ name }) => { const id = playlists.length + 1; playlists.push({ id, name, track_count: 0 }); return id; },
    playlist_rename: ({ id, name }) => { const p = playlists.find((x) => x.id === id); if (p) p.name = name; return null; },
    playlist_delete: ({ id }) => { const i = playlists.findIndex((x) => x.id === id); if (i >= 0) playlists.splice(i, 1); return null; },
    playlist_add_tracks: ({ id }) => { const p = playlists.find((x) => x.id === id); if (p) p.track_count = (p.track_count ?? 0) + 1; return 1; },
    playlist_add_paths: () => 1,
    playlist_copy: ({ id, name }) => { const src = playlists.find((x) => x.id === id); const nid = playlists.length + 1; playlists.push({ id: nid, name, track_count: src?.track_count ?? 0 }); return nid; },
    playlist_remove_items: () => 1,
    playlist_remove_item: () => null,
    playlist_move_item: () => null,
    playlist_reorder: () => null,
    playlist_reorder_all: () => null,
  };

  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    convertFileSrc: (path) => path,
    transformCallback(cb, once) {
      const id = nextCallback++;
      callbacks.set(id, { cb, once });
      return id;
    },
    async invoke(cmd, args) {
      if (cmd === "plugin:event|listen") {
        const cb = callbacks.get(args?.handler);
        const id = nextCallback++;
        if (cb) listeners.set(id, { event: args?.event, cb: cb.cb });
        return id;
      }
      if (cmd === "plugin:event|unlisten") {
        listeners.delete(args?.eventId);
        return null;
      }
      if (cmd?.startsWith("plugin:window|")) {
        if (cmd.endsWith("|is_maximized")) return false;
        return null;
      }
      if (cmd?.startsWith("plugin:opener|")) return null;
      const handler = handlers[cmd];
      if (!handler) throw new Error(`mock: unhandled command ${cmd}`);
      return handler(args ?? {});
    },
  };
})();
