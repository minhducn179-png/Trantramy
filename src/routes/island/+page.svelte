<script lang="ts">
  import { SkipBack, Pause, Play, SkipForward, Volume1, Music2, Waves, Minus, Pin, ArrowUpRight, Heart } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api, fmtTime, getTrackCover, handleImgError } from "$lib/api";
  import { theme, type IslandWaveMode, type UiConfig } from "$lib/settings/theme.svelte";
  import WaveformBar from "$lib/audio/WaveformBar.svelte";
  import AppSpeakerLogo from "$lib/components/AppSpeakerLogo.svelte";

  interface IslandStatePayload {
    title: string;
    artist: string;
    album: string;
    cover_path: string | null;
    url: string | null;
    playing: boolean;
    position: number;
    duration: number;
    volume: number;
    liked: boolean;
    lyricLine: string;
    hasTrack: boolean;
  }

  let stateData = $state<IslandStatePayload>({
    title: "Trantramy",
    artist: "Tran Tra My",
    album: "",
    cover_path: null,
    url: null,
    playing: false,
    position: 0,
    duration: 0,
    volume: 0.9,
    liked: false,
    lyricLine: "",
    hasTrack: false,
  });

  let expanded = $state(false);
  let pinned = $state(false);
  let collapseTimer: ReturnType<typeof setTimeout> | null = null;
  let pillBars = $state<number[]>([3, 3, 3, 3, 3, 3, 3, 3, 3]);
  let scrollBars = $state<number[]>([3, 3, 3, 3, 3, 3, 3, 3, 3]);

  const STATIC_BASE_BARS = [6, 10, 15, 11, 18, 12, 16, 9, 6];

  const waveMode = $derived<IslandWaveMode>(theme.config.islandWaveMode ?? "bounce-progress");
  const progressRatio = $derived(
    stateData.duration > 0
      ? Math.min(1, Math.max(0, stateData.position / stateData.duration))
      : 0,
  );

  const WAVE_MODE_LABELS: Record<IslandWaveMode, string> = {
    "bounce-progress": "Nảy nhịp điệu + Sáng dần",
    "progress-static": "Sóng tĩnh · Sáng dần",
    "bounce-real": "Nảy nhịp điệu",
    "scroll-wave": "Sóng cuộn thời gian thực",
  };

  function cycleIslandWaveMode() {
    const order: IslandWaveMode[] = [
      "bounce-progress",
      "progress-static",
      "bounce-real",
      "scroll-wave",
    ];
    const curIdx = order.indexOf(waveMode);
    const next = order[(curIdx + 1) % order.length];
    theme.update({
      islandWaveMode: next,
      waveformBounce: next === "progress-static" ? "none" : "real-audio",
    });
  }

  function getBarDisplayHeight(idx: number): number {
    if (waveMode === "progress-static") {
      return STATIC_BASE_BARS[idx] ?? 10;
    }
    if (!stateData.playing) return 3;
    if (waveMode === "scroll-wave") {
      return scrollBars[idx] ?? 3;
    }
    return pillBars[idx] ?? 3;
  }

  function getBarFillPct(idx: number): number {
    if (waveMode === "bounce-real" || waveMode === "scroll-wave") {
      return stateData.playing ? 100 : 0;
    }
    const start = idx / 9;
    const end = (idx + 1) / 9;
    if (progressRatio >= end) return 100;
    if (progressRatio <= start) return 0;
    return Math.round(((progressRatio - start) / (end - start)) * 100);
  }

  const islandStyle = $derived.by(() => {
    const c = theme.config;
    const sync = c.islandThemeSync ?? true;
    const bg = sync ? "var(--bg-canvas)" : (c.islandBg || "#000000");
    const txt = sync ? "var(--text-primary)" : (c.islandText || "#ffffff");
    const op = Math.min(1, Math.max(0.4, c.islandOpacity ?? 1));
    const r = Math.min(44, Math.max(8, c.islandRadius ?? 38));
    let s = `--isl-bg:${bg};--isl-text:${txt};--isl-op:${op};--isl-r:${r}px;--isl-pill-r:${Math.min(r, 20)}px;`;
    if (!sync && c.islandAccent) s += `--isl-accent:${c.islandAccent};`;
    return s;
  });

  // Distinguish click vs drag on the compact pill
  let pointerDownPos: { x: number; y: number } | null = null;
  let didDragPill = false;

  function loadSnapshotFromStorage() {
    try {
      const raw = localStorage.getItem("dlc.island.snapshot.v1");
      if (raw) {
        const parsed = JSON.parse(raw);
        if (parsed && typeof parsed === "object") {
          stateData = { ...stateData, ...parsed };
        }
      }
    } catch {
      // ignore
    }
  }

  async function setExpandedState(next: boolean) {
    if (collapseTimer) {
      clearTimeout(collapseTimer);
      collapseTimer = null;
    }
    if (expanded === next) return;
    expanded = next;
    try {
      await api.setIslandExpanded(next);
    } catch {
      // ignore
    }
  }

  function handleMouseEnter() {
    // Only cancel pending collapse if already expanded; NEVER auto-expand on hover!
    if (collapseTimer) {
      clearTimeout(collapseTimer);
      collapseTimer = null;
    }
  }

  function handleMouseLeave() {
    if (!expanded || pinned) return;
    if (collapseTimer) clearTimeout(collapseTimer);
    collapseTimer = setTimeout(() => {
      if (expanded && !pinned) {
        setExpandedState(false);
      }
    }, 140);
  }

  function handlePillPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    pointerDownPos = { x: e.screenX, y: e.screenY };
    didDragPill = false;
  }

  function handlePillPointerMove(e: PointerEvent) {
    if (!pointerDownPos || didDragPill) return;
    const dx = Math.abs(e.screenX - pointerDownPos.x);
    const dy = Math.abs(e.screenY - pointerDownPos.y);
    if (dx > 4 || dy > 4) {
      didDragPill = true;
      pointerDownPos = null;
      try {
        getCurrentWindow().startDragging();
      } catch {
        // ignore
      }
    }
  }

  function handlePillPointerUp(e: PointerEvent) {
    if (e.button !== 0) return;
    const wasClick = pointerDownPos !== null && !didDragPill;
    pointerDownPos = null;
    didDragPill = false;
    if (wasClick) {
      setExpandedState(true);
    }
  }

  function handleCardDragDown(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest("button, input, .wave-wrap")) return;
    try {
      getCurrentWindow().startDragging();
    } catch {
      // ignore
    }
  }

  async function sendCmd(action: string, value?: number) {
    try {
      await emit("dlc-island-cmd", { action, value });
    } catch {
      // ignore
    }
  }

  const artUrl = $derived(
    stateData.hasTrack
      ? getTrackCover(stateData.cover_path, stateData.url, stateData.title, stateData.artist)
      : "",
  );

  onMount(() => {
    document.documentElement.classList.add("island-transparent-mode");
    document.body.classList.add("island-transparent-mode");
    theme.init();
    loadSnapshotFromStorage();

    const onStorage = (ev: StorageEvent) => {
      if (ev.key === "dlc.ui.config.v2" && ev.newValue) {
        try {
          theme.config = JSON.parse(ev.newValue);
          theme.applyToDom();
        } catch {
          // ignore
        }
      } else if (ev.key === "dlc.island.snapshot.v1") {
        loadSnapshotFromStorage();
      }
    };
    window.addEventListener("storage", onStorage);

    let unlisten: (() => void) | null = null;
    let unlistenSpectrum: (() => void) | null = null;
    let unlistenTheme: (() => void) | null = null;
    listen<IslandStatePayload>("dlc-island-state", (ev) => {
      if (ev.payload) {
        stateData = { ...stateData, ...ev.payload };
        if (!stateData.playing) {
          pillBars = [3, 3, 3, 3, 3, 3, 3, 3, 3];
          scrollBars = [3, 3, 3, 3, 3, 3, 3, 3, 3];
        }
      }
    }).then((fn) => {
      unlisten = fn;
    });

    listen<UiConfig>("dlc-theme-updated", (ev) => {
      if (ev.payload) {
        theme.config = ev.payload;
        theme.applyToDom();
      }
    }).then((fn) => {
      unlistenTheme = fn;
    });

    // Map 32 real-time WASAPI audio spectrum bins into the 9 pill bars
    const BAR_BIN_MAP = [22, 14, 8, 3, 1, 5, 10, 17, 25];
    listen<number[]>("dlc-audio-spectrum", (ev) => {
      if (!stateData.playing || !Array.isArray(ev.payload) || ev.payload.length < 28) {
        pillBars = [3, 3, 3, 3, 3, 3, 3, 3, 3];
        scrollBars = [3, 3, 3, 3, 3, 3, 3, 3, 3];
        return;
      }
      const spec = ev.payload;
      const mapped = BAR_BIN_MAP.map((binIdx) => {
        const v0 = spec[binIdx] || 0;
        const v1 = spec[Math.min(spec.length - 1, binIdx + 1)] || 0;
        const avg = (v0 + v1) * 0.5;
        const norm = Math.min(1, Math.max(0, avg / 255));
        return Math.round((3 + norm * 17) * 10) / 10;
      });
      pillBars = mapped;

      // Also push real-time energy into the left-to-right scrolling wave buffer
      const energyPeak = Math.max(mapped[3], mapped[4], mapped[5]);
      scrollBars = [energyPeak, ...scrollBars.slice(0, 8)];
    }).then((fn) => {
      unlistenSpectrum = fn;
    });

    emit("dlc-island-ready", true).catch(() => {});

    return () => {
      window.removeEventListener("storage", onStorage);
      unlisten?.();
      unlistenSpectrum?.();
      unlistenTheme?.();
      document.documentElement.classList.remove("island-transparent-mode");
      document.body.classList.remove("island-transparent-mode");
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="island-viewport"
  class:is-expanded={expanded}
  class:art-circle={theme.config.islandArtShape === "circle"}
  class:is-droplet={theme.config.waterDropletMode}
  class:is-apple-glass={theme.config.appleGlassMode}
  style={islandStyle}
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  {#if !expanded}
    <!-- Ultra-Compact Pill (136x40): ONLY Logo + Audio Waveform, Click to Expand, Drag to Move -->
    <button
      type="button"
      class="island-pill"
      class:is-playing={stateData.playing}
      onpointerdown={handlePillPointerDown}
      onpointermove={handlePillPointerMove}
      onpointerup={handlePillPointerUp}
      onkeydown={(e) => (e.key === "Enter" || e.key === " ") && setExpandedState(true)}
      title="Bấm để mở rộng · Kéo để di chuyển"
      aria-label="Mở rộng Dynamic Island"
    >
      <div class="pill-logo" class:pulse={stateData.playing} >
        {#if stateData.hasTrack && artUrl}
          <img
            src={artUrl}
            alt=""
            onerror={(e) => handleImgError(e, stateData.title, stateData.artist, stateData.url || undefined)}
          />
        {:else}
          <AppSpeakerLogo size={24} />
        {/if}
      </div>

      <div class="pill-wave" class:active={stateData.playing} aria-hidden="true">
        {#each [0, 1, 2, 3, 4, 5, 6, 7, 8] as idx}
          {@const h = getBarDisplayHeight(idx)}
          {@const fillPct = getBarFillPct(idx)}
          <span
            class="w-bar"
            class:filled={fillPct === 100}
            class:partial={fillPct > 0 && fillPct < 100}
            style="height: {h}px; --bar-fill: {fillPct}%"
          ></span>
        {/each}
      </div>
    </button>
  {:else}
    <!-- Expanded Mini Player Card (390x178): Shrinks automatically when mouse leaves -->
    <div class="island-card" onmousedown={handleCardDragDown}>
      <div class="ic-top">
        <button
          type="button"
          class="ic-art-btn"
          class:playing={stateData.playing}
          onclick={() => setExpandedState(false)}
          title="Bấm để thu nhỏ lại thành viên thuốc"
        >
          {#if stateData.hasTrack && artUrl}
            <img
              src={artUrl}
              alt=""
              onerror={(e) => handleImgError(e, stateData.title, stateData.artist, stateData.url || undefined)}
            />
          {:else}
            <div class="ic-art-logo">
              <AppSpeakerLogo size={32} />
            </div>
          {/if}
        </button>

        <div class="ic-meta">
          <div class="ic-title-row">
            <span class="ic-title" title={stateData.title}>{stateData.title}</span>
          </div>
          <span class="ic-artist" title={stateData.artist}>{stateData.artist}</span>
          {#if stateData.lyricLine}
            <span class="ic-lyric" title={stateData.lyricLine}><Music2 size={11} /> {stateData.lyricLine}</span>
          {:else}
            <span class="ic-lyric muted">
              {stateData.playing ? "Đang phát nhạc" : "Đang tạm dừng"}
            </span>
          {/if}
        </div>

        <div class="ic-window-btns">
          <button
            type="button"
            class="ic-mini-btn"
            class:active={waveMode !== "bounce-real"}
            onclick={cycleIslandWaveMode}
            title="Chế độ sóng: {WAVE_MODE_LABELS[waveMode]}"
          >
            <Waves size={13} />
          </button>
          <button
            type="button"
            class="ic-mini-btn"
            onclick={() => setExpandedState(false)}
            title="Thu nhỏ"
          >
            <Minus size={13} />
          </button>
          <button
            type="button"
            class="ic-mini-btn"
            class:active={pinned}
            onclick={() => (pinned = !pinned)}
            title={pinned ? "Bỏ ghim cửa sổ" : "Ghim cửa sổ"}
          >
            <Pin size={13} fill={pinned ? "currentColor" : "none"} />
          </button>
          <button
            type="button"
            class="ic-mini-btn"
            onclick={() => sendCmd("focus-main")}
            title="Mở ứng dụng chính"
          >
            <ArrowUpRight size={13} />
          </button>
        </div>
      </div>

      <div class="ic-wave-row">
        <span class="ic-time active">{fmtTime(stateData.position)}</span>
        <div class="wave-wrap">
          <WaveformBar
            seed={`island:${stateData.title}:${stateData.duration}`}
            position={stateData.position}
            duration={Math.max(1, stateData.duration)}
            playing={stateData.playing}
            barsCount={40}
            height={22}
            onseek={(sec) => sendCmd("seek", sec)}
          />
        </div>
        <span class="ic-time">{fmtTime(stateData.duration)}</span>
      </div>

      <div class="ic-bottom">
        <button
          type="button"
          class="ic-ctrl-btn like"
          class:liked={stateData.liked}
          onclick={() => sendCmd("toggle-like")}
          title={stateData.liked ? "Bỏ thích" : "Yêu thích"}
        >
          <Heart size={15} fill={stateData.liked ? "currentColor" : "none"} />
        </button>

        <div class="ic-transport">
          <button
            type="button"
            class="ic-ctrl-btn"
            onclick={() => sendCmd("prev")}
            title="Bài trước"
            aria-label="Bài trước"
          >
            <SkipBack size={14} />
          </button>

          <button
            type="button"
            class="ic-play-fab"
            onclick={() => sendCmd("play-pause")}
            title={stateData.playing ? "Tạm dừng" : "Phát"}
            aria-label={stateData.playing ? "Tạm dừng" : "Phát"}
          >
            {#if stateData.playing}
              <Pause size={15} />
            {:else}
              <Play size={15} />
            {/if}
          </button>

          <button
            type="button"
            class="ic-ctrl-btn"
            onclick={() => sendCmd("next")}
            title="Bài tiếp theo"
            aria-label="Bài tiếp theo"
          >
            <SkipForward size={14} />
          </button>
        </div>

        <div class="ic-vol">
          <Volume1 size={13} />
          <input
            type="range"
            min="0"
            max="1"
            step="0.02"
            value={stateData.volume}
            aria-label="Âm lượng"
            oninput={(e) => sendCmd("volume", +e.currentTarget.value)}
          />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* ===== Apple-style Dynamic Island: pure black, continuous-curve, spring morph ===== */
  :global(html.island-transparent-mode),
  :global(body.island-transparent-mode) {
    background: transparent !important;
    overflow: hidden !important;
  }

  .island-viewport {
    width: 100vw;
    height: 100vh;
    padding: 0;
    box-sizing: border-box;
    display: flex;
    align-items: stretch;
    justify-content: center;
    background: transparent !important;
    user-select: none;
    overflow: hidden !important;
    --isl-bg: #000;
    --isl-text: #fff;
    --isl-op: 1;
    --isl-r: 38px;
    --isl-pill-r: 20px;
    --isl-accent: var(--accent, #4da3ff);
    color: var(--isl-text);
    -webkit-font-smoothing: antialiased;
  }

  /* Compact pill */
  .island-pill {
    width: 100%;
    height: 100%;
    border-radius: var(--isl-pill-r);
    background: color-mix(in srgb, var(--isl-bg) calc(var(--isl-op) * 100%), transparent);
    border: none;
    box-shadow: none !important;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 0 16px 0 7px;
    box-sizing: border-box;
    overflow: hidden;
    cursor: pointer;
    transform-origin: 50% 0%;
    animation: islandMorphIn 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
    transition: filter 0.2s ease;
  }
  .island-pill:hover {
    filter: brightness(1.15);
  }
  .island-pill:active {
    transform: scale(0.97);
  }

  .island-viewport.is-droplet .island-pill {
    border: 1px solid rgba(255, 255, 255, 0.35) !important;
    box-shadow: inset 0 1.5px 2px rgba(255, 255, 255, 0.5),
                inset 0 -1.5px 3px rgba(0, 0, 0, 0.35) !important;
    background: radial-gradient(circle at 40% 25%, rgba(255, 255, 255, 0.18) 0%, transparent 65%),
                color-mix(in srgb, var(--isl-bg) calc(var(--isl-op) * 100%), transparent) !important;
  }

  .island-viewport.is-droplet .island-card {
    border: 1px solid rgba(255, 255, 255, 0.32) !important;
    box-shadow: inset 0 2px 2.5px rgba(255, 255, 255, 0.45),
                inset 0 -2px 5px rgba(0, 0, 0, 0.35) !important;
    backdrop-filter: blur(28px) saturate(160%) !important;
  }

  /* Apple Liquid Glass dynamic island - strictly NO outer shadows or color bleed */
  .island-viewport.is-apple-glass .island-pill {
    border: 1px solid rgba(255, 255, 255, 0.25) !important;
    box-shadow: inset 0 1.5px 2px rgba(255, 255, 255, 0.4),
                inset 0 -1.5px 2px rgba(0, 0, 0, 0.3) !important;
    backdrop-filter: blur(40px) saturate(200%) !important;
  }

  .island-viewport.is-apple-glass .island-card {
    border: 1px solid rgba(255, 255, 255, 0.22) !important;
    box-shadow: inset 0 1.8px 2px rgba(255, 255, 255, 0.35),
                inset 0 -1.8px 3px rgba(0, 0, 0, 0.3) !important;
    backdrop-filter: blur(48px) saturate(210%) contrast(105%) !important;
  }

  /* Transparent Liquid Glass buttons in Dynamic Island */
  .island-viewport.is-apple-glass .ic-play-fab {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.24) 0%, rgba(255, 255, 255, 0.08) 100%) !important;
    border: 1px solid rgba(255, 255, 255, 0.32) !important;
    box-shadow: inset 0 1.5px 2px rgba(255, 255, 255, 0.45),
                inset 0 -1px 2px rgba(0, 0, 0, 0.2) !important;
    color: #ffffff !important;
    backdrop-filter: blur(16px) !important;
  }
  .island-viewport.is-apple-glass .ic-play-fab:hover {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.36) 0%, rgba(255, 255, 255, 0.14) 100%) !important;
    border-color: rgba(255, 255, 255, 0.5) !important;
    transform: scale(1.05) !important;
  }
  .island-viewport.is-apple-glass .ic-mini-btn {
    background: rgba(255, 255, 255, 0.07) !important;
    border: 1px solid rgba(255, 255, 255, 0.16) !important;
    box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.2) !important;
    backdrop-filter: blur(14px) !important;
    color: rgba(255, 255, 255, 0.85) !important;
  }
  .island-viewport.is-apple-glass .ic-mini-btn:hover {
    background: rgba(255, 255, 255, 0.16) !important;
    border-color: rgba(255, 255, 255, 0.35) !important;
    color: #ffffff !important;
  }
  .island-viewport.is-apple-glass .ic-mini-btn.active {
    background: linear-gradient(135deg, color-mix(in srgb, var(--isl-accent) 45%, rgba(255, 255, 255, 0.28)) 0%, color-mix(in srgb, var(--isl-accent) 25%, rgba(255, 255, 255, 0.08)) 100%) !important;
    border-color: rgba(255, 255, 255, 0.35) !important;
    color: #ffffff !important;
  }
  .island-viewport.is-apple-glass .ic-ctrl-btn:hover {
    background: rgba(255, 255, 255, 0.1) !important;
    backdrop-filter: blur(12px) !important;
  }

  @keyframes islandMorphIn {
    0% { transform: scale(0.85, 0.75); opacity: 0.5; }
    100% { transform: scale(1); opacity: 1; }
  }
  @keyframes islandCardIn {
    0% { transform: scale(0.7, 0.6); opacity: 0.4; }
    100% { transform: scale(1); opacity: 1; }
  }

  .pill-logo {
    width: 27px;
    height: 27px;
    border-radius: 9px;
    overflow: hidden;
    background: color-mix(in srgb, var(--isl-text) 12%, var(--isl-bg));
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }
  .pill-logo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    pointer-events: none;
  }
  .pill-logo.pulse {
    animation: artBreath 1.6s ease-in-out infinite;
  }
  @keyframes artBreath {
    50% { transform: scale(0.92); }
  }

  .pill-wave {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2.5px;
    height: 20px;
  }
  .w-bar {
    display: block;
    width: 3px;
    height: 3px;
    border-radius: 99px;
    background: color-mix(in srgb, var(--isl-text) 28%, transparent);
    transition: height 0.06s linear, background 0.15s ease;
  }
  .w-bar.filled {
    background: var(--isl-accent);
  }
  .w-bar.partial {
    background: linear-gradient(
      90deg,
      var(--isl-accent) var(--bar-fill, 50%),
      color-mix(in srgb, var(--isl-text) 28%, transparent) var(--bar-fill, 50%)
    );
  }

  /* Expanded card */
  .island-card {
    width: 100%;
    height: 100%;
    border-radius: var(--isl-r);
    background: color-mix(in srgb, var(--isl-bg) calc(var(--isl-op) * 100%), transparent);
    border: none;
    box-shadow: none !important;
    padding: 14px 20px 12px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 4px;
    overflow: hidden;
    cursor: grab;
    transform-origin: 50% 0%;
    animation: islandCardIn 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
  }
  .island-card:active {
    cursor: grabbing;
  }

  .ic-top {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .ic-art-btn {
    width: 52px;
    height: 52px;
    border-radius: 14px;
    overflow: hidden;
    border: none;
    background: color-mix(in srgb, var(--isl-text) 12%, var(--isl-bg));
    padding: 0;
    flex-shrink: 0;
    cursor: pointer;
    transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .ic-art-btn:hover { transform: scale(1.06); }
  .ic-art-btn:active { transform: scale(0.94); }
  .ic-art-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .art-circle .pill-logo,
  .art-circle .ic-art-btn {
    border-radius: 50%;
  }
  .art-circle .pill-logo.pulse img,
  .art-circle .ic-art-btn.playing img {
    animation: islandVinylSpin 3s linear infinite;
  }
  @keyframes islandVinylSpin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  .ic-art-logo {
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
  }

  .ic-meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .ic-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--isl-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    letter-spacing: -0.01em;
  }
  .ic-artist {
    font-size: 12.5px;
    font-weight: 500;
    color: color-mix(in srgb, var(--isl-text) 58%, transparent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ic-lyric {
    font-size: 11px;
    font-weight: 500;
    color: var(--isl-accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 1px;
  }
  .ic-lyric.muted {
    color: color-mix(in srgb, var(--isl-text) 34%, transparent);
  }

  .ic-window-btns {
    display: flex;
    align-items: center;
    gap: 5px;
    align-self: flex-start;
    flex-shrink: 0;
  }
  .ic-mini-btn {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: none;
    background: color-mix(in srgb, var(--isl-text) 14%, transparent);
    color: color-mix(in srgb, var(--isl-text) 75%, transparent);
    font-size: 11px;
    font-weight: 700;
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: background 0.15s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .ic-mini-btn:hover {
    background: color-mix(in srgb, var(--isl-text) 24%, transparent);
    color: var(--isl-text);
    transform: scale(1.1);
  }
  .ic-mini-btn.active {
    background: var(--isl-accent);
    color: var(--isl-bg);
  }

  .ic-wave-row {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
  }
  .wave-wrap {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    border-radius: 6px;
  }
  .ic-time {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10px;
    color: color-mix(in srgb, var(--isl-text) 42%, transparent);
    min-width: 30px;
    text-align: center;
  }
  .ic-time.active {
    color: color-mix(in srgb, var(--isl-text) 70%, transparent);
    font-weight: 600;
  }

  .ic-bottom {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .ic-transport {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .ic-ctrl-btn {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: none;
    background: transparent;
    color: var(--isl-text);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: background 0.15s ease, transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .ic-ctrl-btn:hover {
    background: color-mix(in srgb, var(--isl-text) 16%, transparent);
  }
  .ic-ctrl-btn:active {
    transform: scale(0.92);
  }
  .ic-ctrl-btn.like {
    color: color-mix(in srgb, var(--isl-text) 60%, transparent);
  }
  .ic-ctrl-btn.like.liked {
    color: #ff453a;
  }

  .ic-play-fab {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    border: none;
    background: var(--isl-text);
    color: var(--isl-bg);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .ic-play-fab:hover { transform: scale(1.05); }
  .ic-play-fab:active { transform: scale(0.92); }

  .ic-vol {
    display: flex;
    align-items: center;
    gap: 6px;
    color: color-mix(in srgb, var(--isl-text) 60%, transparent);
    width: 84px;
    flex-shrink: 0;
  }
  .ic-vol input[type="range"] {
    width: 100%;
    accent-color: var(--isl-accent);
    cursor: pointer;
    height: 4px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--isl-text) 20%, transparent);
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
    appearance: none;
  }
  .ic-vol input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--isl-accent);
    border: none !important;
    outline: none !important;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5) !important;
    cursor: pointer;
  }
</style>
