<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { fmtTime } from "$lib/api";
  import { theme, type WaveformStyle, type WaveformBounceMode } from "$lib/settings/theme.svelte";

  let {
    seed = "dlc-track",
    position = 0,
    duration = 0,
    playing = false,
    barsCount,
    styleOverride,
    bounceOverride,
    height = 38,
    onseek,
  }: {
    seed?: string;
    position?: number;
    duration?: number;
    playing?: boolean;
    barsCount?: number;
    styleOverride?: WaveformStyle;
    bounceOverride?: WaveformBounceMode;
    height?: number;
    onseek?: (seconds: number) => void;
  } = $props();

  const effectiveBars = $derived(barsCount ?? theme.config.waveformBars ?? 68);
  const wfStyle = $derived(styleOverride ?? theme.config.waveformStyle ?? "mirrored");
  const bounceMode = $derived(bounceOverride ?? theme.config.waveformBounce ?? "none");

  let wrapEl: HTMLDivElement | undefined = $state();
  let hoverRatio = $state<number | null>(null);
  let dragging = $state(false);
  let liveSpec = $state<number[]>([]);

  onMount(() => {
    let unlisten: (() => void) | null = null;
    listen<number[]>("dlc-audio-spectrum", (ev) => {
      if (Array.isArray(ev.payload)) {
        liveSpec = ev.payload;
      }
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {});
    return () => {
      unlisten?.();
    };
  });

  // Deterministic musical waveform generator (SoundCloud-style dynamic range with verses, drops & chorus peaks)
  const peaks = $derived.by(() => {
    const s = seed || "default-wave";
    let h = 2166136261 >>> 0;
    for (let i = 0; i < s.length; i++) {
      h ^= s.charCodeAt(i);
      h = Math.imul(h, 16777619) >>> 0;
    }
    const out: number[] = [];
    for (let i = 0; i < effectiveBars; i++) {
      h ^= (i + 1) * 2654435761;
      h = Math.imul(h, 1597334677) >>> 0;
      const rand = (h % 1000) / 1000;
      const t = i / Math.max(1, effectiveBars - 1);
      const introOutro = Math.sin(t * Math.PI);
      const phrase = 0.72 + 0.28 * Math.sin(t * Math.PI * 6 + (h % 7));
      const val = Math.max(
        0.16,
        Math.min(1, (0.26 + rand * 0.74) * (0.35 + 0.65 * introOutro) * phrase),
      );
      out.push(val);
    }
    return out;
  });

  // Smooth SVG path builder for "neon-curve" and "dual-neon"
  const svgPaths = $derived.by(() => {
    const pts = peaks;
    const n = pts.length;
    if (n < 2) return { topLine: "", topArea: "", dualLine: "", dualArea: "" };

    const W = 1000;
    const H = 100;

    let dLine = "";
    let dDual = "";

    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1)) * W;
      const y = H - pts[i] * 82 - 6;
      const p2 = pts[(i + 3) % n];
      const yDual = 50 + (i % 2 === 0 ? -1 : 1) * (pts[i] * 0.6 + p2 * 0.4) * 44;

      if (i === 0) {
        dLine += `M ${x.toFixed(1)} ${y.toFixed(1)}`;
        dDual += `M ${x.toFixed(1)} ${yDual.toFixed(1)}`;
      } else {
        const prevX = ((i - 1) / (n - 1)) * W;
        const prevY = H - pts[i - 1] * 82 - 6;
        const cx = (prevX + x) / 2;
        const cy = (prevY + y) / 2;
        dLine += ` Q ${prevX.toFixed(1)} ${prevY.toFixed(1)}, ${cx.toFixed(1)} ${cy.toFixed(1)}`;

        const prevP2 = pts[(i + 2) % n];
        const prevYDual =
          50 + ((i - 1) % 2 === 0 ? -1 : 1) * (pts[i - 1] * 0.6 + prevP2 * 0.4) * 44;
        const cx2 = (prevX + x) / 2;
        const cy2 = (prevYDual + yDual) / 2;
        dDual += ` Q ${prevX.toFixed(1)} ${prevYDual.toFixed(1)}, ${cx2.toFixed(1)} ${cy2.toFixed(1)}`;
      }
    }
    const lastY = H - pts[n - 1] * 82 - 6;
    dLine += ` L ${W} ${lastY.toFixed(1)}`;
    const dArea = `${dLine} L ${W} ${H} L 0 ${H} Z`;

    dDual += ` L ${W} 50`;
    const dDualArea = `${dDual} L ${W} 50 L 0 50 Z`;

    return {
      topLine: dLine,
      topArea: dArea,
      dualLine: dDual,
      dualArea: dDualArea,
    };
  });

  const progressRatio = $derived(
    duration > 0 ? Math.min(1, Math.max(0, position / duration)) : 0,
  );

  const hoverTime = $derived(
    hoverRatio !== null && duration > 0 ? fmtTime(hoverRatio * duration) : null,
  );

  function ratioFromEvent(e: MouseEvent | PointerEvent): number {
    if (!wrapEl) return 0;
    const rect = wrapEl.getBoundingClientRect();
    if (rect.width <= 0) return 0;
    return Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
  }

  function handlePointerDown(e: PointerEvent) {
    if (!onseek || duration <= 0) return;
    dragging = true;
    wrapEl?.setPointerCapture(e.pointerId);
    const r = ratioFromEvent(e);
    hoverRatio = r;
    onseek(r * duration);
  }

  function handlePointerMove(e: PointerEvent) {
    const r = ratioFromEvent(e);
    hoverRatio = r;
    if (dragging && onseek && duration > 0) {
      onseek(r * duration);
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (dragging) {
      dragging = false;
      wrapEl?.releasePointerCapture(e.pointerId);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!onseek || duration <= 0) return;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      onseek(Math.min(duration, position + 5));
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      onseek(Math.max(0, position - 5));
    }
  }
</script>

<div
  class="sc-waveform style-{wfStyle}"
  class:is-playing={playing}
  style="--wf-height: {height}px"
  bind:this={wrapEl}
  role="slider"
  tabindex="0"
  aria-label="Sóng nhạc tua thời gian"
  aria-valuemin={0}
  aria-valuemax={Math.round(duration || 0)}
  aria-valuenow={Math.round(position || 0)}
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  onpointerleave={() => {
    if (!dragging) hoverRatio = null;
  }}
  onkeydown={handleKeyDown}
>
  {#if wfStyle === "line"}
    <!-- 1. Classic Single Flat Progress Bar -->
    <div class="classic-line-track">
      {#if hoverRatio !== null}
        <div
          class="classic-line-hover"
          style="width: {Math.round(hoverRatio * 1000) / 10}%"
        ></div>
      {/if}
      <div
        class="classic-line-fill"
        style="width: {Math.round(progressRatio * 1000) / 10}%"
      ></div>
      <div
        class="classic-line-thumb"
        style="left: {Math.round(progressRatio * 1000) / 10}%"
      ></div>
    </div>
  {:else if wfStyle === "neon-curve" || wfStyle === "dual-neon"}
    <!-- 2. Smooth SVG Liquid / Stereo Neon Curves -->
    <div class="svg-wave-wrap">
      <svg class="svg-wave base" viewBox="0 0 1000 100" preserveAspectRatio="none">
        {#if wfStyle === "dual-neon"}
          <line x1="0" y1="50" x2="1000" y2="50" class="center-axis" />
          <path d={svgPaths.dualArea} class="curve-area unplayed" />
          <path d={svgPaths.dualLine} class="curve-stroke unplayed" />
          <path d={svgPaths.topLine} class="curve-stroke secondary unplayed" />
        {:else}
          <path d={svgPaths.topArea} class="curve-area unplayed" />
          <path d={svgPaths.topLine} class="curve-stroke unplayed" />
        {/if}
      </svg>

      <div
        class="svg-played-clip"
        style="width: {Math.round(progressRatio * 1000) / 10}%"
      >
        <svg
          class="svg-wave active"
          viewBox="0 0 1000 100"
          preserveAspectRatio="none"
          style="width: {wrapEl ? `${wrapEl.clientWidth}px` : '100%'}"
        >
          {#if wfStyle === "dual-neon"}
            <path d={svgPaths.dualArea} class="curve-area played" />
            <path d={svgPaths.dualLine} class="curve-stroke played" />
            <path d={svgPaths.topLine} class="curve-stroke secondary played" />
          {:else}
            <path d={svgPaths.topArea} class="curve-area played" />
            <path d={svgPaths.topLine} class="curve-stroke played" />
          {/if}
        </svg>
      </div>

      <div
        class="curve-playhead"
        style="left: {Math.round(progressRatio * 1000) / 10}%"
      ></div>
    </div>
  {:else}
    <!-- 3. Column / Bar / Dot / LED / Retro / Symmetric Waveforms -->
    {#if wfStyle === "center-sym"}
      <div class="sym-axis-line"></div>
    {/if}
    <div class="bars-layer">
      {#each peaks as basePeak, idx}
        {@const barStart = idx / effectiveBars}
        {@const barEnd = (idx + 1) / effectiveBars}
        {@const barPos = (idx + 0.5) / effectiveBars}
        {@const isPlayed = barEnd <= progressRatio}
        {@const isCurrentHead =
          progressRatio > barStart && progressRatio < barEnd}
        {@const partFill = isCurrentHead
          ? Math.round(((progressRatio - barStart) / Math.max(0.0001, barEnd - barStart)) * 100)
          : isPlayed
            ? 100
            : 0}
        {@const isNearHead =
          playing &&
          bounceMode === "playhead" &&
          Math.abs(barPos - progressRatio) < 2.2 / effectiveBars}
        {@const specVal =
          playing && bounceMode === "real-audio" && liveSpec.length > 0
            ? (liveSpec[Math.min(liveSpec.length - 1, Math.floor((idx / Math.max(1, effectiveBars - 1)) * (liveSpec.length - 1)))] || 0) / 255
            : 0}
        {@const peak =
          playing && bounceMode === "real-audio" && liveSpec.length > 0
            ? Math.max(0.14, Math.min(1, basePeak * 0.45 + specVal * 0.72))
            : basePeak}
        {@const isHovered =
          hoverRatio !== null &&
          ((barPos > progressRatio && barPos <= hoverRatio) ||
            (barPos <= progressRatio && barPos >= hoverRatio))}
        <div
          class="wf-col"
          class:played={isPlayed}
          class:partial-played={isCurrentHead}
          class:hovered={isHovered}
          class:near-head={isNearHead}
          style="--peak: {Math.round(peak * 100)}%; --part-pct: {partFill}%; --col-idx: {idx % 5}"
        >
          {#if wfStyle === "led-matrix" || wfStyle === "blocks-retro"}
            <span class="peak-cap"></span>
          {/if}
          <span class="bar-top"></span>
          {#if wfStyle === "mirrored" || wfStyle === "sc-fine" || wfStyle === "flame-peak" || wfStyle === "dots-pulse"}
            <span class="bar-bot"></span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  {#if hoverRatio !== null && hoverTime}
    <div class="hover-pill" style="left: {hoverRatio * 100}%">
      {hoverTime}
    </div>
  {/if}
</div>

<style>
  .sc-waveform {
    position: relative;
    width: 100%;
    height: var(--wf-height, 38px);
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;
    touch-action: none;
    outline: none;
  }

  .bars-layer {
    width: 100%;
    height: 100%;
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 1fr;
    gap: 2px;
    align-items: center;
    position: relative;
    z-index: 2;
  }

  .wf-col {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1.5px;
    position: relative;
  }

  /* Live playhead bounce when playing */
  .sc-waveform.is-playing .wf-col.near-head .bar-top {
    animation: headPulse 0.48s ease-in-out infinite alternate;
  }
  @keyframes headPulse {
    from {
      filter: brightness(1.05);
      transform: scaleY(0.94);
    }
    to {
      filter: brightness(1.3);
      transform: scaleY(1.1);
    }
  }

  /* Upper main bar (68% of total height for mirrored) */
  .bar-top {
    display: block;
    width: 100%;
    height: calc(var(--peak) * 0.66);
    min-height: 2px;
    border-radius: 2px 2px 0 0;
    background: var(--wave-idle, oklch(0.32 0.03 240));
    transition: background 0.12s ease, transform 0.15s ease;
    margin-top: auto;
  }

  /* Lower SoundCloud mirrored reflection bar (32% of total height) */
  .bar-bot {
    display: block;
    width: 100%;
    height: calc(var(--peak) * 0.3);
    min-height: 1.5px;
    border-radius: 0 0 2px 2px;
    background: var(--wave-idle-2, oklch(0.24 0.025 240 / 0.65));
    transition: background 0.12s ease;
    margin-bottom: auto;
  }

  .sc-waveform:hover .wf-col:not(.played) .bar-top {
    background: var(--wave-idle-hover, oklch(0.4 0.04 240));
  }

  /* Hovered scrub preview */
  .wf-col.hovered:not(.played) .bar-top {
    background: var(--accent-hover);
    opacity: 0.65;
  }
  .wf-col.hovered:not(.played) .bar-bot {
    background: var(--accent);
    opacity: 0.35;
  }

  /* Played bars — Dynamic Theme Accent */
  .wf-col.played .bar-top {
    background: linear-gradient(180deg, var(--accent-hover), var(--accent));
    box-shadow: 0 0 8px var(--accent-glow);
  }
  .wf-col.played .bar-bot {
    background: var(--accent);
    opacity: 0.42;
  }

  /* Partial played bar right at the current playhead for smooth left-to-right fill */
  .wf-col.partial-played .bar-top {
    background: linear-gradient(
      90deg,
      var(--accent) var(--part-pct, 50%),
      var(--wave-idle, oklch(0.32 0.03 240)) var(--part-pct, 50%)
    );
  }
  .wf-col.partial-played .bar-bot {
    background: linear-gradient(
      90deg,
      var(--accent) var(--part-pct, 50%),
      var(--wave-idle-2, oklch(0.24 0.025 240 / 0.65)) var(--part-pct, 50%)
    );
    opacity: 0.55;
  }

  /* ── Style Variant: single (1-layer upward bars) ── */
  .sc-waveform.style-single .bar-top {
    height: calc(var(--peak) * 0.88);
    margin: auto 0 2px;
    border-radius: 999px;
  }

  /* ── Style Variant: center-sym (Studio DAW 50/50 Symmetric) ── */
  .sym-axis-line {
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    height: 1px;
    background: var(--accent-soft);
    z-index: 1;
    pointer-events: none;
  }
  .sc-waveform.style-center-sym .bar-top {
    height: calc(var(--peak) * 0.92);
    margin: auto 0;
    border-radius: 999px;
  }

  /* ── Style Variant: bars-thick (Sub-Bass Wide Columns) ── */
  .sc-waveform.style-bars-thick .bars-layer {
    gap: 3px;
  }
  .sc-waveform.style-bars-thick .bar-top {
    height: calc(var(--peak) * 0.92);
    margin: auto 0;
    border-radius: 3px;
  }

  /* ── Style Variant: led-matrix (Hi-Fi Segmented LED VU Meter) ── */
  .sc-waveform.style-led-matrix .wf-col {
    justify-content: flex-end;
    gap: 2px;
    padding-bottom: 2px;
  }
  .sc-waveform.style-led-matrix .bar-top {
    height: calc(var(--peak) * 0.82);
    margin: 0;
    border-radius: 1px;
    -webkit-mask-image: repeating-linear-gradient(
      to top,
      #000 0px,
      #000 3px,
      transparent 3px,
      transparent 5px
    );
    mask-image: repeating-linear-gradient(
      to top,
      #000 0px,
      #000 3px,
      transparent 3px,
      transparent 5px
    );
  }
  .peak-cap {
    display: block;
    width: 100%;
    height: 2.5px;
    border-radius: 1px;
    background: oklch(0.45 0.04 240);
    flex: none;
  }
  .wf-col.played .peak-cap {
    background: var(--accent-hover);
    box-shadow: 0 0 6px var(--accent);
  }

  /* ── Style Variant: blocks-retro (8-Bit Synthwave Pixel Blocks) ── */
  .sc-waveform.style-blocks-retro .bars-layer {
    gap: 3px;
  }
  .sc-waveform.style-blocks-retro .wf-col {
    justify-content: flex-end;
    gap: 3px;
    padding-bottom: 2px;
  }
  .sc-waveform.style-blocks-retro .bar-top {
    height: calc(var(--peak) * 0.78);
    margin: 0;
    border-radius: 0;
  }
  .sc-waveform.style-blocks-retro .peak-cap {
    height: 4px;
    border-radius: 0;
  }

  /* ── Style Variant: dots-pulse (Glowing Dot Matrix Wave) ── */
  .sc-waveform.style-dots-pulse .wf-col {
    align-items: center;
    justify-content: center;
    gap: 3px;
  }
  .sc-waveform.style-dots-pulse .bar-top {
    width: 5px;
    max-width: 100%;
    height: calc(6px + var(--peak) * 0.56);
    border-radius: 999px;
    margin-top: auto;
  }
  .sc-waveform.style-dots-pulse .bar-bot {
    width: 4px;
    height: 4px;
    min-height: 4px;
    border-radius: 50%;
    margin-bottom: auto;
  }

  /* ── Style Variant: flame-peak (Aurora Pointed Wave) ── */
  .sc-waveform.style-flame-peak .bars-layer {
    gap: 1.5px;
  }
  .sc-waveform.style-flame-peak .bar-top {
    height: calc(var(--peak) * 0.72);
    border-radius: 999px 999px 1px 1px;
  }
  .sc-waveform.style-flame-peak .wf-col.played .bar-top {
    background: linear-gradient(
      180deg,
      #ffffff 0%,
      var(--accent-hover) 28%,
      var(--accent) 100%
    );
    box-shadow: 0 0 12px var(--accent-glow);
  }
  .sc-waveform.style-flame-peak .bar-bot {
    height: calc(var(--peak) * 0.24);
    border-radius: 1px 1px 999px 999px;
    filter: blur(0.6px);
  }

  /* ── Style Variant: sc-fine (SoundCloud look, hairline bars, many columns) ── */
  .sc-waveform.style-sc-fine .bars-layer {
    gap: 1px;
  }
  .sc-waveform.style-sc-fine .wf-col {
    gap: 1px;
  }
  .sc-waveform.style-sc-fine .bar-top {
    height: calc(var(--peak) * 0.7);
    min-height: 1px;
    border-radius: 1px 1px 0 0;
  }
  .sc-waveform.style-sc-fine .bar-bot {
    height: calc(var(--peak) * 0.3);
    min-height: 1px;
    border-radius: 0 0 1px 1px;
  }
  .sc-waveform.style-sc-fine .wf-col.played .bar-top {
    background: var(--accent);
    box-shadow: none;
  }
  .sc-waveform.style-sc-fine .wf-col.played .bar-bot {
    background: var(--accent);
    opacity: 0.4;
  }

  /* ── Style Variant: minimal (Whisper Thin) ── */
  .sc-waveform.style-minimal .bar-top {
    height: calc(18% + var(--peak) * 0.32);
    margin: auto 0;
    border-radius: 999px;
  }

  /* ── Style Variant: neon-curve & dual-neon (Smooth SVG Waves) ── */
  .svg-wave-wrap {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    overflow: hidden;
  }
  .svg-wave {
    width: 100%;
    height: 100%;
    display: block;
    overflow: visible;
  }
  .svg-played-clip {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    overflow: hidden;
    pointer-events: none;
  }
  .center-axis {
    stroke: var(--border-subtle);
    stroke-width: 1;
    stroke-dasharray: 4 4;
  }
  .curve-area.unplayed {
    fill: var(--wave-idle-2, oklch(0.32 0.03 240 / 0.22));
  }
  .curve-stroke.unplayed {
    fill: none;
    stroke: var(--wave-idle-hover, oklch(0.42 0.04 240));
    stroke-width: 2.2;
    vector-effect: non-scaling-stroke;
  }
  .curve-stroke.secondary.unplayed {
    stroke: var(--wave-idle-2, oklch(0.34 0.03 240 / 0.55));
    stroke-width: 1.4;
  }
  .curve-area.played {
    fill: var(--accent-soft);
  }
  .curve-stroke.played {
    fill: none;
    stroke: var(--accent-hover);
    stroke-width: 2.5;
    vector-effect: non-scaling-stroke;
    filter: drop-shadow(0 0 6px var(--accent));
  }
  .curve-stroke.secondary.played {
    stroke: var(--accent);
    stroke-width: 1.6;
    opacity: 0.75;
  }
  .curve-playhead {
    position: absolute;
    top: 12%;
    bottom: 12%;
    width: 2px;
    background: var(--text-primary);
    box-shadow: 0 0 8px var(--accent);
    transform: translateX(-50%);
    pointer-events: none;
  }

  /* ── Style Variant: line ("Thanh đơn bình thường") ── */
  .classic-line-track {
    position: relative;
    width: 100%;
    height: 5px;
    border-radius: 999px;
    background: var(--bg-surface-3);
    overflow: visible;
    transition: height 0.16s ease;
  }
  .sc-waveform:hover .classic-line-track {
    height: 7px;
  }
  .classic-line-hover {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    border-radius: 999px;
    background: var(--accent-soft);
    pointer-events: none;
  }
  .classic-line-fill {
    position: relative;
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--accent), var(--accent-hover));
    box-shadow: 0 0 10px var(--accent-glow);
    pointer-events: none;
  }
  .classic-line-thumb {
    position: absolute;
    top: 50%;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: var(--text-primary);
    border: 2px solid var(--accent);
    box-shadow: 0 2px 8px oklch(0 0 0 / 0.65);
    transform: translate(-50%, -50%) scale(0);
    transition: transform 0.15s var(--ease-out-expo);
    pointer-events: none;
  }
  .sc-waveform:hover .classic-line-thumb {
    transform: translate(-50%, -50%) scale(1);
  }

  .hover-pill {
    position: absolute;
    top: -6px;
    transform: translate(-50%, -100%);
    background: var(--bg-surface-1);
    border: 1px solid var(--accent);
    color: var(--accent-hover);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    pointer-events: none;
    white-space: nowrap;
    z-index: 10;
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.6);
  }
</style>
