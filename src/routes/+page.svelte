<script lang="ts">
  import { Minus, Square, X, Search, Menu, House, Music2, Globe, Settings, FolderPlus, ListMusic, Plus, Image, Check } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open, ask } from "@tauri-apps/plugin-dialog";
  import {
    api,
    coverSrc,
    fmtTime,
    type Track,
    type Playlist,
    type OnlineTrack,
    type DownloadProgress,
  } from "$lib/api";
  import { parseLrc, activeIndex, type ParsedLyrics } from "$lib/lrc";
  import { lyricsState } from "$lib/lyrics.svelte";
  import { player } from "$lib/player.svelte";
  import { theme } from "$lib/theme.svelte";
  import PlayerBar from "$lib/PlayerBar.svelte";
  import TrackList from "$lib/TrackList.svelte";
import OnlinePanel from "$lib/OnlinePanel.svelte";
  import TrackDetailView from "$lib/TrackDetailView.svelte";
  import SettingsPanel from "$lib/SettingsPanel.svelte";
  import HomeView from "$lib/HomeView.svelte";
  import EqualizerModal from "$lib/EqualizerModal.svelte";
  import SpotifyImportModal from "$lib/SpotifyImportModal.svelte";
  import AppSpeakerLogo from "$lib/AppSpeakerLogo.svelte";
  import KaraokeRoom from "$lib/KaraokeRoom.svelte";
  import { lyricsUi } from "$lib/lyricsUiConfig.svelte";

  let tracks = $state<Track[]>([]);
  let playlists = $state<Playlist[]>([]);
  let active = $state<number | "home" | "online" | "settings" | null>(
    (typeof localStorage !== "undefined" && localStorage.getItem("dlc.last.active")
      ? (localStorage.getItem("dlc.last.active") === "null" ? null : localStorage.getItem("dlc.last.active") as any)
      : "home")
  );
  let inspectedTrack = $state<Track | null>(null);
  let showEqualizer = $state(false);
  let showSpotifyImport = $state(false);
  let islandVisible = $state(false);
  let islandLyrics = $state<ParsedLyrics | null>(null);
  let islandLyricTrackKey = "";

  // Unified Search State (placed above "Thư viện")
  let searchText = $state("");
  let searchSource = $state<"all" | "youtube" | "soundcloud" | "local">("all");
  let showSourceMenu = $state(false);
  let onlineResults = $state<OnlineTrack[]>([]);
  let onlineBusy = $state(false);
  let onlineErr = $state("");
  let loadingMoreSearch = $state(false);

  async function loadMoreSearch() {
    const q = searchText.trim();
    if (loadingMoreSearch || !q) return;
    loadingMoreSearch = true;
    try {
      const src = searchSource === "local" ? "all" : searchSource;
      const targetCount = onlineResults.length + 14;
      const res = await api.searchOnline(q, src, targetCount);
      if (res && res.length > 0) {
        const urls = new Set(onlineResults.map((r) => r.url));
        const merged = [...onlineResults];
        for (const item of res) {
          if (!urls.has(item.url)) {
            urls.add(item.url);
            merged.push(item);
          }
        }
        onlineResults = merged;
      }
    } catch {
      // ignore
    } finally {
      loadingMoreSearch = false;
    }
  }

  let scan = $state<{ done: number; total: number } | null>(null);
  let downloads = $state<Record<string, number>>({});
  let showQueue = $state(false);
  let showLyrics = $state(false);
  let tempFullscreen = $state(false);
  const isFullscreenLyrics = $derived(lyricsUi.config.windowMode === 'fullscreen' || tempFullscreen);
  let newName = $state("");

  $effect(() => {
    if (inspectedTrack) showLyrics = false;
  });

  function toggleLyrics() {
    showQueue = false;
    showLyrics = !showLyrics;
    tempFullscreen = false;
  }

  let lyricsSettingsRevision = $state(0);
  function openLyricsSettings() {
    lyricsSettingsRevision++;
    localStorage.setItem("dlc.settings.tab", "lyrics");
    inspectedTrack = null;
    showLyrics = false;
    tempFullscreen = false;
    active = "settings";
  }

  // Drag & Drop Link Playback State
  let isDraggingOver = $state(false);
  let dragDepth = 0;
  let dropBusy = $state(false);
  let dropNotice = $state<{ text: string; kind: "ok" | "warn" } | null>(null);
  let dropNoticeTimer: ReturnType<typeof setTimeout> | null = null;

  function showDropNotice(text: string, kind: "ok" | "warn") {
    dropNotice = { text, kind };
    if (dropNoticeTimer) clearTimeout(dropNoticeTimer);
    dropNoticeTimer = setTimeout(() => {
      dropNotice = null;
    }, 4000);
  }

  function extractDroppedUrl(dt: DataTransfer | null): string | null {
    if (!dt) return null;
    const candidates = [
      dt.getData("text/uri-list"),
      dt.getData("URL"),
      dt.getData("text/plain"),
      dt.getData("text/html"),
    ];
    for (const raw of candidates) {
      if (!raw) continue;
      // Check text/uri-list lines first
      for (const line of raw.split(/\r?\n/)) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith("#")) continue;
        if (
          trimmed.startsWith("http://") ||
          trimmed.startsWith("https://") ||
          trimmed.startsWith("spotify:")
        ) {
          return trimmed;
        }
      }
      // Extract href="..." if HTML was dragged
      const hrefMatch = raw.match(/href=["'](https?:\/\/[^"'<>]+|spotify:[^"'<>]+)["']/i);
      if (hrefMatch?.[1]) return hrefMatch[1].trim();
      // Extract any embedded URL from plain text
      const urlMatch = raw.match(/(https?:\/\/[^\s"'<>]+|spotify:[a-zA-Z0-9:]+)/i);
      if (urlMatch?.[1]) return urlMatch[1].trim();
    }
    return null;
  }

  function probeDirectAudio(url: string): Promise<number> {
    return new Promise((resolve, reject) => {
      const probe = new Audio();
      const timer = setTimeout(() => {
        probe.src = "";
        reject(new Error("Timeout"));
      }, 6000);
      probe.preload = "metadata";
      probe.onloadedmetadata = () => {
        clearTimeout(timer);
        const dur = Number.isFinite(probe.duration) ? probe.duration : 0;
        probe.src = "";
        resolve(dur);
      };
      probe.onerror = () => {
        clearTimeout(timer);
        probe.src = "";
        reject(new Error("Invalid audio"));
      };
      probe.src = url;
    });
  }

  async function playDroppedLink(rawUrl: string) {
    const url = rawUrl.trim();
    if (!url) return;
    dropBusy = true;
    try {
      if (url.includes("open.spotify.com") || url.startsWith("spotify:")) {
        const sp = await api.importSpotify(url);
        if (!sp || !sp.tracks || sp.tracks.length === 0) {
          throw new Error("Không tìm thấy nhạc");
        }
        const items: OnlineTrack[] = sp.tracks.map((t) => ({
          title: t.title,
          artist: t.artist,
          album: t.album || sp.title,
          duration: t.duration,
          thumbnail: t.thumbnail || sp.thumbnail || null,
          url: t.url || url,
          source: "spotify",
        }));
        const first = items[0];
        const query = `${first.artist} - ${first.title} official audio`.trim();
        const ytMatches = await api.searchOnline(query, "youtube", 1);
        if (!ytMatches || ytMatches.length === 0) {
          throw new Error("Không tìm thấy nhạc");
        }
        const matchUrl = ytMatches[0].url;
        const streamUrl = await api.resolveStream(matchUrl);
        if (!streamUrl) {
          throw new Error("Không tìm thấy nhạc");
        }
        first.url = matchUrl;
        if (!first.thumbnail && ytMatches[0].thumbnail) {
          first.thumbnail = ytMatches[0].thumbnail;
        }
        onlineResults = items;
        player.playOnlineList(items, 0);
        showDropNotice(`Đang phát: ${first.title} — ${first.artist}`, "ok");
      } else if (/\.(mp3|flac|wav|ogg|m4a|aac|opus|webm)(\?.*)?$/i.test(url)) {
        const dur = await probeDirectAudio(url);
        const fileName = decodeURIComponent(url.split("/").pop()?.split("?")[0] || "Bản nhạc trực tuyến")
          .replace(/\.(mp3|flac|wav|ogg|m4a|aac|opus|webm)$/i, "");
        const item: OnlineTrack = {
          title: fileName,
          artist: "Nguồn trực tiếp",
          album: "",
          duration: dur,
          thumbnail: null,
          url,
          source: "youtube",
        };
        onlineResults = [item];
        player.playOnlineList([item], 0);
        showDropNotice(`Đang phát: ${item.title}`, "ok");
      } else if (url.startsWith("http://") || url.startsWith("https://")) {
        const isSingleTrackLink =
          !url.toLowerCase().includes("/playlist") && !url.toLowerCase().includes("/sets/");
        const preStreamPromise = isSingleTrackLink
          ? api.resolveStream(url).catch(() => "")
          : Promise.resolve("");
        const resolved = await api.resolveInfo(url);
        if (!resolved || resolved.length === 0) {
          throw new Error("Không tìm thấy nhạc");
        }
        const first = resolved[0];
        const streamUrl = (await preStreamPromise) || (await api.resolveStream(first.url));
        if (!streamUrl) {
          throw new Error("Không tìm thấy nhạc");
        }
        onlineResults = resolved;
        player.playOnlineList(resolved, 0);
        showDropNotice(`Đang phát: ${first.title}`, "ok");
      } else {
        throw new Error("Không tìm thấy nhạc");
      }
    } catch {
      // Do NOT stop or affect currently playing music!
      showDropNotice("Không tìm thấy nhạc từ đường dẫn này — vẫn tiếp tục phát nhạc hiện tại", "warn");
    } finally {
      dropBusy = false;
    }
  }

  function handleWindowDragEnter(e: DragEvent) {
    e.preventDefault();
    dragDepth++;
    isDraggingOver = true;
  }

  function handleWindowDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = "copy";
    }
    if (!isDraggingOver) isDraggingOver = true;
  }

  function handleWindowDragLeave(e: DragEvent) {
    e.preventDefault();
    dragDepth = Math.max(0, dragDepth - 1);
    if (dragDepth === 0 || e.relatedTarget === null) {
      dragDepth = 0;
      isDraggingOver = false;
    }
  }

  async function handleWindowDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragDepth = 0;
    isDraggingOver = false;

    const dt = e.dataTransfer;
    if (!dt) return;

    // 1. Check if dropped local audio or image files
    if (dt.files && dt.files.length > 0) {
      const audioFiles = Array.from(dt.files).filter((f) =>
        /\.(mp3|flac|wav|ogg|m4a|aac|opus|webm)$/i.test(f.name) || f.type.startsWith("audio/"),
      );
      if (audioFiles.length > 0) {
        const localDroppedTracks: Track[] = audioFiles.map((f, idx) => {
          const blobUrl = URL.createObjectURL(f);
          return {
            id: -9000 - idx,
            path: blobUrl,
            title: f.name.replace(/\.[^.]+$/, ""),
            artist: "Tệp kéo thả",
            album: "",
            duration: 0,
            cover_path: null,
            source: "local",
            url: blobUrl,
          };
        });
        player.playList(localDroppedTracks, 0);
        showDropNotice(`Đang phát: ${localDroppedTracks[0].title}`, "ok");
        return;
      }

      const imageFiles = Array.from(dt.files).filter((f) =>
        /\.(png|jpe?g|webp|gif|bmp|avif|svg)$/i.test(f.name) || f.type.startsWith("image/"),
      );
      if (imageFiles.length > 0) {
        const imgFile = imageFiles[0];
        try {
          const ext = imgFile.name.split(".").pop() || "png";
          const buf = await imgFile.arrayBuffer();
          const savedPath = await api.saveBackgroundBytes(ext, Array.from(new Uint8Array(buf)));
          const cleanName = imgFile.name.replace(/\.[^.]+$/, "");
          await theme.setCustomBgImage(savedPath, cleanName, coverSrc(savedPath));
          showDropNotice(`Đã đổi phông nền giao diện: ${cleanName}`, "ok");
        } catch (err) {
          showDropNotice(`Không thể lưu ảnh phông nền: ${err}`, "warn");
        }
        return;
      }
    }

    // 2. Check if dropped URL/link
    const droppedUrl = extractDroppedUrl(dt);
    if (!droppedUrl) {
      showDropNotice("Không tìm thấy nhạc từ nội dung vừa thả — vẫn tiếp tục phát nhạc hiện tại", "warn");
      return;
    }

    if (/\.(png|jpe?g|webp|gif|bmp|avif|svg)(\?.*)?$/i.test(droppedUrl)) {
      try {
        const savedPath = await api.saveBackgroundImage(droppedUrl);
        await theme.setCustomBgImage(savedPath, "Ảnh nền trực tuyến", coverSrc(savedPath));
        showDropNotice("Đã đổi phông nền giao diện từ liên kết ảnh!", "ok");
      } catch {
        await theme.setCustomBgImage(droppedUrl, "Ảnh nền trực tuyến", droppedUrl);
        showDropNotice("Đã đổi phông nền giao diện từ liên kết ảnh!", "ok");
      }
      return;
    }

    await playDroppedLink(droppedUrl);
  }

  async function refresh() {
    const localQuery = active !== "home" && active !== "online" && active !== "settings" ? searchText.trim() : "";
    tracks =
      typeof active === "number"
        ? await api.getPlaylist(active)
        : await api.listTracks(localQuery);
    playlists = await api.listPlaylists();
  }

  let lastPrefetchedUrl = "";
  function handleSearchInput() {
    const q = searchText.trim();
    if (
      (q.startsWith("http://") || q.startsWith("https://") || q.startsWith("spotify:")) &&
      q !== lastPrefetchedUrl
    ) {
      lastPrefetchedUrl = q;
      if (q.includes("open.spotify.com") || q.startsWith("spotify:")) {
        api.importSpotify(q).catch(() => {});
      } else {
        api.resolveInfo(q).catch(() => {});
      }
    }
    if (active !== "home" && active !== "online" && active !== "settings") {
      refresh();
    }
  }

  async function runUnifiedSearch() {
    const q = searchText.trim();
    showSourceMenu = false;

    // Allow instant preset or custom saved code switching right from the unified search bar!
    const isSavedCode = theme.savedProfiles.some(
      (p) => p.code.toUpperCase() === q.toUpperCase(),
    );
    if ((q.toUpperCase().startsWith("DLC-") || isSavedCode) && theme.applyCode(q)) {
      searchText = "";
      return;
    }

    if (searchSource === "local" && !q.startsWith("http://") && !q.startsWith("https://")) {
      if (active === "online" || active === "settings") active = null;
      inspectedTrack = null;
      await refresh();
      return;
    }

    if (!q) {
      if (active !== "online" && active !== "settings") await refresh();
      return;
    }

    // Switch to Khám phá view and run online search / link resolver
    active = "online";
    inspectedTrack = null;
    onlineBusy = true;
    onlineErr = "";
    try {
      if (q.includes("open.spotify.com") || q.startsWith("spotify:")) {
        const sp = await api.importSpotify(q);
        onlineResults = sp.tracks.map((t) => ({
          title: t.title,
          artist: t.artist,
          album: t.album || sp.title,
          duration: t.duration,
          thumbnail: t.thumbnail || sp.thumbnail || null,
          url: t.url || q,
          source: "spotify",
        }));
      } else if (q.startsWith("http://") || q.startsWith("https://")) {
        onlineResults = await api.resolveInfo(q);
      } else {
        const src = searchSource === "local" ? "all" : searchSource;
        onlineResults = await api.searchOnline(q, src);
      }
    } catch (e) {
      onlineErr = String(e);
    } finally {
      onlineBusy = false;
    }
  }

  function chooseSource(src: "all" | "youtube" | "soundcloud" | "local") {
    searchSource = src;
    showSourceMenu = false;
    if (src === "local") {
      if (active === "online") active = null;
      refresh();
    } else if (searchText.trim()) {
      runUnifiedSearch();
    }
  }

  async function addFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== "string") return;
    scan = { done: 0, total: 0 };
    try {
      await api.scanFolder(dir);
    } finally {
      scan = null;
      await refresh();
    }
  }

  async function select(id: number | "home" | "online" | "settings" | null) {
    active = id;
    inspectedTrack = null;
    showLyrics = false;
    tempFullscreen = false;
    if (typeof localStorage !== "undefined") {
      try {
        localStorage.setItem("dlc.last.active", String(id));
      } catch {}
    }
    await refresh();
  }

  async function makePlaylist() {
    const n = newName.trim();
    if (!n) return;
    newName = "";
    await api.createPlaylist(n);
    await refresh();
  }

  async function toggleLikeTrack(t: Track) {
    let targetTrack: Track | undefined;
    if (t.id > 0) {
      targetTrack = tracks.find((item) => item.id === t.id) || t;
    } else {
      const u = t.url || t.path;
      const existing = tracks.find((item) => item.url === u || item.path === u);
      if (existing) {
        targetTrack = existing;
      } else {
        const onlineTrk: OnlineTrack = {
          title: t.title,
          artist: t.artist,
          album: t.album || "",
          thumbnail: t.cover_path || "",
          duration: t.duration,
          source: (t.source || "youtube") as any,
          url: u,
        };
        targetTrack = await api.addOnlineToLibrary(onlineTrk);
      }
    }

    if (targetTrack && targetTrack.id > 0) {
      const liked = await api.toggleLike(targetTrack.id);
      targetTrack.liked = liked;
      t.liked = liked;
      if (player.current && (player.current.id === targetTrack.id || (player.current.url && player.current.url === targetTrack.url))) {
        player.current.liked = liked;
      }
      if (inspectedTrack && (inspectedTrack.id === targetTrack.id || (inspectedTrack.url && inspectedTrack.url === targetTrack.url))) {
        inspectedTrack.liked = liked;
      }
      await refresh();
    }
  }

  async function deleteTrack(t: Track) {
    const ok = await ask(`Xóa "${t.title}" khỏi thư viện?\n(File nhạc trên máy được giữ nguyên.)`, {
      title: "Xác nhận xóa",
      kind: "warning",
    });
    if (!ok) return;
    await api.deleteTrack(t.id);
    if (inspectedTrack?.id === t.id) {
      inspectedTrack = null;
    }
    const qi = player.queue.findIndex((q) => q.id === t.id);
    if (qi >= 0) player.removeFromQueue(qi);
    await refresh();
  }

  async function startDownloadOnline(o: OnlineTrack, existingId?: number) {
    downloads = { ...downloads, [o.url]: 0 };
    try {
      const updated = await api.downloadTrack(o, existingId ?? null);
      player.queue = player.queue.map((q) =>
        q.url === o.url || q.id === updated.id ? updated : q,
      );
      if (inspectedTrack && (inspectedTrack.url === o.url || inspectedTrack.id === updated.id)) {
        inspectedTrack = updated;
      }
      await refresh();
    } catch (e) {
      player.error = `Lỗi tải "${o.title}": ${e}`;
    } finally {
      const copy = { ...downloads };
      delete copy[o.url];
      downloads = copy;
    }
  }

  async function startDownloadFromLibrary(t: Track) {
    const url = t.url || t.path;
    await startDownloadOnline(
      {
        title: t.title,
        artist: t.artist,
        album: t.album,
        duration: t.duration,
        thumbnail: t.cover_path,
        url,
        source: t.source,
      },
      t.id,
    );
  }

  async function changeLogo(id: number) {
    const f = await open({
      multiple: false,
      filters: [{ name: "Ảnh", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }],
    });
    if (typeof f !== "string") return;
    try {
      await api.setPlaylistCover(id, f);
    } catch (e) {
      player.error = String(e);
    }
    await refresh();
  }

  async function removeLogo(id: number) {
    await api.setPlaylistCover(id, null);
    await refresh();
  }

  async function rename(id: number, name: string) {
    const n = name.trim();
    if (!n || n === activePl?.name) return;
    await api.renamePlaylist(id, n);
    await refresh();
  }

  const activePl = $derived(
    typeof active === "number" ? (playlists.find((p) => p.id === active) ?? null) : null,
  );

  const totalSeconds = $derived(tracks.reduce((acc, t) => acc + (t.duration || 0), 0));

  const sourceShortLabel = $derived(
    searchSource === "youtube"
      ? "YT"
      : searchSource === "soundcloud"
        ? "SC"
        : searchSource === "local"
          ? "MÁY"
          : "TẤT CẢ",
  );

  async function delPlaylist(id: number) {
    await api.deletePlaylist(id);
    if (active === id) active = null;
    await refresh();
  }

  function toggleInspectCurrent() {
    const current = player.current;
    if (!current) return;
    const isInspectingCurrent =
      (inspectedTrack?.url || inspectedTrack?.path) === (current.url || current.path);
    inspectedTrack = isInspectingCurrent ? null : current;
  }

  function handleDragMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, a, [role='button'], [role='slider']")) return;
    api.windowAction("drag");
  }

  function buildIslandStatePayload() {
    const cur = player.current;
    let currentLyricText = "";
    if (islandLyrics && islandLyrics.lines.length > 0) {
      if (islandLyrics.synced) {
        const idx = activeIndex(islandLyrics.lines, player.position + 0.15);
        if (idx >= 0) currentLyricText = islandLyrics.lines[idx]?.text || "";
      } else {
        currentLyricText = islandLyrics.lines[0]?.text || "";
      }
    }
    return {
      title: cur?.title || "Trantramy - Tran Tra My",
      artist: cur?.artist || "Sẵn sàng phát nhạc",
      album: cur?.album || "",
      cover_path: cur?.cover_path || null,
      url: cur?.url || cur?.path || null,
      playing: player.playing,
      position: player.position,
      duration: player.duration,
      volume: player.volume,
      liked: Boolean(cur?.liked),
      lyricLine: currentLyricText,
      hasTrack: Boolean(cur),
    };
  }

  function broadcastIslandState() {
    const payload = buildIslandStatePayload();
    try {
      localStorage.setItem("dlc.island.snapshot.v1", JSON.stringify(payload));
    } catch {
      // ignore
    }
    emit("dlc-island-state", payload).catch(() => {});
  }

  async function toggleDynamicIsland(force?: boolean) {
    try {
      islandVisible = await api.toggleIsland(force, theme.config.stealthMode);
      if (islandVisible) {
        broadcastIslandState();
      }
    } catch {
      // ignore
    }
  }

  async function handleWinMinimizeOrClose(action: "minimize" | "close") {
    if (theme.config.autoIslandOnMinimize) {
      await toggleDynamicIsland(true);
    }
    await api.windowAction(action);
  }

  $effect(() => {
    const cur = player.current;
    if (!cur) {
      islandLyrics = null;
      islandLyricTrackKey = "";
      return;
    }
    const key = `${cur.path}:${cur.title}:${cur.artist}:${lyricsState.revision}`;
    if (key === islandLyricTrackKey) return;
    islandLyricTrackKey = key;
    islandLyrics = null;
    api
      .getLyrics(cur.path || "", cur.title, cur.artist, cur.album || "", cur.duration || 0, cur.url)
      .then((raw) => {
        if (islandLyricTrackKey !== key) return;
        islandLyrics = raw ? parseLrc(raw) : null;
      })
      .catch(() => {});
  });

  $effect(() => {
    // Sync playback & lyric state to Dynamic Island
    void player.current;
    void player.playing;
    void player.position;
    void player.duration;
    void player.volume;
    void islandLyrics;
    broadcastIslandState();
  });

  onMount(() => lyricsState.subscribe());
  onMount(() => {
    theme.init();
    player.init();
    refresh();
    if (theme.config.autoIslandOnMinimize ?? true) {
      toggleDynamicIsland(true);
    }
    const unlisten = [
      listen<{ done: number; total: number }>("scan-progress", (e) => (scan = e.payload)),
      listen<DownloadProgress>("download-progress", (e) => {
        const p = e.payload;
        if (p.status === "downloading") {
          downloads = { ...downloads, [p.url]: p.percent };
        }
      }),
      listen<string>("media-key", (e) => {
        if (e.payload === "play-pause") player.toggle();
        else if (e.payload === "next") player.next();
        else if (e.payload === "prev") player.prev();
      }),
      listen<boolean>("dlc-island-ready", () => {
        broadcastIslandState();
      }),
      listen<boolean>("island-visibility-changed", (e) => {
        islandVisible = Boolean(e.payload);
        if (islandVisible) broadcastIslandState();
      }),
      listen<any>("dlc-theme-updated", (e) => {
        if (e.payload && typeof e.payload === "object") {
          theme.config = e.payload;
          theme.applyToDom();
        }
      }),
      listen<{ action: string; value?: number }>("dlc-island-cmd", (e) => {
        const cmd = e.payload;
        if (!cmd) return;
        if (cmd.action === "play-pause") {
          if (!player.current && tracks.length > 0) {
            player.playList(tracks, 0);
          } else {
            player.toggle();
          }
        } else if (cmd.action === "next") {
          player.next();
        } else if (cmd.action === "prev") {
          player.prev();
        } else if (cmd.action === "seek" && typeof cmd.value === "number") {
          player.seek(cmd.value);
        } else if (cmd.action === "volume" && typeof cmd.value === "number") {
          player.setVolume(cmd.value);
        } else if (cmd.action === "toggle-like" && player.current) {
          toggleLikeTrack(player.current);
        } else if (cmd.action === "focus-main") {
          const w = getCurrentWindow();
          w.show().catch(() => {});
          w.unminimize().catch(() => {});
          w.setFocus().catch(() => {});
        }
      }),
    ];
    return () => unlisten.forEach((p) => p.then((f) => f()));
  });
</script>

<svelte:window
  ondragenter={handleWindowDragEnter}
  ondragover={handleWindowDragOver}
  ondragleave={handleWindowDragLeave}
  ondrop={handleWindowDrop}
  onkeydown={(e) => {
    if (e.key === "Escape") {
      showQueue = false;
      showLyrics = false;
      showSourceMenu = false;
    }
  }}
/>

{#if isDraggingOver}
  <div class="drop-overlay" role="presentation">
    <div class="drop-card">
      <div class="drop-icon">
        <AppSpeakerLogo size={56} />
      </div>
      <h3>Kéo thả liên kết bài hát hoặc ảnh nền</h3>
      <p>Hỗ trợ liên kết YouTube, SoundCloud, Spotify, tệp âm thanh hoặc hình nền (PNG, JPG, WEBP, GIF)</p>
    </div>
  </div>
{/if}

{#if theme.config.bgImage}
  <div class="discord-wallpaper-layer" aria-hidden="true">
    <img
      class="discord-wallpaper-img"
      class:is-blurred={(theme.config.bgImageBlur ?? 0) > 0}
      src={coverSrc(theme.config.bgImage)}
      alt=""
    />
    <div class="discord-wallpaper-tint"></div>
  </div>
{/if}

<!-- Frameless Top Drag Strip + Auto-Hide Proximity Window Controls -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="frameless-top-zone"
  onmousedown={handleDragMouseDown}
  ondblclick={() => api.windowAction("maximize")}
>
  <div class="win-proximity-zone">
    <div
      class="win-controls"
      class:always-visible={theme.config.winControlsMode === "always"}
      role="group"
      aria-label="Điều khiển cửa sổ"
    >
      <button
        class="win-btn min"
        onclick={() => handleWinMinimizeOrClose("minimize")}
        title="Thu nhỏ"
        aria-label="Thu nhỏ cửa sổ"
      >
        <Minus size={12} />
      </button>

      <button
        class="win-btn max"
        onclick={() => api.windowAction("maximize")}
        title="Phóng to / Thu nhỏ"
        aria-label="Phóng to hoặc thu nhỏ"
      >
        <Square size={12} />
      </button>

      <button
        class="win-btn close"
        onclick={() => handleWinMinimizeOrClose("close")}
        title="Đóng ứng dụng"
        aria-label="Đóng ứng dụng"
      >
        <X size={12} />
      </button>
    </div>
  </div>
</div>

<div class="studio-shell">
  <!-- Navigation Rail -->
  <aside class="sidebar">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="brand" onmousedown={handleDragMouseDown}>
      <AppSpeakerLogo
        size={42}
        interactive
        syncNativeIcon
        onclick={() => select("settings")}
        title="Biểu tượng loa Trantramy — Tự động đổi màu sắc & họa tiết theo cài đặt giao diện"
      />
      <div class="brand-text">
        <h1>Trantramy</h1>
        <span>Tran Tra My</span>
      </div>
    </div>

    <!-- Unified Search Bar right above "Thư viện" with 3-Line Source Selector -->
    <div class="sidebar-search-wrap">
      <form
        class="sidebar-search"
        onsubmit={(e) => {
          e.preventDefault();
          runUnifiedSearch();
        }}
      >
        <button type="submit" class="s-icon-btn" title="Tìm kiếm" aria-label="Tìm kiếm">
          {#if onlineBusy}
            <span class="s-spinner"></span>
          {:else}
            <Search size={14} />
          {/if}
        </button>

        <input
          class="s-input"
          placeholder="Tìm nhạc, dán link…"
          bind:value={searchText}
          oninput={handleSearchInput}
        />

        {#if searchText}
          <button
            type="button"
            class="s-clear"
            onclick={() => {
              searchText = "";
              handleSearchInput();
            }}
            title="Xóa tìm kiếm"
            aria-label="Xóa tìm kiếm"
          >
            <X size={14} />
          </button>
        {/if}

        <!-- 3-Horizontal-Lines Source Switcher Icon Button -->
        <button
          type="button"
          class="s-filter-btn"
          class:open={showSourceMenu}
          onclick={() => (showSourceMenu = !showSourceMenu)}
          title="Đổi nguồn tìm kiếm ({sourceShortLabel})"
          aria-label="Đổi nguồn tìm kiếm"
        >
          <Menu size={15} />
        </button>
      </form>

      {#if showSourceMenu}
        <div class="source-dropdown">
          <div class="sd-title">Nguồn tìm kiếm</div>
          <button
            type="button"
            class="sd-item"
            class:sel={searchSource === "all"}
            onclick={() => chooseSource("all")}
          >
            <span>Tất cả (YouTube & SoundCloud)</span>
            {#if searchSource === "all"}<Check size={12} />{/if}
          </button>
          <button
            type="button"
            class="sd-item"
            class:sel={searchSource === "youtube"}
            onclick={() => chooseSource("youtube")}
          >
            <span>YouTube</span>
            {#if searchSource === "youtube"}<Check size={12} />{/if}
          </button>
          <button
            type="button"
            class="sd-item"
            class:sel={searchSource === "soundcloud"}
            onclick={() => chooseSource("soundcloud")}
          >
            <span>SoundCloud</span>
            {#if searchSource === "soundcloud"}<Check size={12} />{/if}
          </button>
          <button
            type="button"
            class="sd-item"
            class:sel={searchSource === "local"}
            onclick={() => chooseSource("local")}
          >
            <span>Thư viện trên máy</span>
            {#if searchSource === "local"}<Check size={12} />{/if}
          </button>
        </div>
      {/if}
    </div>

    <nav class="nav-group" aria-label="Điều hướng chính">
      <button class="nav-item" class:active={active === "home" && !inspectedTrack} onclick={() => select("home")}>
        <House size={17} />
        <span>Trang chủ</span>
      </button>

      <button class="nav-item" class:active={active === null && !inspectedTrack} onclick={() => select(null)}>
        <Music2 size={17} />
        <span>Thư viện</span>
      </button>

      <button class="nav-item" class:active={active === "online" && !inspectedTrack} onclick={() => select("online")}>
        <Globe size={17} />
        <span>Khám phá</span>
      </button>

      <button class="nav-item" class:active={active === "settings" && !inspectedTrack} onclick={() => select("settings")}>
        <Settings size={17} />
        <span>Cài đặt</span>
      </button>
    </nav>

    <button class="import-btn" onclick={addFolder}>
      <FolderPlus size={16} />
      <span>Thêm thư mục nhạc</span>
    </button>

    <div class="section-label">
      <span>DANH SÁCH PHÁT</span>
      <div class="pl-actions-top">
        <button
          type="button"
          class="spotify-pill-btn"
          onclick={() => (showSpotifyImport = true)}
          title="Nhập danh sách phát từ Spotify"
          aria-label="Nhập danh sách phát từ Spotify"
        >
          <ListMusic size={12} />
          <span>+ Spotify</span>
        </button>
        <span class="count-num">{playlists.length}</span>
      </div>
    </div>

    <div class="playlist-list">
      {#each playlists as p (p.id)}
        <div class="pl-row" class:active={active === p.id && !inspectedTrack}>
          <button class="pl-btn" onclick={() => select(p.id)}>
            {#if p.cover_path}
              <img class="pl-thumb" src={coverSrc(p.cover_path)} alt="" />
            {:else}
              <span class="pl-thumb ph" aria-hidden="true">
                <Music2 size={13} />
              </span>
            {/if}
            <span class="pl-name">{p.name}</span>
            <span class="pl-count">{p.count}</span>
          </button>
          <button class="pl-del" onclick={() => delPlaylist(p.id)} title="Xóa playlist" aria-label="Xóa playlist {p.name}">
            <X size={13} />
          </button>
        </div>
      {/each}
    </div>

    <form
      class="new-pl-form"
      onsubmit={(e) => {
        e.preventDefault();
        makePlaylist();
      }}
    >
      <Plus size={14} />
      <input placeholder="Tạo danh sách phát mới…" bind:value={newName} />
    </form>
  </aside>

  <!-- Main Stage -->
  <main class="stage" class:stage-lyrics={showLyrics && !isFullscreenLyrics}>
    {#if showLyrics && !isFullscreenLyrics}
      <KaraokeRoom
        embedded={true}
        onclose={() => { showLyrics = false; tempFullscreen = false; }}
        ontogglemaximize={() => { tempFullscreen = true; }}
        onsettings={openLyricsSettings}
      />
    {:else}
      {#if !inspectedTrack}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <header class="stage-header" onmousedown={handleDragMouseDown}>
        {#if active === "home"}
          <div class="hero-block">
            <span class="kicker">ĐỀ XUẤT & KHÁM PHÁ</span>
            <div class="hero-title-row">
              <h2>Trang chủ</h2>
            </div>
          </div>
        {:else if active === null}
          <div class="hero-block">
            <span class="kicker">BỘ SƯU TẬP BÀI HÁT</span>
            <div class="hero-title-row">
              <h2>Thư viện</h2>
              <span class="stats-pill">
                {tracks.length} bài · {fmtTime(totalSeconds)}
              </span>
            </div>
          </div>
        {:else if active === "online"}
          <div class="hero-block">
            <span class="kicker">ÂM NHẠC TRỰC TUYẾN</span>
            <div class="hero-title-row">
              <h2>Khám phá</h2>
            </div>
          </div>
        {:else if active === "settings"}
          <div class="hero-block">
            <span class="kicker">TÙY BIẾN & THIẾT LẬP</span>
            <div class="hero-title-row">
              <h2>Cài đặt</h2>
            </div>
          </div>
        {:else if activePl}
          <div class="playlist-hero">
            <button
              class="pl-cover-upload"
              onclick={() => changeLogo(activePl.id)}
              title="Đổi ảnh bìa danh sách phát"
            >
              {#if activePl.cover_path}
                <img src={coverSrc(activePl.cover_path)} alt="Cover" />
              {:else}
                <div class="upload-hint">
                  <Image size={24} />
                  <small>Đổi ảnh bìa</small>
                </div>
              {/if}
            </button>

            <div class="pl-hero-meta">
              <span class="kicker">DANH SÁCH PHÁT</span>
              <input
                class="pl-title-input"
                value={activePl.name}
                title="Bấm để đổi tên danh sách phát"
                onblur={(e) => rename(activePl.id, e.currentTarget.value)}
                onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()}
              />
              <div class="pl-sub-row">
                <span class="stats-pill">{tracks.length} bài · {fmtTime(totalSeconds)}</span>
                {#if activePl.cover_path}
                  <button class="text-btn" onclick={() => removeLogo(activePl.id)}>Xóa ảnh bìa</button>
                {/if}
              </div>
            </div>
          </div>
        {/if}
      </header>
    {/if}

    {#if scan || player.error || dropBusy || dropNotice}
      <div class="status-strip">
        {#if scan}
          <div class="scan-pill">
            <span class="mini-pulse"></span>
            <span>Đang quét thư mục nhạc: <strong>{scan.done}</strong> / {scan.total}</span>
          </div>
        {/if}
        {#if dropBusy}
          <div class="scan-pill">
            <span class="mini-pulse"></span>
            <span>Đang kiểm tra đường dẫn nhạc vừa thả…</span>
          </div>
        {/if}
        {#if dropNotice}
          <div class={dropNotice.kind === "warn" ? "error-pill" : "scan-pill"}>
            <span>{dropNotice.text}</span>
            <button onclick={() => (dropNotice = null)} aria-label="Đóng thông báo"><X size={14} /></button>
          </div>
        {/if}
        {#if player.error}
          <div class="error-pill">
            <span>{player.error}</span>
            <button onclick={() => (player.error = "")} aria-label="Đóng thông báo lỗi"><X size={14} /></button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="stage-content">
      {#if inspectedTrack}
        <TrackDetailView
          track={inspectedTrack}
          onlyricssettings={openLyricsSettings}
          contextTracks={tracks}
          {playlists}
          {downloads}
          onclose={() => (inspectedTrack = null)}
          ondelete={deleteTrack}
          ondownload={startDownloadFromLibrary}
          onadd={async (p, t) => {
            await api.addToPlaylist(p, t);
            await refresh();
          }}
        />
      {:else if active === "home"}
        <HomeView
          libraryTracks={tracks}
          onplaytrack={(t) => {
            const idx = tracks.findIndex((item) => item.id === t.id);
            if (idx >= 0) {
              player.playList(tracks, idx);
            } else {
              player.playList([t], 0);
            }
            inspectedTrack = t;
          }}
          onplayonlinelist={(list, idx) => {
            player.playOnlineList(list, idx);
          }}
          onopeninspector={(t) => (inspectedTrack = t)}
          ondownload={startDownloadOnline}
          ontogglelike={async (t) => {
            const liked = await api.toggleLike(t.id);
            t.liked = liked;
            await refresh();
          }}
          onnavigateto={(target) => {
            if (target === "library") select(null);
            else if (target === "online") select("online");
            else if (target === "settings") select("settings");
          }}
        />
      {:else if active === "online"}
        <OnlinePanel
          libraryTracks={tracks}
          results={onlineResults}
          busy={onlineBusy}
          err={onlineErr}
          {downloads}
          onrefresh={refresh}
          ondownload={(o) => startDownloadOnline(o)}
          onopen={(t) => (inspectedTrack = t)}
          onclear={() => (onlineResults = [])}
          onloadmore={loadMoreSearch}
          loadingMore={loadingMoreSearch}
        />
      {:else if active === "settings"}
        {#key lyricsSettingsRevision}
        <SettingsPanel
          onopenequalizer={() => (showEqualizer = true)}
          onopenspotify={() => (showSpotifyImport = true)}
          onopenkaraoke={() => (showLyrics = true)}
          ontoggleisland={() => toggleDynamicIsland()}
          islandActive={islandVisible}
        />
        {/key}
      {:else}
        <TrackList
          {tracks}
          {playlists}
          {downloads}
          activePlaylist={typeof active === "number" ? active : null}
          ontogglelike={toggleLikeTrack}
          ondelete={deleteTrack}
          ondownload={startDownloadFromLibrary}
          onopen={(t) => (inspectedTrack = t)}
          onadd={async (p, t) => {
            await api.addToPlaylist(p, t);
            await refresh();
          }}
          onremove={async (t) => {
            if (typeof active === "number") {
              await api.removeFromPlaylist(active, t);
              await refresh();
            }
          }}
        />
      {/if}
    </div>
    {/if}
  </main>

  <!-- Queue Drawer -->
  {#if showQueue}
    <aside class="queue-drawer" aria-label="Hàng chờ phát">
      <header class="drawer-head">
        <div>
          <span class="kicker">TIẾP THEO</span>
          <h3>Hàng chờ phát ({player.queue.length})</h3>
        </div>
        <div class="drawer-head-actions">
          {#if player.queue.length > 0}
            <button
              type="button"
              class="drawer-clear-btn"
              onclick={() => player.clearQueue()}
              title="Xóa toàn bộ hàng chờ"
            >
              Xóa hết
            </button>
          {/if}
          <button
            type="button"
            class="drawer-close-btn"
            onclick={() => (showQueue = false)}
            title="Đóng hàng chờ"
            aria-label="Đóng hàng chờ"
          >
            <X size={14} />
          </button>
        </div>
      </header>

      <div class="drawer-body">
        {#if player.queue.length === 0}
          <p class="drawer-empty">Chưa có bài hát nào trong hàng chờ.</p>
        {:else}
          {#each player.queue as t, i (t.id + ":" + i)}
            {@const isCur = i === player.index}
            <div class="q-item" class:cur={isCur}>
              <span class="q-idx">{String(i + 1).padStart(2, "0")}</span>
              <button
                class="q-main"
                onclick={() => {
                  player.jumpTo(i);
                  inspectedTrack = t;
                }}
              >
                <span class="q-title">{t.title}</span>
                <span class="q-artist">{t.artist}</span>
              </button>
              <button
                class="q-rm"
                onclick={() => player.removeFromQueue(i)}
                title="Xóa khỏi hàng chờ"
                aria-label="Xóa khỏi hàng chờ"
              >
                <X size={13} />
              </button>
            </div>
          {/each}
        {/if}
      </div>
    </aside>
  {/if}

  <div class="deck-slot">
    <PlayerBar
      {showQueue}
      {showLyrics}
      {showEqualizer}
      showIsland={islandVisible}
      inspecting={Boolean(inspectedTrack)}
      oninspect={toggleInspectCurrent}
      onequalizer={() => (showEqualizer = !showEqualizer)}
      onisland={() => toggleDynamicIsland()}
      onqueue={() => {
        showQueue = !showQueue;
        if (showQueue) showLyrics = false;
      }}
      onlyrics={toggleLyrics}
      ontogglelike={toggleLikeTrack}
      onplayrandom={() => {
        if (tracks.length > 0) {
          player.playRandom(tracks);
        }
      }}
      onplaydefault={() => {
        if (tracks.length > 0) {
          player.playList(tracks, 0);
        }
      }}
    />
  </div>

  {#if showLyrics && isFullscreenLyrics}
    <KaraokeRoom
      embedded={false}
      onclose={() => { showLyrics = false; tempFullscreen = false; }}
      ontogglemaximize={() => { tempFullscreen = false; }}
      onsettings={openLyricsSettings}
    />
  {/if}

  <!-- Equalizer 10-band & Auto-Normalizer Modal -->
  <EqualizerModal bind:open={showEqualizer} />

  <!-- Spotify Playlist / Album / Track Import Modal -->
  <SpotifyImportModal
    bind:open={showSpotifyImport}
    onimported={async (plId, _name) => {
      await refresh();
      select(plId);
    }}
  />
</div>

<style>
  /* Frameless Top Bar & Proximity Auto-Hiding Window Controls */
  .frameless-top-zone {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 32px;
    z-index: 100;
    background: var(--bg-sidebar, var(--bg-canvas));
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    justify-content: flex-end;
    pointer-events: auto;
  }

  .win-proximity-zone {
    width: 132px;
    height: 32px;
    display: flex;
    justify-content: flex-end;
    align-items: flex-start;
    padding: 2px 6px;
  }

  .win-controls {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px;
    border-radius: 999px;
    background: var(--bg-glass, oklch(0.11 0.016 242 / 0.92));
    border: 1px solid var(--border-subtle);
    backdrop-filter: blur(14px);
    box-shadow: 0 6px 20px oklch(0 0 0 / 0.65);
    /* Hidden until cursor approaches */
    opacity: 0;
    transform: translateY(-6px) scale(0.9);
    pointer-events: none;
    transition:
      opacity 0.2s var(--ease-out-expo),
      transform 0.2s var(--ease-out-expo);
  }

  .win-proximity-zone:hover .win-controls,
  .win-proximity-zone:focus-within .win-controls,
  .win-controls.always-visible {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }

  .win-btn {
    width: 34px;
    height: 24px;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--text-secondary);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: background 0.15s ease, color 0.15s ease, transform 0.15s ease;
  }
  .win-btn :global(svg) {
    width: clamp(9.5px, 1.15vw, 12px);
    height: clamp(9.5px, 1.15vw, 12px);
  }
  .win-btn:hover {
    background: var(--bg-surface-3);
    color: var(--text-primary);
  }
  .win-btn.close:hover {
    background: var(--danger);
    color: #fff;
  }

  /* Discord-style Custom Wallpaper Layer */
  .discord-wallpaper-layer {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    max-width: 100vw;
    max-height: 100vh;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
    contain: strict;
    background: #06080d;
  }
  .discord-wallpaper-img {
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    display: block;
    object-fit: var(--bg-image-fit, cover);
    object-position: var(--bg-image-pos, center);
    filter: blur(var(--bg-image-blur, 0px))
      brightness(var(--bg-image-brightness, 1))
      saturate(var(--bg-image-saturation, 1.08));
    transition: filter 0.2s ease, transform 0.2s ease;
  }
  .discord-wallpaper-img.is-blurred {
    transform: scale(1.06);
  }
  .discord-wallpaper-tint {
    position: absolute;
    inset: 0;
    background: var(--bg-overlay-tint, var(--bg-canvas));
    opacity: var(--bg-image-dim, 0.48);
    transition: opacity 0.18s ease, background 0.2s ease;
  }

  /* Responsive App Shell */
  .studio-shell {
    position: relative;
    z-index: 1;
    display: grid;
    --deck-height: 90px;
    --stage-gutter: clamp(14px, 2vw, 26px);
    grid-template-columns: clamp(176px, 19vw, 224px) minmax(0, 1fr) auto;
    grid-template-rows: minmax(0, 1fr) auto;
    width: 100vw;
    max-width: 100vw;
    height: 100vh;
    padding-top: 32px;
    overflow: hidden;
    background: var(--bg-app, var(--bg-canvas));
  }
  .deck-slot {
    grid-column: 1 / -1;
    grid-row: 2;
    min-width: 0;
  }

  /* Sidebar */
  .sidebar {
    background: var(--bg-sidebar, var(--bg-canvas));
    backdrop-filter: blur(14px) saturate(135%);
    border-right: 1px solid var(--border-subtle);
    padding: clamp(14px, 1.8vw, 20px) clamp(10px, 1.2vw, 14px) 14px;
    display: flex;
    flex-direction: column;
    gap: 9px;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
  }
  .sidebar > :not(.playlist-list) {
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 6px 6px;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .brand-text h1 {
    margin: 0;
    font-size: clamp(14px, 1.5vw, 16px);
    font-weight: 800;
    letter-spacing: -0.025em;
    color: var(--text-primary);
    line-height: 1.15;
  }
  .brand-text span {
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 600;
    letter-spacing: 0.12em;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Unified Search Bar above "Thư viện" */
  .sidebar-search-wrap {
    position: relative;
    margin-bottom: 2px;
  }
  .sidebar-search {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 36px;
    padding: 0 5px 0 8px;
    border-radius: var(--ui-radius-sm, 10px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    transition: border-color 0.16s ease, background 0.16s ease;
  }
  .sidebar-search:focus-within {
    border-color: var(--accent);
    background: var(--bg-surface-2);
  }
  .s-icon-btn {
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    flex: none;
  }
  .s-icon-btn:hover {
    color: var(--accent);
  }
  .s-spinner {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid oklch(0.7 0.17 238 / 0.25);
    border-top-color: var(--accent);
    animation: spin 0.75s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .s-input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12.5px;
    outline: none;
  }
  .s-input::placeholder {
    color: var(--text-muted);
  }
  .s-clear {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    padding: 2px 4px;
  }
  .s-clear:hover {
    color: var(--text-primary);
  }

  /* 3-Horizontal-Lines Source Filter Button */
  .s-filter-btn {
    position: relative;
    display: inline-grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border-radius: 6px;
    border: 1px solid transparent;
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    cursor: pointer;
    flex: none;
    transition: all 0.15s ease;
  }
  .s-filter-btn:hover,
  .s-filter-btn.open {
    color: var(--accent);
    border-color: oklch(0.7 0.17 238 / 0.35);
    background: var(--accent-soft);
  }

  .source-dropdown {
    position: absolute;
    top: 40px;
    left: 0;
    right: 0;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 6px;
    box-shadow: 0 14px 32px oklch(0 0 0 / 0.7);
    z-index: 50;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .sd-title {
    font-size: 9.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    padding: 3px 8px 5px;
    border-bottom: 1px solid var(--border-subtle);
    margin-bottom: 2px;
  }
  .sd-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
  }
  .sd-item:hover {
    background: var(--bg-surface-3);
    color: var(--text-primary);
  }
  .sd-item.sel {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .sd-item.sel :global(svg) {
    color: var(--accent);
  }

  .nav-group {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 36px;
    padding: 0 11px;
    border-radius: var(--ui-radius-sm, 10px);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .nav-item:hover {
    color: var(--text-primary);
    background: var(--bg-surface-1);
  }
  .nav-item.active {
    color: var(--text-primary);
    background: var(--bg-surface-2);
    border-color: oklch(0.7 0.17 238 / 0.3);
  }
  .nav-item.active :global(svg) {
    color: var(--accent);
  }

  .import-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 36px;
    border-radius: var(--ui-radius-sm, 10px);
    border: 1px dashed var(--border-strong);
    background: transparent !important;
    box-shadow: none !important;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.18s ease;
    margin: 2px 0 4px;
  }
  .import-btn:hover {
    border-style: solid;
    border-color: var(--accent);
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent) !important;
    box-shadow: none !important;
  }

  .section-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px 2px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-muted);
  }
  .count-num {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .pl-actions-top {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .spotify-pill-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 9.5px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-soft);
    border: 1px solid var(--border-strong);
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .spotify-pill-btn:hover {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    border-color: var(--accent);
    transform: scale(1.04);
  }

  .playlist-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 44px;
  }
  .pl-row {
    display: flex;
    align-items: center;
    border-radius: 8px;
    transition: background 0.15s ease;
  }
  .pl-row:hover {
    background: var(--bg-surface-1);
  }
  .pl-row.active {
    background: var(--bg-surface-2);
  }
  .pl-btn {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 12.5px;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
  }
  .pl-row.active .pl-btn {
    color: var(--accent);
    font-weight: 600;
  }
  .pl-thumb {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    object-fit: cover;
    flex: none;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    display: inline-grid;
    place-items: center;
    color: var(--accent);
  }
  .pl-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pl-count {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10.5px;
    color: var(--text-muted);
  }
  .pl-del {
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-muted);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
    margin-right: 4px;
    transition: all 0.15s ease;
  }
  .pl-row:hover .pl-del,
  .pl-row:focus-within .pl-del {
    opacity: 1;
  }
  .pl-del:hover {
    color: var(--danger);
    background: oklch(0.67 0.19 25 / 0.14);
  }

  .new-pl-form {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px;
    border-radius: var(--ui-radius-sm, 10px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
  }
  .new-pl-form:focus-within {
    border-color: var(--accent);
    color: var(--accent);
  }
  .new-pl-form input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    outline: none;
  }
  .new-pl-form input::placeholder {
    color: var(--text-muted);
  }

  /* Main Stage */
  .stage {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: radial-gradient(ellipse 80% 35% at 50% -8%, var(--stage-glow, transparent), transparent);
    container: stage / inline-size;
    position: relative;
  }
  .stage.stage-lyrics {
    padding: 0;
    overflow: hidden;
  }

  .stage-header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    padding: 20px var(--stage-gutter) 16px;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .hero-block {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .kicker {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.1em;
    color: var(--accent);
  }
  .hero-title-row {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
  }
  .hero-title-row h2 {
    margin: 0;
    font-size: clamp(20px, 2.4vw, 26px);
    font-weight: 800;
    letter-spacing: -0.03em;
    color: var(--text-primary);
  }
  .stats-pill {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    padding: 3px 9px;
    border-radius: 999px;
  }

  /* Playlist Hero */
  .playlist-hero {
    min-width: 0;
    max-width: 100%;
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .pl-cover-upload {
    width: clamp(64px, 7.5vw, 82px);
    height: clamp(64px, 7.5vw, 82px);
    border-radius: var(--ui-radius, 14px);
    overflow: hidden;
    border: 1px dashed var(--border-strong);
    background: var(--bg-surface-1);
    color: var(--text-muted);
    padding: 0;
    cursor: pointer;
    flex: none;
    transition: border-color 0.18s ease;
  }
  .pl-cover-upload:hover {
    border-color: var(--accent);
    color: var(--accent);
  }
  .pl-cover-upload img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .upload-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 100%;
  }
  .upload-hint small {
    font-size: 9.5px;
    font-weight: 600;
  }

  .pl-hero-meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .pl-title-input {
    width: 100%;
    min-width: 0;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: clamp(18px, 2.2vw, 24px);
    font-weight: 800;
    letter-spacing: -0.025em;
    padding: 2px 8px;
    margin-left: -8px;
    border-radius: 8px;
    outline: none;
  }
  .pl-title-input:hover,
  .pl-title-input:focus {
    background: var(--bg-surface-1);
    border-color: var(--border-subtle);
  }
  .pl-sub-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .text-btn {
    border: none;
    background: transparent;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    text-decoration: underline;
  }
  .text-btn:hover {
    color: var(--danger);
  }

  /* Status Strip */
  .status-strip {
    display: flex;
    gap: 10px;
    padding: 0 var(--stage-gutter) 10px;
    flex-wrap: wrap;
  }
  .scan-pill,
  .error-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 12px;
  }
  .scan-pill {
    background: var(--accent-soft);
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
  .mini-pulse {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1s infinite;
  }
  .error-pill {
    background: oklch(0.67 0.19 25 / 0.14);
    border: 1px solid oklch(0.67 0.19 25 / 0.35);
    color: var(--danger);
  }
  .scan-pill button,
  .error-pill button {
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }

  /* Drag & Drop Overlay */
  .drop-overlay {
    position: fixed;
    inset: 0;
    z-index: 2000;
    background: oklch(0 0 0 / 0.58);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    pointer-events: none;
  }
  .drop-card {
    max-width: 440px;
    width: 100%;
    padding: 28px 24px;
    border-radius: calc(var(--ui-radius, 12px) + 6px);
    background: var(--bg-surface-1);
    border: 2px dashed var(--accent);
    box-shadow: 0 20px 50px oklch(0 0 0 / 0.65), 0 0 28px var(--accent-glow);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .drop-icon {
    width: 58px;
    height: 58px;
    border-radius: 50%;
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    color: var(--accent);
    display: grid;
    place-items: center;
    box-shadow: 0 4px 16px var(--accent-glow);
  }
  .drop-card h3 {
    margin: 4px 0 0;
    font-size: 16px;
    font-weight: 800;
    color: var(--text-primary);
  }
  .drop-card p {
    margin: 0;
    font-size: 12.5px;
    line-height: 1.5;
    color: var(--text-secondary);
  }

  .stage-content {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    scrollbar-gutter: stable;
    padding: 0 var(--stage-gutter) 20px;
  }

  /* Queue Drawer */
  .queue-drawer {
    width: clamp(220px, 24vw, 280px);
    background: var(--bg-surface-1);
    border-left: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .drawer-head {
    padding: 18px 16px 12px;
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .drawer-head h3 {
    margin: 2px 0 0;
    font-size: 14.5px;
    font-weight: 700;
    letter-spacing: -0.015em;
  }
  .drawer-head-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .drawer-clear-btn {
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 8px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .drawer-clear-btn:hover {
    color: var(--danger);
    border-color: oklch(0.67 0.19 25 / 0.4);
    background: oklch(0.67 0.19 25 / 0.12);
  }
  .drawer-close-btn {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    display: inline-grid;
    place-items: center;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .drawer-close-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
  .drawer-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .drawer-empty {
    padding: 24px 12px;
    text-align: center;
    font-size: 12px;
    color: var(--text-muted);
  }
  .q-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 8px;
    transition: background 0.15s ease;
  }
  .q-item:hover {
    background: var(--bg-surface-2);
  }
  .q-item.cur {
    background: var(--accent-soft);
  }
  .q-idx {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    color: var(--text-muted);
    width: 20px;
    text-align: center;
  }
  .q-item.cur .q-idx {
    color: var(--accent);
    font-weight: 700;
  }
  .q-main {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0;
  }
  .q-title {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-item.cur .q-title {
    color: var(--accent);
  }
  .q-artist {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-rm {
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-muted);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
  }
  .q-item:hover .q-rm,
  .q-item:focus-within .q-rm {
    opacity: 1;
  }
  .q-rm:hover {
    color: var(--danger);
  }

  @media (max-width: 1100px) {
    .queue-drawer {
      position: absolute;
      top: 32px;
      right: 0;
      bottom: var(--deck-height);
      z-index: 25;
      max-width: 100%;
      box-shadow: -12px 0 32px oklch(0 0 0 / 0.3);
      width: 320px;
    }
  }
  @media (max-width: 1000px) {
    .studio-shell {
      --deck-height: 136px;
    }
  }
  @media (max-width: 720px) {
    .studio-shell {
      grid-template-columns: 148px minmax(0, 1fr) auto;
      --stage-gutter: 14px;
    }
    .sidebar {
      padding-inline: 8px;
    }
    .import-btn span {
      font-size: 11px;
    }
    .brand-text span {
      font-size: 7.5px;
    }
    .section-label {
      flex-wrap: wrap;
      gap: 6px;
    }
  }
  @media (max-height: 560px) and (max-width: 1000px) {
    .studio-shell {
      --deck-height: 112px;
    }
    .stage-header {
      padding-block: 10px;
    }
    .stage-header .kicker {
      display: none;
    }
  }
  @container stage (max-width: 500px) {
    .hero-title-row {
      gap: 6px 10px;
    }
  }
</style>
