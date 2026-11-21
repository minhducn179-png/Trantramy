<script lang="ts">
  import { Pause, Play, Disc3, Heart, Plus, Bookmark, Download, Copy, Trash2, Music2 } from "@lucide/svelte";
  import LyricsPanel from "$lib/LyricsPanel.svelte";
  import { api, fmtTime, getTrackCover, handleImgError, type Track, type Playlist, type OnlineTrack } from "$lib/api";
  import { player } from "$lib/player.svelte";
  import WaveformBar from "./WaveformBar.svelte";

  let {
    track,
    contextTracks = [],
    playlists = [],
    downloads = {},
    onadd,
    ondelete,
    ondownload,
    ontogglelike,
    onlyricssettings,
  }: {
    track: Track;
    contextTracks?: Track[];
    playlists?: Playlist[];
    downloads?: Record<string, number>;
    onclose?: () => void;
    onadd?: (playlistId: number, trackId: number) => void;
    ondelete?: (t: Track) => void;
    ondownload?: (t: Track) => void;
    ontogglelike?: (t: Track) => void;
    onlyricssettings?: () => void;
  } = $props();

  let showPlMenu = $state(false);
  let copied = $state(false);
  // Progressive enrichment: continue loading related tracks from the artist
  let relatedTracks = $state<OnlineTrack[]>([]);
  let relatedLoading = $state(false);

  $effect(() => {
    const t = track;
    relatedTracks = [];
    if (!t) {
      relatedLoading = false;
      return;
    }
    const query = (t.artist && t.artist !== "Unknown Artist" && t.artist !== "Nghệ sĩ chưa rõ" ? t.artist : t.title) + " hits";
    relatedLoading = true;
    api
      .searchOnline(query, "all", 6)
      .then((res) => {
        if (t.id === track.id || t.url === track.url || t.path === track.path) {
          relatedTracks = res.filter((x) => x.url !== (t.url || t.path)).slice(0, 5);
        }
      })
      .catch(() => {})
      .finally(() => {
        relatedLoading = false;
      });
  });

  function playRelated(rel: OnlineTrack, idx: number) {
    player.playOnlineList(relatedTracks, idx);
    track = {
      id: -200 - idx,
      path: rel.url,
      title: rel.title,
      artist: rel.artist,
      album: rel.album,
      duration: rel.duration,
      cover_path: rel.thumbnail,
      source: rel.source,
      url: rel.url,
    };
  }

  const isCurrent = $derived(
    player.current?.id === track.id ||
      Boolean(player.current?.url && track.url && player.current.url === track.url) ||
      player.current?.path === track.path,
  );

  const isPlayingThis = $derived(isCurrent && player.playing);
  const currentPos = $derived(isCurrent ? player.position : 0);
  const currentDur = $derived(isCurrent && player.duration > 0 ? player.duration : track.duration);

  const isRemote = $derived(track.path.startsWith("http://") || track.path.startsWith("https://"));
  const dlKey = $derived(track.url ?? track.path);
  const dlPct = $derived(downloads[dlKey]);
  const cover = $derived(getTrackCover(track.cover_path, track.url || track.path, track.title, track.artist));

  const formatInfo = $derived.by(() => {
    const p = track.path.toLowerCase();
    if (p.endsWith(".flac")) return "FLAC · Âm thanh Lossless";
    if (p.endsWith(".mp3")) return "MP3 · Âm thanh tiêu chuẩn";
    if (p.endsWith(".m4a") || p.endsWith(".aac")) return "M4A / AAC · Chất lượng cao";
    if (p.endsWith(".opus") || p.endsWith(".ogg")) return "OPUS / OGG · Âm thanh nén";
    if (p.endsWith(".wav")) return "WAV · Âm thanh gốc PCM";
    if (isRemote) {
      return track.source === "soundcloud"
        ? "AAC / HLS · SoundCloud Stream"
        : track.source === "spotify"
          ? "M4A / OPUS · Spotify Matched Stream"
          : "M4A / OPUS · YouTube Audio Stream";
    }
    return "Bản thu kỹ thuật số";
  });

  const sourceLabel = $derived(
    track.source === "soundcloud"
      ? "SoundCloud"
      : track.source === "youtube"
        ? "YouTube"
        : track.source === "spotify"
          ? "Spotify"
          : "Trên máy (Offline)",
  );

  function togglePlayThis() {
    if (isCurrent) {
      player.toggle();
    } else {
      const list = contextTracks.length > 0 ? contextTracks : [track];
      const idx = list.findIndex(
        (x) => x.id === track.id || (x.url && track.url && x.url === track.url) || x.path === track.path,
      );
      if (idx >= 0) {
        player.playList(list, idx);
      } else {
        player.playList([track], 0);
      }
    }
  }

  function handleWaveSeek(sec: number) {
    if (isCurrent) {
      player.seek(sec);
    } else {
      togglePlayThis();
      setTimeout(() => player.seek(sec), 250);
    }
  }

  async function copyPath() {
    try {
      await navigator.clipboard.writeText(track.url || track.path);
      copied = true;
      setTimeout(() => (copied = false), 1800);
    } catch {
      // ignore
    }
  }

</script>

<div class="detail-stage">
  <!-- Hero Banner -->
  <section class="sc-hero">
    {#if cover}
      <div class="hero-backdrop" style="background-image: url('{cover}')"></div>
    {/if}

    <div class="hero-main">
      <!-- Left: Transport + Title + Giant Waveform -->
      <div class="hero-left">
        <div class="hero-top-row">
          <button
            class="sc-play-giant"
            onclick={togglePlayThis}
            title={isPlayingThis ? "Tạm dừng" : "Phát bài hát này"}
            aria-label={isPlayingThis ? "Tạm dừng" : "Phát bài hát này"}
          >
            {#if isPlayingThis}
              <Pause size={24} />
            {:else}
              <Play size={26} />
            {/if}
          </button>

          <div class="sc-title-stack">
            <div class="sc-artist-row">
              <span class="sc-artist-tag">{track.artist || "Nghệ sĩ chưa rõ"}</span>
              {#if track.album}
                <span class="sc-album-tag">· {track.album}</span>
              {/if}
              <span class="hero-source-badge">{sourceLabel}</span>
            </div>
            <h1 class="sc-track-title">{track.title}</h1>
          </div>
        </div>

        <!-- Giant Waveform -->
        <div class="hero-wave-box">
          <span class="wave-time left">{fmtTime(currentPos)}</span>
          <WaveformBar
            seed="{track.id}:{track.title}:{track.duration}"
            position={currentPos}
            duration={currentDur}
            playing={isPlayingThis}
            height={92}
            onseek={handleWaveSeek}
          />
          <span class="wave-time right">{fmtTime(currentDur)}</span>
        </div>
      </div>

      <!-- Right: High-Res Artwork Square -->
      <div class="hero-artwork">
        {#if cover}
          <img src={cover} alt={track.title} onerror={(e) => handleImgError(e, track.title, track.artist, track.url || track.path)} />
        {:else}
          <div class="hero-art-ph">
            <Disc3 size={54} />
          </div>
        {/if}
      </div>
    </div>
  </section>

  <!-- Action Toolbar -->
  <div class="sc-toolbar">
    <div class="tool-left">
      <button
        class="sc-btn sc-like-btn"
        class:liked={track.liked}
        onclick={() => ontogglelike?.(track)}
        title={track.liked ? "Bỏ thích bài hát này" : "Yêu thích bài hát này (Lưu vào Trang chủ)"}
      >
        <Heart size={14} fill={track.liked ? "currentColor" : "none"} />
        <span>{track.liked ? "Đã thích" : "Yêu thích"}</span>
      </button>

      <button class="sc-btn" onclick={() => player.enqueue(track)}>
        <Plus size={14} />
        <span>Thêm vào hàng chờ</span>
      </button>

      {#if onadd && track.id > 0}
        <div class="pl-dropdown-wrap">
          <button class="sc-btn" onclick={() => (showPlMenu = !showPlMenu)}>
            <Bookmark size={14} />
            <span>Thêm vào Playlist</span>
          </button>

          {#if showPlMenu}
            <div class="pl-menu">
              {#each playlists as p}
                <button
                  onclick={() => {
                    onadd(p.id, track.id);
                    showPlMenu = false;
                  }}
                >
                  <span>{p.name}</span>
                  <small>{p.count}</small>
                </button>
              {:else}
                <span class="empty-pl">Chưa có playlist nào</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      {#if isRemote && ondownload}
        {#if dlPct !== undefined}
          <span class="dl-progress-badge">Đang tải {Math.round(dlPct)}%</span>
        {:else}
          <button class="sc-btn" onclick={() => ondownload(track)}>
            <Download size={14} />
            <span>Tải về máy (Offline)</span>
          </button>
        {/if}
      {/if}

      <button class="sc-btn" onclick={copyPath}>
        <Copy size={14} />
        <span>{copied ? "Đã sao chép!" : "Sao chép liên kết"}</span>
      </button>
    </div>

    {#if ondelete && track.id > 0}
      <button class="sc-btn danger" onclick={() => ondelete(track)}>
        <Trash2 size={14} />
        <span>Xóa khỏi thư viện</span>
      </button>
    {/if}
  </div>

  <!-- Two-Column Track Inspector: Metadata Specs + Synced Lyrics -->
  <div class="inspector-grid">
    <!-- Left Column: Concise Track Information -->
    <section class="info-card">
      <header class="card-head">
        <div>
          <span class="kicker">THÔNG TIN BÀI HÁT</span>
          <h3>Chi tiết bản thu</h3>
        </div>
      </header>

      <div class="spec-grid">
        <div class="spec-item">
          <span class="spec-label">Nghệ sĩ</span>
          <strong class="spec-val accent">{track.artist || "Nghệ sĩ chưa rõ"}</strong>
        </div>

        <div class="spec-item">
          <span class="spec-label">Album / Tuyển tập</span>
          <span class="spec-val">{track.album || "Đĩa đơn (Single)"}</span>
        </div>

        <div class="spec-item">
          <span class="spec-label">Thời lượng</span>
          <span class="spec-val mono">{fmtTime(track.duration)}</span>
        </div>

        <div class="spec-item">
          <span class="spec-label">Nguồn phát</span>
          <span class="spec-val">{sourceLabel}</span>
        </div>

        <div class="spec-item">
          <span class="spec-label">Định dạng âm thanh</span>
          <span class="spec-val mono">{formatInfo}</span>
        </div>

        <div class="spec-item">
          <span class="spec-label">Trạng thái lưu trữ</span>
          <span class="spec-val">
            {isRemote ? "Phát trực tuyến (Cloud)" : "Đã lưu trong máy (Offline)"}
          </span>
        </div>
      </div>
    </section>

    <section id="track-detail-lyrics" class="detail-lyrics" aria-label="Lời bài hát trên trang thông tin" tabindex="-1">
      <LyricsPanel {track} embedded onsettings={onlyricssettings} />
    </section>
  </div>

  <!-- Progressive Section: Related Tracks from same artist / vibe -->
  <section class="related-section">
    <header class="card-head">
      <div>
        <span class="kicker">KHÁM PHÁ TIẾP THEO</span>
        <h3>Gợi ý cùng nghệ sĩ & Bản nhạc tương tự</h3>
      </div>
      {#if relatedLoading}
        <div class="rel-loading-pill">
          <span class="mini-spin"></span>
          <span>Đang tìm nạp thêm bài hát liên quan…</span>
        </div>
      {/if}
    </header>

    {#if relatedTracks.length > 0}
      <div class="related-track-grid">
        {#each relatedTracks as rel, ri (rel.url + ":rel")}
          <div class="rel-card">
            <button
              type="button"
              class="rel-art-btn"
              onclick={() => playRelated(rel, ri)}
              title="Phát & mở {rel.title}"
            >
              <img
                src={getTrackCover(rel.thumbnail, rel.url, rel.title, rel.artist)}
                alt=""
                loading="lazy"
                onerror={(e) => handleImgError(e, rel.title, rel.artist, rel.url)}
              />
              <span class="rel-badge">{rel.source === "soundcloud" ? "SC" : "YT"}</span>
            </button>

            <div class="rel-meta">
              <button
                type="button"
                class="rel-title"
                onclick={() => playRelated(rel, ri)}
                title={rel.title}
              >
                {rel.title}
              </button>
              <span class="rel-artist">{rel.artist}</span>
              <span class="rel-dur">{fmtTime(rel.duration)}</span>
            </div>

            <button
              type="button"
              class="rel-play-action"
              onclick={() => playRelated(rel, ri)}
              title="Phát ngay"
            >
              <Play size={14} />
            </button>
          </div>
        {/each}
      </div>
    {:else if !relatedLoading}
      <div class="rel-empty-note">
        Đã tải đầy đủ thông tin bài hát và hồ sơ âm thanh.
      </div>
    {/if}
  </section>
</div>

<style>
  .related-section {
    background: linear-gradient(145deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 70%, var(--accent-soft) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius-lg, 18px);
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.14);
  }
  .rel-loading-pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--accent);
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
  }
  .mini-spin {
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: miniRotate 0.75s linear infinite;
  }
  @keyframes miniRotate {
    to { transform: rotate(360deg); }
  }
  .related-track-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 220px), 1fr));
    gap: 12px;
  }
  .rel-card {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    border-radius: var(--ui-radius-md, 14px);
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    transition: all 0.16s ease;
  }
  .rel-card:hover {
    background: var(--bg-surface-3);
    border-color: var(--accent);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.16);
  }
  .rel-art-btn {
    position: relative;
    width: 46px;
    height: 46px;
    border-radius: var(--ui-radius-sm, 11px);
    border: none;
    padding: 0;
    overflow: hidden;
    cursor: pointer;
    flex: none;
    background: var(--bg-surface-3);
  }
  .rel-art-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .rel-badge {
    position: absolute;
    bottom: 2px;
    right: 2px;
    font-size: 8.5px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 999px;
    background: oklch(0 0 0 / 0.75);
    color: var(--accent);
  }
  .rel-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .rel-title {
    background: none;
    border: none;
    padding: 0;
    text-align: left;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }
  .rel-title:hover {
    color: var(--accent);
  }
  .rel-artist {
    font-size: 11.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .rel-dur {
    font-size: 10.5px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .rel-play-action {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: none;
    background: var(--bg-surface-3);
    color: var(--text-primary);
    display: grid;
    place-items: center;
    cursor: pointer;
    flex: none;
    transition: all 0.15s ease;
  }
  .rel-play-action:hover {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    transform: scale(1.08);
  }
  .rel-empty-note {
    font-size: 12.5px;
    color: var(--text-secondary);
    font-style: italic;
  }

  .detail-stage {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
    padding: 16px 0 24px;
  }

  .hero-source-badge {
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    margin-left: 4px;
  }

  /* Brightened, Rounded Hero Banner */
  .sc-hero {
    position: relative;
    border-radius: var(--ui-radius-lg, 20px);
    overflow: hidden;
    background: linear-gradient(135deg, var(--bg-surface-3) 0%, var(--bg-surface-2) 55%, var(--accent-soft) 100%);
    border: 1px solid var(--border-strong);
    padding: 24px 26px;
    box-shadow: 0 12px 32px oklch(0 0 0 / 0.22);
  }
  .hero-backdrop {
    position: absolute;
    inset: -30px;
    background-size: cover;
    background-position: center;
    filter: blur(48px) saturate(155%) brightness(0.95);
    opacity: 0.22;
    pointer-events: none;
  }

  .hero-main {
    position: relative;
    z-index: 2;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 208px;
    gap: 28px;
    align-items: stretch;
  }

  .hero-left {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 22px;
    min-width: 0;
  }

  .hero-top-row {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }

  .sc-play-giant {
    width: 58px;
    height: 58px;
    border-radius: 50%;
    border: none;
    background: linear-gradient(135deg, var(--accent-hover), var(--accent));
    color: var(--accent-fg, oklch(0.1 0.01 240));
    display: grid;
    place-items: center;
    cursor: pointer;
    flex: none;
    box-shadow: 0 8px 24px var(--accent-glow);
    transition: transform 0.2s var(--ease-out-expo), box-shadow 0.2s ease;
  }
  .sc-play-giant:hover {
    transform: scale(1.06);
    box-shadow: 0 10px 30px var(--accent-glow);
  }
  .sc-play-giant:active {
    transform: scale(0.95);
  }

  .sc-title-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    align-items: flex-start;
  }
  .sc-artist-row {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--bg-surface-1);
    border: 1px solid var(--border-strong);
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 600;
    color: var(--accent);
    max-width: 100%;
    flex-wrap: wrap;
    overflow-wrap: anywhere;
  }
  .sc-album-tag {
    color: var(--text-secondary);
    font-weight: 500;
  }
  .sc-track-title {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
    letter-spacing: -0.025em;
    line-height: 1.25;
    color: var(--text-primary);
    word-break: break-word;
  }

  .hero-wave-box {
    position: relative;
    padding: 8px 12px 6px;
    border-radius: var(--ui-radius-md, 14px);
    background: color-mix(in oklch, var(--bg-surface-1) 72%, transparent);
    border: 1px solid var(--border-subtle);
  }
  .wave-time {
    position: absolute;
    bottom: 24px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10.5px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    color: var(--text-primary);
    pointer-events: none;
    z-index: 4;
  }
  .wave-time.left {
    left: 10px;
    color: var(--accent);
  }
  .wave-time.right {
    right: 10px;
    color: var(--text-secondary);
  }

  .hero-artwork {
    width: 208px;
    height: 208px;
    border-radius: var(--ui-radius, 16px);
    overflow: hidden;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    box-shadow: 0 10px 28px oklch(0 0 0 / 0.35);
  }
  .hero-artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .hero-art-ph {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--accent);
    background: radial-gradient(circle at 30% 30%, var(--bg-surface-3), var(--bg-surface-1));
  }

  /* Toolbar */
  .sc-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-wrap: wrap;
    padding: 10px 14px;
    border-radius: var(--ui-radius, 16px);
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
  }
  .tool-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .sc-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 34px;
    padding: 0 14px;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface-1);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .sc-btn.sc-like-btn.liked {
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.45);
    background: rgba(239, 68, 68, 0.12);
  }
  .sc-btn:hover {
    background: var(--bg-surface-3);
    border-color: var(--accent);
  }
  .sc-btn.danger:hover {
    border-color: var(--danger);
    color: var(--danger);
    background: oklch(0.67 0.19 25 / 0.12);
  }
  .dl-progress-badge {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 6px 14px;
    border-radius: 999px;
  }

  .pl-dropdown-wrap {
    position: relative;
  }
  .pl-menu {
    position: absolute;
    top: 40px;
    left: 0;
    min-width: 190px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 14px;
    padding: 6px;
    z-index: 30;
    box-shadow: 0 14px 32px oklch(0 0 0 / 0.45);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .pl-menu button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 7px 10px;
    border: none;
    border-radius: 9px;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12.5px;
    cursor: pointer;
  }
  .pl-menu button:hover {
    background: var(--bg-surface-3);
    color: var(--accent);
  }
  .empty-pl {
    font-size: 12px;
    color: var(--text-muted);
    padding: 8px;
  }

  .detail-lyrics { display: flex; min-width: 0; scroll-margin-block: 16px; }
  .detail-lyrics:focus-visible { outline: 2px solid var(--accent); outline-offset: 4px; border-radius: 12px; }

  /* Two-Column Inspector — Brightened & Elevated Surface */
  .inspector-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
    gap: 16px;
    align-items: stretch;
  }

  .info-card {
    background: linear-gradient(145deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 65%, var(--accent-soft) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius-lg, 18px);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.14);
  }

  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-wrap: wrap;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .kicker {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--accent);
    display: block;
  }
  .card-head h3 {
    margin: 2px 0 0;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.015em;
  }

  .spec-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 12px;
  }
  .spec-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 11px 13px;
    border-radius: var(--ui-radius-md, 14px);
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    min-width: 0;
  }
  .spec-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }
  .spec-val {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-primary);
    word-break: break-word;
  }
  .spec-val.accent {
    color: var(--accent);
  }
  .spec-val.mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }


  @container stage (max-width: 760px) {
    .detail-stage {
      padding: 14px 0 24px;
      gap: 14px;
    }
    .sc-hero {
      padding: 16px;
    }
    .hero-main {
      grid-template-columns: 1fr;
      gap: 16px;
    }
    .hero-artwork {
      width: 120px;
      height: 120px;
      order: -1;
    }
    .sc-track-title {
      font-size: 20px;
    }
    .inspector-grid {
      grid-template-columns: minmax(0, 1fr);
    }
    .related-section,
    .info-card {
      padding: 16px;
    }
    .hero-top-row {
      gap: 12px;
    }
    .sc-play-giant {
      width: 46px;
      height: 46px;
    }
  }
</style>