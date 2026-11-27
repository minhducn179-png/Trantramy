<script lang="ts">
  import { Disc3, Expand, Heart, Shuffle, SkipBack, Pause, Play, SkipForward, Repeat, CirclePlay, Mic2, ListMusic, SlidersVertical, PictureInPicture2, VolumeX, Volume1, Volume2 } from "@lucide/svelte";
  import { player } from "./player.svelte";
  import { fmtTime, getTrackCover, handleImgError, type Track } from "$lib/api";
  import WaveformBar from "$lib/audio/WaveformBar.svelte";

  let {
    onqueue,
    onlyrics,
    oninspect,
    onplayrandom,
    onplaydefault,
    showQueue = false,
    showLyrics = false,
    showEqualizer = false,
    showKaraoke = false,
    showIsland = false,
    inspecting = false,
    onequalizer,
    onkaraoke,
    onisland,
    ontogglelike,
  }: {
    ontogglelike?: (t: Track) => void;
    onqueue: () => void;
    onlyrics: () => void;
    onequalizer?: () => void;
    onkaraoke?: () => void;
    onisland?: () => void;
    oninspect?: () => void;
    onplayrandom?: () => void;
    onplaydefault?: () => void;
    showQueue?: boolean;
    showLyrics?: boolean;
    showEqualizer?: boolean;
    showKaraoke?: boolean;
    showIsland?: boolean;
    inspecting?: boolean;
  } = $props();

  let prevVol = $state(0.9);
  const cover = $derived(
    player.current
      ? getTrackCover(
          player.current.cover_path,
          player.current.url || player.current.path,
          player.current.title,
          player.current.artist,
        )
      : "",
  );
  const volPct = $derived(Math.min(100, Math.max(0, player.volume * 100)));

  function toggleMute() {
    if (player.volume > 0) {
      prevVol = player.volume;
      player.setVolume(0);
    } else {
      player.setVolume(prevVol > 0 ? prevVol : 0.8);
    }
  }

  function handleShuffle() {
    if (!player.current || player.queue.length === 0) {
      onplayrandom?.();
      return;
    }
    if (!player.shuffle) {
      if (player.queue.length <= 1 || !player.playing) {
        onplayrandom?.();
      } else {
        player.toggleShuffle();
      }
    } else {
      player.toggleShuffle();
    }
  }

  function handlePlayToggle() {
    if (!player.current) {
      if (player.shuffle) onplayrandom?.();
      else onplaydefault?.();
      return;
    }
    player.toggle();
  }
</script>

<footer class="deck">
  <div class="identity-group">
  <!-- Track Identity (Click to open full SoundCloud Track Stage) -->
  <button
    class="identity"
    class:active-inspect={inspecting}
    onclick={() => player.current && oninspect?.()}
    disabled={!player.current}
    title={player.current ? "Bấm để mở khung chi tiết bài hát & sóng nhạc lớn" : "Chưa chọn bài hát"}
  >
    <div class="artwork" class:spinning={player.playing}>
      {#if cover}
        <img
          src={cover}
          alt="Cover"
          onerror={(e) =>
            handleImgError(
              e,
              player.current?.title,
              player.current?.artist,
              player.current?.url || player.current?.path,
            )}
        />
      {:else}
        <div class="artwork-ph" aria-hidden="true">
          <Disc3 size={22} />
        </div>
      {/if}
      {#if player.current}
        <div class="expand-badge" aria-hidden="true">
          <Expand size={13} />
        </div>
      {/if}
    </div>

    <div class="meta">
      <div class="title-row">
        <span class="title">
          {player.current?.title ?? "Chưa chọn bài hát"}
        </span>
        {#if player.loadingStream}
          <span class="stream-pill">
            <span class="pulse-dot"></span>
            Đang kết nối
          </span>
        {/if}
      </div>
      <div class="subtitle">
        {#if player.current}
          <span>{player.current.artist || "Nghệ sĩ chưa xác định"}</span>
          {#if player.current.album}
            <span class="sep">·</span>
            <span class="album">{player.current.album}</span>
          {/if}
        {:else}
          <span class="idle-hint">Chọn một bài hát để bắt đầu nghe</span>
        {/if}
      </div>
    </div>
  </button>

  {#if player.current}
    <button
      type="button"
      class="bar-like-btn"
      class:liked={player.current.liked}
      onclick={() => ontogglelike?.(player.current!)}
      title={player.current.liked ? "Bỏ thích" : "Yêu thích bài hát này"}
      aria-label="Thích"
    >
      <Heart size={18} fill={player.current.liked ? "currentColor" : "none"} />
    </button>
  {/if}
  </div>

  <!-- SoundCloud Transport & Waveform Timeline -->
  <div class="transport">
    <div class="controls">
      <button
        class="icon-btn"
        class:active={player.shuffle}
        title={player.shuffle ? "Phát ngẫu nhiên: Đang bật" : "Phát ngẫu nhiên: Đang tắt"}
        aria-label="Phát ngẫu nhiên"
        onclick={handleShuffle}
      >
        <Shuffle size={16} />
        {#if player.shuffle}<span class="dot"></span>{/if}
      </button>

      <button class="icon-btn" title="Bài trước" aria-label="Bài trước" onclick={() => player.prev()}>
        <SkipBack size={18} />
      </button>

      <button
        class="play-fab"
        title={player.playing ? "Tạm dừng" : "Phát"}
        aria-label={player.playing ? "Tạm dừng" : "Phát"}
        onclick={handlePlayToggle}
      >
        {#if player.playing}
          <Pause size={19} />
        {:else}
          <Play size={19} />
        {/if}
      </button>

      <button class="icon-btn" title="Bài tiếp theo" aria-label="Bài tiếp theo" onclick={() => player.next()}>
        <SkipForward size={18} />
      </button>

      <button
        class="icon-btn"
        class:active={player.repeat !== "off"}
        title={player.repeat === "off"
          ? "Chế độ lặp: Tắt"
          : player.repeat === "all"
            ? "Chế độ lặp: Toàn bộ danh sách"
            : "Chế độ lặp: 1 bài hát"}
        aria-label="Chế độ lặp"
        onclick={() => player.cycleRepeat()}
      >
        <Repeat size={16} />
        {#if player.repeat === "one"}
          <span class="repeat-one">1</span>
        {/if}
        {#if player.repeat !== "off"}<span class="dot"></span>{/if}
      </button>

      <button
        class="icon-btn"
        class:active={player.autoRelated}
        title={player.autoRelated
          ? "Tự động phát bài liên quan: Đang bật"
          : "Tự động phát bài liên quan: Đang tắt"}
        aria-label="Tự phát bài liên quan"
        onclick={() => player.toggleAutoRelated()}
      >
        <CirclePlay size={16} />
        {#if player.autoRelated}<span class="dot"></span>{/if}
      </button>
    </div>

    <!-- SoundCloud Waveform Timeline -->
    <div class="timeline">
      <span class="timecode active">{fmtTime(player.position)}</span>
      <div class="wave-slot">
        <WaveformBar
          seed={player.current ? `${player.current.id}:${player.current.title}:${player.current.duration}` : "idle-wave"}
          position={player.position}
          duration={player.duration}
          playing={player.playing}
          height={30}
          onseek={(sec) => player.seek(sec)}
        />
      </div>
      <span class="timecode">{fmtTime(player.duration)}</span>
    </div>
  </div>

  <!-- Studio Utilities -->
  <div class="utilities">
    <button
      class="util-btn icon-only"
      class:active={showLyrics}
      title={inspecting ? "Đến lời bài hát trên trang thông tin" : "Lời bài hát & Studio Karaoke"}
      aria-label={inspecting ? "Đến lời bài hát trên trang thông tin" : "Lời bài hát & Studio Karaoke"}
      aria-controls={inspecting ? "track-detail-lyrics" : undefined}
      aria-pressed={inspecting ? undefined : showLyrics}
      onclick={onlyrics}
    >
      <Mic2 size={16} />
    </button>

    <button
      class="util-btn"
      class:active={showQueue}
      title="Hàng chờ phát"
      aria-label="Hàng chờ phát"
      aria-pressed={showQueue}
      onclick={onqueue}
    >
      <ListMusic size={15} />
      {#if player.queue.length > 0}
        <span class="q-badge">{player.queue.length}</span>
      {/if}
    </button>

    <!-- Equalizer Button -->
    <button
      class="util-btn eq-bar-btn"
      class:active={showEqualizer}
      title="Bộ cân bằng âm thanh (Equalizer)"
      onclick={onequalizer}
    >
      <SlidersVertical size={15} />
    </button>

    <!-- Dynamic Island / Floating Mini Player Button -->
    <button
      class="util-btn icon-only"
      class:active={showIsland}
      title="Dynamic Island (Mini Player)"
      aria-label="Dynamic Island"
      onclick={onisland}
    >
      <PictureInPicture2 size={15} />
    </button>

    <div class="volume-group">
      <button class="icon-btn vol-icon" onclick={toggleMute} title={player.volume === 0 ? "Bật tiếng" : "Tắt tiếng"} aria-label="Âm lượng">
        {#if player.volume === 0}
          <VolumeX size={16} />
        {:else if player.volume < 0.5}
          <Volume1 size={16} />
        {:else}
          <Volume2 size={16} />
        {/if}
      </button>

      <div class="scrubber vol-scrubber" style="--pct: {volPct}%">
        <div class="scrubber-track">
          <div class="scrubber-fill"></div>
          <div class="scrubber-thumb"></div>
        </div>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={player.volume}
          aria-label="Thanh âm lượng"
          oninput={(e) => player.setVolume(+e.currentTarget.value)}
        />
      </div>
    </div>
  </div>
</footer>

<style>
  .deck {
    display: grid;
    grid-template-columns: minmax(170px, 1fr) minmax(270px, 1.6fr) auto;
    align-items: center;
    gap: 16px;
    height: var(--deck-height, 90px);
    padding: 0 20px;
    background: var(--bg-glass);
    backdrop-filter: blur(24px) saturate(160%);
    border-top: 1px solid var(--border-subtle);
    position: relative;
    z-index: 30;
  }


  .identity-group {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .identity-group .identity {
    flex: 1;
  }
  .bar-like-btn {
    all: unset;
    cursor: pointer;
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    color: var(--text-secondary);
    transition: color 0.16s ease, transform 0.16s ease, background 0.16s ease;
    flex-shrink: 0;
  }
  .bar-like-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-2);
    transform: scale(1.1);
  }
  .bar-like-btn.liked {
    color: #ef4444;
  }

  /* Identity */
  .identity {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    border: 1px solid var(--border-subtle);
    background: linear-gradient(135deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    padding: 6px 12px 6px 7px;
    border-radius: var(--ui-radius-md, 14px);
    text-align: left;
    color: inherit;
    font-family: var(--font-sans);
    cursor: pointer;
    transition: background 0.16s ease, border-color 0.16s ease;
  }
  .identity:hover:not(:disabled) {
    background: var(--bg-surface-3);
    border-color: var(--border-strong);
  }
  .identity.active-inspect {
    background: var(--accent-soft);
    border-color: oklch(0.7 0.17 238 / 0.35);
  }
  .identity:disabled {
    cursor: default;
  }

  .artwork {
    position: relative;
    width: 52px;
    height: 52px;
    border-radius: var(--ui-radius-sm, 10px);
    overflow: hidden;
    flex: none;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    box-shadow: 0 6px 18px oklch(0 0 0 / 0.5);
    transition: border-radius 0.4s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.3s ease, border-color 0.3s ease;
  }
  .artwork.spinning {
    border-radius: 50%;
    border-color: var(--accent);
    box-shadow: 0 0 14px var(--accent-glow, rgba(99, 102, 241, 0.4)), 0 4px 10px rgba(0, 0, 0, 0.25);
  }
  .artwork.spinning img,
  .artwork.spinning .artwork-ph {
    border-radius: 50%;
    animation: spin-mini-vinyl 3s linear infinite;
  }
  @keyframes spin-mini-vinyl {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  /* Mini vinyl record spindle hole and groove reflection */
  .artwork.spinning::after {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    width: 10px;
    height: 10px;
    margin: -5px 0 0 -5px;
    border-radius: 50%;
    background: var(--bg-canvas, #09090e);
    border: 2px solid rgba(255, 255, 255, 0.7);
    box-shadow: 0 0 4px rgba(0, 0, 0, 0.8);
    pointer-events: none;
    z-index: 2;
  }
  .artwork.spinning::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(from 45deg, transparent 0deg, rgba(255, 255, 255, 0.28) 35deg, transparent 70deg, transparent 180deg, rgba(255, 255, 255, 0.28) 215deg, transparent 250deg);
    mix-blend-mode: screen;
    pointer-events: none;
    z-index: 1;
    animation: spin-mini-vinyl 3s linear infinite;
  }
  .artwork img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .artwork-ph {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--accent);
    background: radial-gradient(circle at 30% 30%, var(--bg-surface-3), var(--bg-surface-1));
  }
  .expand-badge {
    position: absolute;
    inset: 0;
    background: oklch(0.08 0.01 240 / 0.65);
    color: var(--accent);
    display: grid;
    place-items: center;
    opacity: 0;
    transition: opacity 0.16s ease;
  }
  .identity:hover .expand-badge {
    opacity: 1;
  }

  .meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .title {
    font-size: 13.5px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .stream-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 2px 7px;
    border-radius: 999px;
    flex: none;
  }
  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1.2s infinite ease-in-out;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.35; transform: scale(0.85); }
    50% { opacity: 1; transform: scale(1.15); }
  }

  .subtitle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sep {
    color: var(--text-muted);
  }
  .album {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .idle-hint {
    color: var(--text-muted);
  }

  /* Transport */
  .transport {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    min-width: 0;
    max-width: 640px;
    width: 100%;
    margin: 0 auto;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .icon-btn {
    position: relative;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    transition: color 0.16s ease, background 0.16s ease, transform 0.16s var(--ease-out-expo);
  }
  .icon-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-2);
  }
  .icon-btn:active {
    transform: scale(0.93);
  }
  .icon-btn.active {
    color: var(--accent);
  }
  .dot {
    position: absolute;
    bottom: 2px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent);
  }
  .repeat-one {
    position: absolute;
    top: 2px;
    right: 3px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    color: var(--accent);
  }

  .play-fab {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    border: none;
    background: linear-gradient(135deg, var(--accent-hover), var(--accent));
    color: var(--accent-fg, oklch(0.09 0.015 240));
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    box-shadow: 0 4px 16px var(--accent-glow);
    transition: transform 0.2s var(--ease-out-expo), box-shadow 0.2s ease;
  }
  .play-fab:hover {
    transform: scale(1.06);
    box-shadow: 0 6px 22px var(--accent-glow);
  }
  .play-fab:active {
    transform: scale(0.95);
  }

  /* SoundCloud Waveform Timeline */
  .timeline {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
  }
  .wave-slot {
    flex: 1;
    min-width: 0;
  }
  .timecode {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    color: var(--text-muted);
    min-width: 36px;
    text-align: center;
  }
  .timecode.active {
    color: var(--accent);
    font-weight: 600;
  }

  /* Volume Scrubber */
  .scrubber {
    position: relative;
    flex: 1;
    height: 18px;
    display: flex;
    align-items: center;
  }
  .scrubber-track {
    position: relative;
    width: 100%;
    height: 4px;
    border-radius: 999px;
    background: var(--bg-surface-3);
    overflow: visible;
    transition: height 0.15s ease;
  }
  .scrubber:hover .scrubber-track {
    height: 6px;
  }
  .scrubber-fill {
    height: 100%;
    width: var(--pct);
    border-radius: 999px;
    background: var(--accent);
  }
  .scrubber-thumb {
    position: absolute;
    top: 50%;
    left: var(--pct);
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: var(--text-primary);
    box-shadow: 0 2px 6px oklch(0 0 0 / 0.5);
    transform: translate(-50%, -50%) scale(0);
    transition: transform 0.15s var(--ease-out-expo);
    pointer-events: none;
  }
  .scrubber:hover .scrubber-thumb {
    transform: translate(-50%, -50%) scale(1);
  }
  .scrubber input[type="range"] {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: pointer;
  }

  /* Utilities */
  .utilities {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
  }
  .util-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .util-btn.icon-only {
    width: 32px;
    padding: 0;
  }
  .util-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-2);
  }
  .util-btn.active {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: oklch(0.7 0.17 238 / 0.3);
  }
  .q-badge {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10.5px;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--bg-surface-3);
    color: var(--text-primary);
  }
  .util-btn.active .q-badge {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
  }

  .volume-group {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-left: 2px;
  }
  .vol-scrubber {
    width: 80px;
    flex: none;
  }

  @media (max-width: 1000px) {
    .deck {
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-rows: 48px minmax(0, 1fr);
      gap: 6px 12px;
      padding: 8px 16px;
    }
    .transport {
      grid-column: 1 / -1;
      grid-row: 2;
      max-width: 760px;
    }
    .utilities {
      grid-column: 2;
      grid-row: 1;
      gap: 3px;
    }
    .identity {
      padding: 3px 8px 3px 4px;
      gap: 8px;
    }
    .artwork {
      width: 38px;
      height: 38px;
    }
    .meta {
      gap: 2px;
    }
    .subtitle {
      font-size: 11px;
    }
    .util-btn {
      padding: 0 7px;
    }
    .vol-scrubber {
      width: 52px;
    }
  }
  @media (max-height: 560px) and (max-width: 1000px) {
    .deck {
      grid-template-rows: 38px minmax(0, 1fr);
      padding-block: 6px;
      row-gap: 2px;
    }
    .artwork {
      width: 30px;
      height: 30px;
    }
    .play-fab {
      width: 32px;
      height: 32px;
    }
    .transport {
      gap: 0;
    }
    .controls {
      gap: 6px;
    }
  }
  @media (max-width: 600px) {
    .deck {
      padding-inline: 10px;
      column-gap: 8px;
    }
    .utilities {
      gap: 1px;
    }
    .util-btn,
    .util-btn.icon-only {
      width: 30px;
      padding: 0 4px;
    }
    .util-btn:has(.q-badge) {
      width: auto;
    }
    .q-badge {
      padding-inline: 4px;
    }
    .volume-group {
      gap: 2px;
      margin-left: 0;
    }
    .vol-scrubber {
      width: 42px;
    }
    .title {
      font-size: 12px;
    }
    .identity-group {
      gap: 4px;
    }
    .bar-like-btn {
      width: 28px;
    }
    .album,
    .sep,
    .stream-pill {
      display: none;
    }
  }
</style>
