import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export interface Track {
  id: number;
  path: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  cover_path: string | null;
  source: string;
  url: string | null;
  liked?: boolean;
}

export interface LyricsResult {
  content: string;
  source: string;
  timing: "word" | "line" | "plain" | "estimated";
  review: boolean;
  draft_only?: boolean;
  notice?: string;
}
export interface LyricsReview {
  content: string;
  changed_lines: number;
}
export interface LyricsConfig {
  automatic: boolean;
  network_lookup: boolean;
  lyricsplus_lookup: boolean;
  provider: "offline" | "groq" | "gemini" | "openai";
  cloud_consent: boolean;
  text_consent: boolean;
  auto_review: boolean;
  audio_pipeline: boolean;
  context_rewrite: boolean;
  offline_fallback: boolean;
  model: "base" | "small";
  language: "auto" | "vi" | "en";
  gemini_model: string;
  max_audio_seconds: number;
  daily_cloud_seconds: number;
  batch_limit: number;
}
export interface LyricsSettings {
  config: LyricsConfig;
  keys: string[];
  models: {name: string; bytes: number; installed: boolean}[];
  downloading: boolean;
  cloud_seconds_used: number;
  lyrics_dir?: string;
  lyrics_count?: number;
}
export interface LyricsProgress {
  path: string;
  stage: number;
  total?: number;
  message: string;
  state: "running" | "done" | "error" | "needs_review";
}
export interface LyricsQueue {
  running: boolean;
  error: string;
  jobs: {track: Pick<Track, "path" | "title" | "artist" | "album" | "duration" | "url">; state: string; message: string}[];
}

export interface HomeData {
  recommended: OnlineTrack[];
  library_recent: Track[];
  liked_tracks: Track[];
  history_recent: {
    title: string;
    artist: string;
    source: string;
    url: string | null;
    played_at: number;
    completed: boolean;
  }[];
  top_artists: [string, number][];
}

export interface OnlineTrack {
  title: string;
  artist: string;
  album: string;
  duration: number;
  thumbnail: string | null;
  url: string;
  source: string;
  reason?: string | null;
}

export interface DownloadProgress {
  url: string;
  track_id: number | null;
  percent: number;
  status: string;
}

export interface Playlist {
  id: number;
  name: string;
  count: number;
  cover_path: string | null;
}

export interface SpotifyTrackItem {
  title: string;
  artist: string;
  album?: string | null;
  duration: number;
  thumbnail?: string | null;
  url: string;
}

export interface SpotifyCollection {
  kind: "playlist" | "album" | "track";
  id: string;
  title: string;
  author: string;
  thumbnail?: string | null;
  total: number;
  tracks: SpotifyTrackItem[];
}

export const api = {
  scanFolder: (path: string) => invoke<number>("scan_folder", { path }),
  listTracks: (query?: string) => invoke<Track[]>("list_tracks", { query }),
  deleteTrack: (id: number) => invoke<void>("delete_track", { id }),
  listPlaylists: () => invoke<Playlist[]>("list_playlists"),
  createPlaylist: (name: string) => invoke<number>("create_playlist", { name }),
  renamePlaylist: (id: number, name: string) => invoke<void>("rename_playlist", { id, name }),
  setPlaylistCover: (id: number, src: string | null) =>
    invoke<string | null>("set_playlist_cover", { id, src }),
  saveBackgroundImage: (src: string) =>
    invoke<string>("save_background_image", { src }),
  saveBackgroundBytes: (ext: string, data: number[]) =>
    invoke<string>("save_background_bytes", { ext, data }),
  deleteBackgroundImage: (path: string) =>
    invoke<void>("delete_background_image", { path }),
  deletePlaylist: (id: number) => invoke<void>("delete_playlist", { id }),
  addToPlaylist: (playlistId: number, trackId: number) =>
    invoke<void>("add_to_playlist", { playlistId, trackId }),
  removeFromPlaylist: (playlistId: number, trackId: number) =>
    invoke<void>("remove_from_playlist", { playlistId, trackId }),
  getPlaylist: (id: number) => invoke<Track[]>("get_playlist", { id }),
  searchOnline: (query: string, source?: string, limit?: number) =>
    invoke<OnlineTrack[]>("search_online", { query, source, limit }),
  resolveInfo: (url: string) => invoke<OnlineTrack[]>("resolve_info", { url }),
  resolveStream: (url: string) => invoke<string>("resolve_stream", { url }),
  addOnlineToLibrary: (track: OnlineTrack) =>
    invoke<Track>("add_online_to_library", { track }),
  downloadTrack: (track: OnlineTrack, trackId?: number | null) =>
    invoke<Track>("download_track", { track, trackId: trackId ?? null }),
  recordPlay: (
    title: string,
    artist: string,
    source: string,
    url: string | null,
    completed: boolean,
  ) => invoke<void>("record_play", { title, artist, source, url, completed }),
  getRecommendations: (force = false) =>
    invoke<OnlineTrack[]>("get_recommendations", { force }),
  getHomeData: (force = false) =>
    invoke<HomeData>("get_home_data", { force }),
  toggleLike: (id: number) =>
    invoke<boolean>("toggle_like", { id }),
  listLiked: () =>
    invoke<Track[]>("list_liked"),
  windowAction: (action: "minimize" | "maximize" | "close" | "drag") =>
    invoke<void>("window_action", { action }),
  setStealthMode: (enabled: boolean) =>
    invoke<void>("set_stealth_mode", { enabled }),
  toggleIsland: (show?: boolean, stealth?: boolean) =>
    invoke<boolean>("toggle_island_window", {
      show: show ?? null,
      stealth: stealth ?? null,
    }),
  setIslandExpanded: (expanded: boolean) =>
    invoke<void>("set_island_expanded", { expanded }),
  getLyrics: (
    path: string,
    title: string,
    artist: string,
    album: string,
    duration: number,
    url?: string | null,
  ) => invoke<string | null>("get_lyrics", { path, title, artist, album, duration, url: url ?? null }),
  readLyrics: (track: Track) => invoke<LyricsResult | null>("lyrics_read", { track }),
  generateLyrics: (track: Track, refreshOnline = false, rewrite = false) => invoke<LyricsResult>("lyrics_generate", { track, refreshOnline, rewrite }),
  deleteLyrics: (track: Track) => invoke<void>("lyrics_delete", { track }),
  reviewLyrics: (content: string) => invoke<LyricsReview>("lyrics_review_text", { content }),
  transcribeLyrics: (track: Track) => invoke<LyricsResult>("lyrics_transcribe", { track }),
  lyricsSettings: () => invoke<LyricsSettings>("lyrics_settings"),
  openLyricsFolder: () => invoke<string>("open_lyrics_folder"),
  saveLyricsSettings: (config: LyricsConfig, apiKey: string | null = null) => invoke<void>("lyrics_save_settings", { config, apiKey }),
  storeLyricsKey: (provider: string, apiKey: string) => invoke<void>("lyrics_store_key", { provider, apiKey }),
  downloadLyricsModel: (model: string) => invoke<void>("lyrics_download_model", { model }),
  cancelLyricsDownload: () => invoke<void>("lyrics_cancel_download"),
  lyricsQueue: () => invoke<LyricsQueue>("lyrics_queue"),
  startLyricsLibrary: (resume = false) => invoke<LyricsQueue>("lyrics_start_library", { resume }),
  pauseLyricsLibrary: () => invoke<void>("lyrics_pause"),
  saveLyrics: (
    path: string,
    title: string,
    artist: string,
    album: string,
    duration: number,
    content: string,
  ) => invoke<void>("save_lyrics", { path, title, artist, album, duration, content }),
  getAudioProxyUrl: (url: string) => invoke<string>("get_audio_proxy_url", { url }),
  importSpotify: (url: string, clientId?: string, clientSecret?: string) =>
    invoke<SpotifyCollection>("import_spotify", {
      url,
      clientId: clientId ?? null,
      clientSecret: clientSecret ?? null,
    }),
};

export function extractYoutubeId(urlOrPath?: string | null): string | null {
  if (!urlOrPath) return null;
  const m = urlOrPath.match(/(?:v=|\/embed\/|youtu\.be\/|\/v\/|vi=|\/shorts\/)([a-zA-Z0-9_-]{11})/);
  if (m) return m[1];
  return null;
}

export function generateDynamicCoverSvg(title = "DLC Music", artist = "Audio Deck"): string {
  let hash = 0;
  const str = `${title} ${artist}`;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  hash = Math.abs(hash);

  const GRADIENTS = [
    ["#0f2027", "#203a43", "#2c5364", "#00d2ff"], // Ocean Midnight
    ["#141E30", "#243B55", "#00c6ff", "#0072ff"], // Deep Blue Neo
    ["#1a0826", "#4a00e0", "#8e2de2", "#f72585"], // Cyberpunk Purple
    ["#23074d", "#cc5333", "#ff5500", "#ffaa00"], // SoundCloud Sunset
    ["#0b1e13", "#134e5e", "#71b280", "#1db954"], // Spotify Emerald
    ["#1c0505", "#4a0e17", "#e52d27", "#b31217"], // Crimson Dark
    ["#12131C", "#2B1055", "#7597DE", "#6C5CE7"], // Violet Night
    ["#191919", "#303030", "#f39c12", "#e67e22"], // Vinyl Gold
  ];

  const palette = GRADIENTS[hash % GRADIENTS.length];
  const tClean = (title || "Music").trim();
  const aClean = (artist || "Artist").trim();
  const initial = (tClean[0] || "D").toUpperCase();
  const subInitial = (aClean[0] || "M").toUpperCase();

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 300" width="100%" height="100%">
    <defs>
      <linearGradient id="g" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color="${palette[0]}" />
        <stop offset="50%" stop-color="${palette[1]}" />
        <stop offset="100%" stop-color="${palette[2]}" />
      </linearGradient>
      <radialGradient id="disc" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#111111" />
        <stop offset="50%" stop-color="#1c1c1c" />
        <stop offset="85%" stop-color="#0a0a0a" />
        <stop offset="100%" stop-color="#000000" />
      </radialGradient>
    </defs>
    <rect width="300" height="300" fill="url(#g)" />
    <circle cx="150" cy="150" r="130" fill="none" stroke="rgba(255,255,255,0.07)" stroke-width="2" />
    <circle cx="150" cy="150" r="110" fill="none" stroke="rgba(255,255,255,0.05)" stroke-width="1.5" />
    <circle cx="150" cy="150" r="90" fill="none" stroke="rgba(255,255,255,0.07)" stroke-width="2" />
    <circle cx="150" cy="150" r="72" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="1.5" />
    <circle cx="150" cy="150" r="54" fill="url(#disc)" stroke="${palette[3]}" stroke-width="2.5" />
    <!-- Lucide audio-lines (ISC), see static/lucide-LICENSE.txt. -->
    <g transform="translate(114, 114) scale(3)" fill="none" stroke="${palette[3]}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M2 10v3" /><path d="M6 6v11" /><path d="M10 3v18" />
      <path d="M14 8v7" /><path d="M18 5v13" /><path d="M22 10v3" />
    </g>
    <rect x="18" y="18" width="56" height="22" rx="6" fill="rgba(0,0,0,0.55)" />
    <text x="46" y="33" fill="${palette[3]}" font-family="system-ui, -apple-system, sans-serif" font-size="10.5" font-weight="800" text-anchor="middle" letter-spacing="1.5">DLC</text>
    <rect x="18" y="244" width="264" height="38" rx="8" fill="rgba(0,0,0,0.6)" />
    <text x="32" y="268" fill="#ffffff" font-family="system-ui, -apple-system, sans-serif" font-size="13" font-weight="700">${initial}${subInitial} · MUSIC</text>
  </svg>`;

  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

export function getTrackCover(
  coverPathOrThumb?: string | null,
  urlOrPath?: string | null,
  title?: string,
  artist?: string,
): string {
  if (coverPathOrThumb && coverPathOrThumb.trim()) {
    const trimmed = coverPathOrThumb.trim();
    if (trimmed.startsWith("http://") || trimmed.startsWith("https://") || trimmed.startsWith("data:")) {
      return trimmed;
    }
    return convertFileSrc(trimmed);
  }

  const ytId = extractYoutubeId(urlOrPath) || extractYoutubeId(coverPathOrThumb);
  if (ytId) {
    return `https://i.ytimg.com/vi/${ytId}/hqdefault.jpg`;
  }

  return generateDynamicCoverSvg(title, artist);
}

export function handleImgError(
  e: Event,
  fallbackTitle?: string,
  fallbackArtist?: string,
  urlOrPath?: string,
) {
  const imgEl = e.currentTarget as HTMLImageElement;
  if (!imgEl) return;
  const curSrc = imgEl.src || "";
  if (curSrc.startsWith("data:image/svg+xml")) {
    imgEl.onerror = null;
    return;
  }
  if (curSrc.includes("hqdefault.jpg")) {
    imgEl.src = curSrc.replace("hqdefault.jpg", "mqdefault.jpg");
    return;
  }
  const ytId = extractYoutubeId(urlOrPath);
  if (ytId && !curSrc.includes(ytId)) {
    imgEl.src = `https://i.ytimg.com/vi/${ytId}/mqdefault.jpg`;
    return;
  }
  imgEl.onerror = null;
  imgEl.src = generateDynamicCoverSvg(fallbackTitle, fallbackArtist);
}

export function coverSrc(pathOrUrl: string | null | undefined): string {
  if (!pathOrUrl) return "";
  if (pathOrUrl.startsWith("http://") || pathOrUrl.startsWith("https://") || pathOrUrl.startsWith("data:")) {
    return pathOrUrl;
  }
  return convertFileSrc(pathOrUrl);
}

let tempSeq = -1;
export function onlineToTrack(o: OnlineTrack): Track {
  return {
    id: tempSeq--,
    path: o.url,
    title: o.title,
    artist: o.artist,
    album: o.album,
    duration: o.duration,
    cover_path: o.thumbnail,
    source: o.source,
    url: o.url,
  };
}

export function fmtTime(s: number): string {
  if (!isFinite(s) || s < 0) s = 0;
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60);
  return `${m}:${sec.toString().padStart(2, "0")}`;
}
