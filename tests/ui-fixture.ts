(() => {
  const cover = 'data:image/svg+xml,' + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect width="200" height="200" fill="#313244"/><circle cx="100" cy="100" r="56" fill="#cba6f7"/><circle cx="100" cy="100" r="18" fill="#1e1e2e"/></svg>');
  const wav = new Uint8Array(44 + 8000 * 60);
  const view = new DataView(wav.buffer);
  const text = (at: number, value: string) => [...value].forEach((c, i) => wav[at + i] = c.charCodeAt(0));
  text(0, 'RIFF'); view.setUint32(4, wav.length - 8, true); text(8, 'WAVEfmt ');
  view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, 1, true);
  view.setUint32(24, 8000, true); view.setUint32(28, 8000, true);
  view.setUint16(32, 1, true); view.setUint16(34, 8, true); text(36, 'data');
  view.setUint32(40, wav.length - 44, true); wav.fill(128, 44);
  const audio = URL.createObjectURL(new Blob([wav], { type: 'audio/wav' }));
  const tracks = Array.from({ length: 12 }, (_, i) => ({
    id: i + 1, path: `fixture-${i}.wav`,
    title: i === 0 ? 'Bản nhạc tiếng Việt với tên thật dài để kiểm tra bố cục và dấu câu' : `Bài hát ${i + 1}`,
    artist: 'Nghệ sĩ có tên dài · Bản thu trực tiếp', album: 'Tuyển tập âm nhạc',
    duration: 60, cover_path: cover, source: 'local', url: null, liked: i === 1,
  }));
  const online = tracks.map(t => ({ ...t, thumbnail: cover, url: `https://example.invalid/track/${t.id}`, source: 'youtube' }));
  const lyrics = { content: '[00:00.00]<00:00.00>Xin <00:00.60>chào <00:01.20>Việt <00:01.80>Nam\n[00:04.00]Lời bài hát được giữ nguyên', source: 'manual', timing: 'word', review: false };
  let callback = 0;
  let savedLyricsConfig: Record<string, unknown> | null = null;
  const callbacks = new Map<number, (event: unknown) => void>();
  const listeners = new Map<number, { event: string; handler: number }>();
  const emit = (event: string, payload: unknown) => {
    for (const [id, listener] of listeners) if (listener.event === event) callbacks.get(listener.handler)?.({ event, id, payload });
  };
  const calls: { command: string; args: { id?: number } }[] = [];
  const native = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
    transformCallback(handler: (event: unknown) => void) { callbacks.set(++callback, handler); return callback; },
    unregisterCallback(id: number) { callbacks.delete(id); },
    convertFileSrc(path: string) { return path.endsWith('.wav') ? audio : cover; },
    async invoke(command: string, args: { id?: number; event?: string; handler?: number; eventId?: number; payload?: unknown; config?: Record<string, unknown> } = {}) {
      calls.push({ command, args });
      if (command === 'list_tracks' || command === 'get_playlist') return structuredClone(tracks);
      if (command === 'list_playlists') return [{ id: 1, name: 'Danh sách phát có tên dài để kiểm tra', count: tracks.length, cover_path: cover }];
      if (command === 'toggle_like') {
        const t = tracks.find(t => t.id === args.id);
        if (!t) throw new Error('Unknown fixture track');
        return t.liked = !t.liked;
      }
      if (command === 'list_liked') return tracks.filter(t => t.liked);
      if (command === 'get_home_data') return { recommended: online, library_recent: tracks, liked_tracks: tracks.filter(t => t.liked), history_recent: tracks.slice(0, 3), top_artists: [] };
      if (command === 'get_recommendations' || command === 'search_online') return online;
      if (command === 'import_spotify') return { kind: 'playlist', id: 'fixture', title: tracks[0].title, author: tracks[0].artist, thumbnail: cover, total: online.length, tracks: online };
      if (command === 'lyrics_read' || command === 'lyrics_generate') return lyrics;
      if (command === 'get_lyrics') return lyrics.content;
      if (command === 'lyrics_queue') return { running: false, error: '', jobs: [] };
      if (command === 'lyrics_save_settings') { savedLyricsConfig = JSON.parse(JSON.stringify(args.config)); return null; }
      if (command === 'lyrics_settings') return {
        config: savedLyricsConfig ?? { automatic: false, network_lookup: false, lyricsplus_lookup: false, provider: 'offline', cloud_consent: false, text_consent: false, auto_review: false, audio_pipeline: false, context_rewrite: false, offline_fallback: false, model: 'base', language: 'vi', gemini_model: 'gemini-2.5-flash', max_audio_seconds: 600, daily_cloud_seconds: 3600, batch_limit: 20 },
        keys: [], models: [], downloading: false, cloud_seconds_used: 0,
      };
      if (command === 'resolve_stream' || command === 'get_audio_proxy_url') return audio;
      if (command === 'toggle_island_window') return false;
      if (command === 'plugin:event|listen') {
        const id = ++callback;
        listeners.set(id, { event: args.event!, handler: args.handler! });
        return id;
      }
      if (command === 'plugin:event|unlisten') { listeners.delete(args.eventId!); return null; }
      if (command === 'plugin:event|emit') { emit(args.event!, args.payload); return null; }
      if (command === 'plugin:window|is_maximized') return false;
      if (['set_app_icon', 'set_stealth_mode', 'window_action', 'record_play', 'save_lyrics', 'lyrics_save_settings', 'plugin:event|emit', 'plugin:event|unlisten'].includes(command)) return null;
      throw new Error(`Unhandled UI fixture IPC: ${command}`);
    },
  };
  Object.assign(window, {
    __uiCalls: calls,
    __uiEmit: emit,
    __TAURI_EVENT_PLUGIN_INTERNALS__: { unregisterListener() {} },
    __TAURI_INTERNALS__: native,
  });
})();
