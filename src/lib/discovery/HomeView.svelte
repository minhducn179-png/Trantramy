<script lang="ts">
  import { RefreshCw, Play, Heart } from "@lucide/svelte";
  import {
    api,
    fmtTime,
    getTrackCover,
    handleImgError,
    type HomeData,
    type OnlineTrack,
    type Track,
  } from "$lib/api";
  import { player } from "$lib/player/player.svelte";
  import WaveformBar from "$lib/audio/WaveformBar.svelte";

  function trackCover(t: Track | null | undefined): string {
    if (!t) return "";
    return getTrackCover(t.cover_path, t.url || t.path, t.title, t.artist);
  }

  let {
    libraryTracks = [],
    onplaytrack,
    onplayonlinelist,
    onopeninspector,
    ondownload,
    ontogglelike,
    onnavigateto,
  }: {
    libraryTracks: Track[];
    onplaytrack: (t: Track) => void;
    onplayonlinelist: (list: OnlineTrack[], idx: number) => void;
    onopeninspector: (t: Track) => void;
    ondownload?: (t: OnlineTrack) => Promise<void>;
    ontogglelike?: (t: Track) => Promise<void>;
    onnavigateto?: (target: "library" | "online" | "settings") => void;
  } = $props();

  let homeData = $state<HomeData | null>(null);
  let loading = $state(true);
  let activeFilter = $state<"all" | "recommended" | "library" | "liked" | "recent">("all");
  let refreshing = $state(false);

  const FILTERS = [
    { id: "all", label: "Tất cả" },
    { id: "recommended", label: "Gợi ý cho bạn" },
    { id: "liked", label: "Bài hát yêu thích" },
    { id: "library", label: "Thư viện" },
    { id: "recent", label: "Nghe gần đây" },
  ] as const;

  async function loadData(force = false) {
    if (force) refreshing = true;
    else loading = true;
    try {
      homeData = await api.getHomeData(force);
    } catch {
      // fallback
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  $effect(() => {
    loadData(false);
  });

  const curTrack = $derived(player.current);

  const likedTracks = $derived.by(() => {
    const list = libraryTracks.filter((t) => t.liked);
    if (list.length > 0) return list;
    return homeData?.liked_tracks || [];
  });

  const recentTracks = $derived.by(() => {
    if (!homeData?.history_recent) return [];
    const seen = new Set<string>();
    const res: { title: string; artist: string; source: string; url: string | null }[] = [];
    for (const h of homeData.history_recent) {
      const key = `${h.title.toLowerCase()}::${h.artist.toLowerCase()}`;
      if (!seen.has(key)) {
        seen.add(key);
        res.push(h);
      }
      if (res.length >= 10) break;
    }
    return res;
  });

  function toTrackFromOnline(o: OnlineTrack, idx: number): Track {
    const matched = libraryTracks.find((t) => t.url === o.url || t.path === o.url);
    if (matched) return matched;
    return {
      id: -1000 - idx,
      path: o.url,
      title: o.title,
      artist: o.artist,
      album: o.album || "YouTube Music",
      duration: o.duration,
      cover_path: o.thumbnail,
      source: o.source || "youtube",
      url: o.url,
      liked: false,
    };
  }

  function playOnlineItem(list: OnlineTrack[], idx: number) {
    onplayonlinelist(list, idx);
    const item = list[idx];
    if (item) {
      onopeninspector(toTrackFromOnline(item, idx));
    }
  }

  function playRecentHistory(h: { title: string; artist: string; source: string; url: string | null }) {
    if (h.url) {
      const existing = libraryTracks.find((t) => t.url === h.url || t.path === h.url);
      if (existing) {
        onplaytrack(existing);
        return;
      }
      const onlTrack: OnlineTrack = {
        title: h.title,
        artist: h.artist,
        album: "YouTube Music",
        thumbnail: "",
        duration: 0,
        source: (h.source || "youtube") as any,
        url: h.url,
      };
      onplayonlinelist([onlTrack], 0);
    } else {
      const match = libraryTracks.find(
        (t) => t.title.toLowerCase() === h.title.toLowerCase() && t.artist.toLowerCase() === h.artist.toLowerCase(),
      );
      if (match) {
        onplaytrack(match);
      }
    }
  }

  // Hero card computed
  const heroOnlineTrack = $derived(homeData?.recommended?.[0] ?? null);
</script>

<div class="yt-home-container">
  <!-- Top Filter Chips (YouTube Style) -->
  <header class="yt-filter-bar">
    <div class="filter-chips">
      {#each FILTERS as f (f.id)}
        <button
          type="button"
          class="chip-btn"
          class:active={activeFilter === f.id}
          onclick={() => (activeFilter = f.id)}
        >
          {f.label}
        </button>
      {/each}
    </div>

    <button
      type="button"
      class="refresh-btn"
      onclick={() => loadData(true)}
      disabled={refreshing}
      title="Làm mới đề xuất"
    >
      <RefreshCw size={16} class={refreshing ? "spin" : ""} />
      <span>Làm mới</span>
    </button>
  </header>

  {#if loading && !homeData}
    <div class="home-loading">
      <div class="yt-spinner"></div>
      <p>Đang tải gợi ý âm nhạc...</p>
    </div>
  {:else}
    <!-- YouTube Music Big Hero Banner (when 'all' or 'recommended') -->
    {#if (activeFilter === "all" || activeFilter === "recommended") && heroOnlineTrack}
      {@const heroTrk = toTrackFromOnline(heroOnlineTrack, 0)}
      <section class="yt-hero-card">
        <div class="hero-bg">
          <img
            src={trackCover(heroTrk)}
            alt=""
            onerror={handleImgError}
          />
        </div>
        <button
          type="button"
          class="hero-art-wrap"
          onclick={() => playOnlineItem(homeData?.recommended || [], 0)}
          title="Phát {heroOnlineTrack.title}"
        >
          <img
            src={trackCover(heroTrk)}
            alt={heroOnlineTrack.title}
            class="hero-art"
            onerror={handleImgError}
          />
          <span class="hero-art-play">
            <Play size={22} />
          </span>
        </button>
        <div class="hero-content">
          <span class="hero-badge">GỢI Ý NỔI BẬT</span>
          <h2 class="hero-title">{heroOnlineTrack.title}</h2>
          <p class="hero-subtitle">
            <span class="hero-artist">{heroOnlineTrack.artist}</span>
            {#if heroOnlineTrack.reason}
              <span class="hero-dot">·</span>
              <span class="hero-reason">{heroOnlineTrack.reason}</span>
            {/if}
          </p>
          <div class="hero-actions">
            <button
              type="button"
              class="hero-play-btn"
              onclick={() => playOnlineItem(homeData?.recommended || [], 0)}
            >
              <Play size={20} />
              <span>Phát ngay</span>
            </button>
            <button
              type="button"
              class="hero-detail-btn"
              onclick={() => onopeninspector(toTrackFromOnline(heroOnlineTrack, 0))}
            >
              Xem chi tiết
            </button>
            <button
              type="button"
              class="hero-like-btn"
              class:liked={heroTrk.liked}
              onclick={() => ontogglelike?.(heroTrk)}
              title={heroTrk.liked ? "Bỏ thích" : "Yêu thích"}
              aria-label="Thích"
            >
              <Heart size={18} fill={heroTrk.liked ? "currentColor" : "none"} />
            </button>
          </div>
        </div>
      </section>
    {/if}

    <!-- 1. Đề xuất cho bạn (YouTube Music Shelves) -->
    {#if (activeFilter === "all" || activeFilter === "recommended") && (homeData?.recommended?.length ?? 0) > 0}
      <section class="yt-shelf">
        <div class="shelf-header">
          <div>
            <span class="shelf-kicker">ÂM NHẠC THEO SỞ THÍCH</span>
            <h3 class="shelf-title">Gợi ý cho bạn</h3>
          </div>
          <button
            type="button"
            class="shelf-more-btn"
            onclick={() => onnavigateto?.("online")}
          >
            Xem tất cả
          </button>
        </div>

        <div class="yt-grid-row">
          {#each (homeData?.recommended || []).slice(0, 10) as item, idx (item.url + idx)}
            {@const trk = toTrackFromOnline(item, idx)}
            {@const isPlayingThis = curTrack && (curTrack.url === item.url || curTrack.path === item.url)}
            <div class="yt-music-card" class:playing={isPlayingThis}>
              <div class="card-thumb-wrap">
                <img
                  src={trackCover(trk)}
                  alt={item.title}
                  class="card-thumb"
                  onerror={handleImgError}
                  loading="lazy"
                />
                {#if isPlayingThis}
                  <div class="playing-overlay">
                    <WaveformBar playing={player.playing} barsCount={4} height={18} />
                  </div>
                {/if}
                <button
                  type="button"
                  class="card-like-fab"
                  class:liked={trk.liked}
                  onclick={(e) => {
                    e.stopPropagation();
                    ontogglelike?.(trk);
                  }}
                  title={trk.liked ? "Bỏ thích" : "Yêu thích"}
                  aria-label="Thích"
                >
                  <Heart size={16} fill={trk.liked ? "currentColor" : "none"} />
                </button>
                <button
                  type="button"
                  class="card-play-fab"
                  onclick={() => playOnlineItem(homeData?.recommended || [], idx)}
                  title="Phát bài này"
                  aria-label="Phát"
                >
                  <Play size={18} />
                </button>
              </div>

              <div class="card-body">
                <button
                  type="button"
                  class="card-title-btn"
                  title={item.title}
                  onclick={() => playOnlineItem(homeData?.recommended || [], idx)}
                >
                  <h4 class="card-title">{item.title}</h4>
                </button>
                <p class="card-artist" title={item.artist}>{item.artist}</p>
                <div class="card-meta">
                  <span class="card-source-tag">{item.source || "youtube"}</span>
                  {#if item.duration}
                    <span class="card-dur">{fmtTime(item.duration)}</span>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- 2. Bài hát đã thích & Đã lưu (Liked & Saved) -->
    {#if (activeFilter === "all" || activeFilter === "liked")}
      <section class="yt-shelf liked-shelf">
        <div class="shelf-header">
          <div>
            <span class="shelf-kicker">DANH SÁCH YÊU THÍCH</span>
            <h3 class="shelf-title">Bài hát đã thích ({likedTracks.length})</h3>
          </div>
          {#if likedTracks.length > 0}
            <button
              type="button"
              class="shelf-more-btn"
              onclick={() => onplaytrack(likedTracks[0])}
            >
              Phát tất cả
            </button>
          {/if}
        </div>

        {#if likedTracks.length === 0}
          <div class="empty-shelf-banner">
            <Heart size={28} />
            <div>
              <p class="empty-title">Chưa có bài hát nào được thích</p>
              <p class="empty-sub">Bấm vào biểu tượng trái tim <Heart size={12} /> ở bài hát bất kỳ để lưu lại vào đây.</p>
            </div>
          </div>
        {:else}
          <div class="yt-grid-row">
            {#each likedTracks.slice(0, 8) as trk (trk.id)}
              {@const isPlayingThis = curTrack && curTrack.id === trk.id}
              <div class="yt-music-card liked-card" class:playing={isPlayingThis}>
                <div class="card-thumb-wrap">
                  <img
                    src={trackCover(trk)}
                    alt={trk.title}
                    class="card-thumb"
                    onerror={handleImgError}
                    loading="lazy"
                  />
                  <div class="liked-badge"><Heart size={12} fill="currentColor" /></div>
                  <button
                    type="button"
                    class="card-play-fab"
                    onclick={() => onplaytrack(trk)}
                    title="Phát bài này"
                    aria-label="Phát"
                  >
                    <Play size={18} />
                  </button>
                </div>
                <div class="card-body">
                  <button
                    type="button"
                    class="card-title-btn"
                    title={trk.title}
                    onclick={() => onplaytrack(trk)}
                  >
                    <h4 class="card-title">{trk.title}</h4>
                  </button>
                  <p class="card-artist">{trk.artist}</p>
                  <div class="card-meta">
                    <span class="card-dur">{fmtTime(trk.duration)}</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {/if}

    <!-- 3. Thư viện của bạn (From Your Library) -->
    {#if (activeFilter === "all" || activeFilter === "library") && libraryTracks.length > 0}
      <section class="yt-shelf">
        <div class="shelf-header">
          <div>
            <span class="shelf-kicker">BỘ SƯU TẬP TRÊN THIẾT BỊ</span>
            <h3 class="shelf-title">Thư viện của bạn ({libraryTracks.length} bài)</h3>
          </div>
          <button
            type="button"
            class="shelf-more-btn"
            onclick={() => onnavigateto?.("library")}
          >
            Quản lý thư viện &gt;
          </button>
        </div>

        <div class="yt-grid-row">
          {#each libraryTracks.slice(0, 10) as trk (trk.id)}
            {@const isPlayingThis = curTrack && curTrack.id === trk.id}
            <div class="yt-music-card" class:playing={isPlayingThis}>
              <div class="card-thumb-wrap">
                <img
                  src={trackCover(trk)}
                  alt={trk.title}
                  class="card-thumb"
                  onerror={handleImgError}
                  loading="lazy"
                />
                {#if isPlayingThis}
                  <div class="playing-overlay">
                    <WaveformBar playing={player.playing} barsCount={4} height={18} />
                  </div>
                {/if}
                <button
                  type="button"
                  class="card-like-fab"
                  class:liked={trk.liked}
                  onclick={(e) => {
                    e.stopPropagation();
                    ontogglelike?.(trk);
                  }}
                  title={trk.liked ? "Bỏ thích" : "Yêu thích (Lưu vào Trang chủ)"}
                  aria-label="Thích"
                >
                  <Heart size={16} fill={trk.liked ? "currentColor" : "none"} />
                </button>
                <button
                  type="button"
                  class="card-play-fab"
                  onclick={() => onplaytrack(trk)}
                  title="Phát bài này"
                  aria-label="Phát"
                >
                  <Play size={18} />
                </button>
              </div>

              <div class="card-body">
                <button
                  type="button"
                  class="card-title-btn"
                  title={trk.title}
                  onclick={() => onplaytrack(trk)}
                >
                  <h4 class="card-title">{trk.title}</h4>
                </button>
                <p class="card-artist" title={trk.artist}>{trk.artist}</p>
                <div class="card-meta">
                  <span class="card-source-tag">{trk.source === "local" ? "offline" : trk.source}</span>
                  <span class="card-dur">{fmtTime(trk.duration)}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- 4. Nghe lại gần đây (Listen Again / History) -->
    {#if (activeFilter === "all" || activeFilter === "recent") && recentTracks.length > 0}
      <section class="yt-shelf">
        <div class="shelf-header">
          <div>
            <span class="shelf-kicker">LỊCH SỬ PHÁT GẦN ĐÂY</span>
            <h3 class="shelf-title">Nghe lại gần đây</h3>
          </div>
        </div>

        <div class="recent-list-row">
          {#each recentTracks as h, i (h.title + h.artist + i)}
            <div class="recent-item-card">
              <button
                type="button"
                class="recent-play-area"
                onclick={() => playRecentHistory(h)}
                title="Phát lại"
              >
                <div class="recent-icon">
                  <Play size={16} />
                </div>
                <div class="recent-info">
                  <span class="recent-title">{h.title}</span>
                  <span class="recent-artist">{h.artist}</span>
                </div>
              </button>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  .yt-home-container {
    display: flex;
    flex-direction: column;
    gap: 32px;
    padding: 8px 12px 48px;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    font-family: var(--font-sans);
  }

  /* Framed Filter Bar matching Studio UI */
  .yt-filter-bar {
    position: sticky;
    top: 4px;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 7px 12px;
    border-radius: var(--ui-radius-bar, 999px);
    background: var(--bg-glass, var(--bg-surface-2));
    border: 1px solid var(--border-strong);
    backdrop-filter: blur(16px) saturate(140%);
    box-shadow: 0 6px 20px oklch(0 0 0 / 0.14);
  }

  .filter-chips {
    min-width: 0;
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    overflow-x: auto;
    padding: 2px;
    scrollbar-width: none;
  }
  .filter-chips::-webkit-scrollbar {
    display: none;
  }

  .chip-btn {
    flex-shrink: 0;
    padding: 6px 14px;
    border-radius: var(--ui-radius-bar, 999px);
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .chip-btn:hover {
    background: var(--bg-surface-3);
    border-color: var(--border-strong);
    color: var(--text-primary);
  }
  .chip-btn.active {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 242));
    border-color: var(--accent);
    font-weight: 700;
    box-shadow: 0 2px 10px var(--accent-glow);
  }

  .refresh-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    padding: 6px 13px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .refresh-btn:hover:not(:disabled) {
    background: var(--bg-surface-3);
    border-color: var(--accent);
    color: var(--accent);
  }
  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  .refresh-btn :global(.spin) {
    animation: rot 0.8s linear infinite;
  }
  @keyframes rot {
    to {
      transform: rotate(360deg);
    }
  }

  /* Studio-Themed Featured Hero Card */
  .yt-hero-card {
    position: relative;
    border-radius: var(--ui-radius-lg, 20px);
    overflow: hidden;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 20px 24px;
    border: 1px solid var(--border-strong);
    background: linear-gradient(135deg, var(--bg-surface-3) 0%, var(--bg-surface-2) 55%, var(--accent-soft) 100%);
    box-shadow: 0 10px 28px oklch(0 0 0 / 0.16);
  }
  .hero-bg {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    opacity: 0.16;
  }
  .hero-bg img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: blur(34px) saturate(150%);
    transform: scale(1.12);
  }
  .hero-art-wrap {
    position: relative;
    z-index: 2;
    width: 118px;
    height: 118px;
    flex-shrink: 0;
    padding: 0;
    border-radius: var(--ui-radius, 16px);
    border: 1px solid var(--border-strong);
    background: var(--bg-surface-3);
    overflow: hidden;
    cursor: pointer;
    box-shadow: 0 8px 22px oklch(0 0 0 / 0.28);
    transition: transform 0.2s ease, border-color 0.2s ease;
  }
  .hero-art-wrap:hover {
    transform: scale(1.03);
    border-color: var(--accent);
  }
  .hero-art {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .hero-art-play {
    position: absolute;
    inset: 0;
    background: oklch(0 0 0 / 0.42);
    color: #fff;
    display: grid;
    place-items: center;
    opacity: 0;
    transition: opacity 0.18s ease;
  }
  .hero-art-wrap:hover .hero-art-play {
    opacity: 1;
  }
  .hero-content {
    position: relative;
    z-index: 2;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .hero-badge {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent);
    text-transform: uppercase;
  }
  .hero-title {
    margin: 0;
    font-size: clamp(18px, 2vw, 23px);
    font-weight: 800;
    line-height: 1.25;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  .hero-subtitle {
    margin: 0;
    font-size: 13.5px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
  }
  .hero-artist {
    color: var(--text-primary);
    font-weight: 600;
  }
  .hero-dot {
    opacity: 0.5;
  }
  .hero-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 6px;
  }
  .hero-play-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 20px;
    border-radius: 999px;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 242));
    font-size: 13.5px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 14px var(--accent-glow);
    transition: transform 0.16s ease, background 0.16s ease;
  }
  .hero-play-btn:hover {
    transform: translateY(-1px);
    background: var(--accent-hover);
  }
  .hero-detail-btn {
    padding: 8px 17px;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
  }
  .hero-detail-btn:hover {
    background: var(--bg-surface-3);
    border-color: var(--accent);
    color: var(--accent);
  }

  /* Shelf Component */
  .yt-shelf {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .shelf-header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
  }
  .shelf-kicker {
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent);
    text-transform: uppercase;
  }
  .shelf-title {
    margin: 2px 0 0;
    font-size: 19px;
    font-weight: 800;
    color: var(--text-primary);
  }
  .shelf-more-btn {
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 10px;
    border-radius: 999px;
    transition: all 0.15s ease;
  }
  .shelf-more-btn:hover {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: var(--border-subtle);
  }

  /* YouTube Music Grid Row — padded top/sides so hovered cards never clip their top border */
  .yt-grid-row {
    min-width: 0;
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(170px, 190px);
    gap: 16px;
    overflow-x: auto;
    padding: 8px 4px 12px;
    margin: -4px -4px 0;
    scrollbar-width: thin;
  }

  .yt-music-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: linear-gradient(160deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 16px);
    padding: 11px;
    transition: transform 0.2s ease, border-color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
    cursor: pointer;
  }
  .yt-music-card:hover {
    transform: translateY(-3px);
    background: linear-gradient(160deg, var(--bg-surface-3) 0%, var(--bg-surface-2) 100%);
    border-color: var(--accent);
    box-shadow: 0 6px 18px oklch(0 0 0 / 0.18);
  }
  .yt-music-card.playing {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .card-thumb-wrap {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
    border-radius: var(--ui-radius-sm, 11px);
    overflow: hidden;
    background: var(--bg-surface-3);
  }
  .card-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.3s ease;
  }
  .yt-music-card:hover .card-thumb {
    transform: scale(1.05);
  }

  .playing-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
  }


  .card-like-fab {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: none;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    color: rgba(255, 255, 255, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    opacity: 0;
    transform: scale(0.85);
    transition: opacity 0.2s ease, transform 0.2s ease, background 0.15s ease;
    cursor: pointer;
    z-index: 4;
  }
  .yt-music-card:hover .card-like-fab,
  .yt-music-card:focus-within .card-like-fab,
  .card-like-fab.liked {
    opacity: 1;
    transform: scale(1);
  }
  .card-like-fab.liked {
    color: #ef4444;
  }
  .card-like-fab:hover {
    background: rgba(0, 0, 0, 0.85);
    transform: scale(1.1);
  }

  .hero-like-btn {
    all: unset;
    box-sizing: border-box;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text-secondary);
    transition: transform 0.16s ease, background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
  }
  .hero-like-btn:hover {
    background: var(--bg-surface-3);
    border-color: var(--accent);
    color: var(--text-primary);
    transform: translateY(-1px);
  }
  .hero-like-btn.liked {
    color: #ef4444;
  }

  .card-play-fab {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    opacity: 0;
    transform: scale(0.85);
    transition: opacity 0.2s ease, transform 0.2s ease;
    cursor: pointer;
  }
  .card-play-fab :global(svg) {
    margin-left: 2px;
  }
  .yt-music-card:hover .card-play-fab,
  .yt-music-card:focus-within .card-play-fab {
    opacity: 1;
    transform: scale(1);
  }

  .liked-badge {
    position: absolute;
    top: 6px;
    left: 6px;
    font-size: 13px;
    background: rgba(0, 0, 0, 0.6);
    border-radius: 50%;
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .card-title {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
    line-height: 1.35;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  .card-title-btn {
    all: unset;
    cursor: pointer;
    text-align: left;
    display: block;
    width: 100%;
  }
  .card-title-btn:hover .card-title,
  .card-title:hover {
    color: var(--accent);
  }
  .card-artist {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .card-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 4px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .card-source-tag {
    text-transform: uppercase;
    font-weight: 700;
    font-size: 9.5px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-surface-3);
    color: var(--text-secondary);
  }

  /* Empty state */
  .empty-shelf-banner {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px 24px;
    border-radius: var(--ui-radius, 16px);
    background: var(--bg-surface-1);
    border: 1px dashed var(--border-subtle);
    color: var(--text-secondary);
  }
  .empty-title {
    margin: 0;
    font-size: 14.5px;
    font-weight: 700;
    color: var(--text-primary);
  }
  .empty-sub {
    margin: 2px 0 0;
    font-size: 12.5px;
    color: var(--text-muted);
  }

  /* Recent history horizontal pills */
  .recent-list-row {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 240px), 1fr));
    gap: 10px;
  }
  .recent-item-card {
    background: linear-gradient(135deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius-md, 14px);
    overflow: hidden;
    transition: background 0.16s ease, border-color 0.16s ease;
  }
  .recent-item-card:hover {
    background: var(--bg-surface-2);
    border-color: var(--accent);
  }
  .recent-play-area {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px 14px;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .recent-icon {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--bg-surface-3);
    color: var(--accent);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .recent-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .recent-title {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .recent-artist {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .home-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 20px;
    gap: 12px;
    color: var(--text-secondary);
    font-size: 14px;
  }
  .yt-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-subtle);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: rot 0.8s linear infinite;
  }
  @container stage (max-width: 600px) {
    .yt-filter-bar {
      gap: 8px;
      padding: 6px 8px;
    }
    .chip-btn {
      padding-inline: 10px;
      font-size: 12px;
    }
    .refresh-btn {
      padding-inline: 9px;
    }
    .yt-hero-card {
      gap: 14px;
      padding: 16px;
      align-items: flex-start;
    }
    .hero-art-wrap {
      width: 76px;
      height: 76px;
    }
    .hero-title {
      font-size: 18px;
    }
    .hero-subtitle {
      font-size: 12px;
    }
    .hero-actions {
      gap: 8px;
    }
    .hero-play-btn,
    .hero-detail-btn {
      padding: 8px 12px;
      font-size: 12px;
    }
    .yt-grid-row {
      grid-auto-columns: minmax(144px, 164px);
      gap: 12px;
    }
    .shelf-header {
      align-items: center;
    }
    .shelf-title {
      font-size: 17px;
    }
    .shelf-more-btn {
      flex-shrink: 0;
      padding-inline: 6px;
    }
    .empty-shelf-banner {
      padding: 16px;
      gap: 10px;
    }
  }
  @container stage (max-width: 400px) {
    .yt-hero-card {
      display: grid;
      grid-template-columns: 56px minmax(0, 1fr);
    }
    .hero-art-wrap {
      width: 56px;
      height: 56px;
    }
    .refresh-btn {
      font-size: 11px;
    }
  }
</style>
