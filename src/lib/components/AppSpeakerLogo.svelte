<script lang="ts">
  import Volume2 from "@lucide/svelte/icons/volume-2";
  import { theme, oklchApproxToHex } from "$lib/settings/theme.svelte";
  import { player } from "$lib/player/player.svelte";
  import { readableColor } from "$lib/settings/palettes";
  import { invoke } from "@tauri-apps/api/core";

  let {
    size = 40,
    interactive = false,
    syncNativeIcon = false,
    onclick,
    title = "Trantramy — Biểu tượng loa theo phong cách giao diện",
  }: {
    size?: number;
    interactive?: boolean;
    syncNativeIcon?: boolean;
    onclick?: () => void;
    title?: string;
  } = $props();

  let svgEl = $state<SVGSVGElement>();
  const uid = $props.id();
  const cfg = $derived(theme.config);
  const palette = $derived(theme.activePalette);
  const isLight = $derived(palette ? palette.scheme === "light" : cfg.colorScheme === "light");
  const accentHex = $derived(
    palette ? readableColor(palette.colors.accent, [palette.colors.base, palette.colors.surface], 3) :
      oklchApproxToHex(cfg.accentLightness, cfg.accentChroma, cfg.accentHue),
  );
  const accentBrightHex = $derived(
    palette ? accentHex : oklchApproxToHex(
      Math.min(0.93, cfg.accentLightness + 0.14),
      Math.min(0.26, cfg.accentChroma + 0.03),
      cfg.accentHue,
    ),
  );
  const secondaryHex = $derived(
    palette ? accentHex : cfg.bgGradient !== "flat"
      ? oklchApproxToHex(cfg.accentLightness, Math.max(0.14, cfg.accentChroma), cfg.bgHue2)
      : oklchApproxToHex(
          Math.max(0.48, cfg.accentLightness - 0.1),
          cfg.accentChroma,
          (cfg.accentHue + 28) % 360,
        ),
  );
  const bgOuterHex = $derived.by(() => {
    if (palette) return palette.colors.base;
    if (isLight) return oklchApproxToHex(0.94, Math.min(0.03, cfg.bgChroma), cfg.bgHue);
    if (cfg.canvasMode === "oled") return "#06080c";
    if (cfg.canvasMode === "graphite") return "#14171c";
    if (cfg.canvasMode === "warm-noir") return "#181411";
    return oklchApproxToHex(
      Math.max(0.09, Math.min(0.22, cfg.bgLightness)),
      Math.min(0.08, cfg.bgChroma),
      cfg.bgHue,
    );
  });
  const bgInnerHex = $derived.by(() => {
    if (palette) return palette.colors.surface;
    if (isLight) return oklchApproxToHex(0.98, Math.min(0.02, cfg.bgChroma), cfg.bgHue);
    const l = Math.max(0.15, Math.min(0.28, cfg.bgLightness + 0.06));
    const h = cfg.bgGradient !== "flat" ? cfg.bgHue2 : cfg.bgHue;
    const c = cfg.bgGradient !== "flat"
      ? Math.max(0.04, cfg.bgChroma + cfg.bgGradientStrength * 0.05)
      : Math.min(0.08, cfg.bgChroma + 0.015);
    return oklchApproxToHex(l, c, h);
  });
  const badgeRx = $derived(
    cfg.radiusStyle === "sharp" ? 12 : cfg.radiusStyle === "soft" ? 30 : 21,
  );

  // Use the same Lucide paths for the UI, taskbar and system tray.
  $effect(() => {
    if (!syncNativeIcon) return;
    const colors = [bgInnerHex, bgOuterHex, accentBrightHex, accentHex, secondaryHex];
    const radius = badgeRx;
    const timer = setTimeout(() => {
      try {
        const iconSize = 128;
        const canvas = document.createElement("canvas");
        canvas.width = canvas.height = iconSize;
        const ctx = canvas.getContext("2d");
        if (!ctx || !svgEl) return;
        ctx.scale(iconSize / 100, iconSize / 100);
        const bg = ctx.createLinearGradient(0, 0, 100, 100);
        bg.addColorStop(0, colors[0]);
        bg.addColorStop(1, colors[1]);
        const accent = ctx.createLinearGradient(0, 0, 100, 100);
        accent.addColorStop(0, colors[2]);
        accent.addColorStop(0.55, colors[3]);
        accent.addColorStop(1, colors[4]);
        ctx.beginPath();
        ctx.roundRect(4, 4, 92, 92, radius);
        ctx.fillStyle = bg;
        ctx.fill();
        ctx.strokeStyle = accent;
        ctx.lineWidth = 3.2;
        ctx.stroke();
        ctx.translate(16, 16);
        ctx.scale(68 / 24, 68 / 24);
        ctx.lineWidth = 1.75;
        ctx.lineCap = ctx.lineJoin = "round";
        for (const path of svgEl.querySelectorAll(".speaker-mark path")) {
          const d = path.getAttribute("d");
          if (d) ctx.stroke(new Path2D(d));
        }
        invoke("set_app_icon", {
          rgba: Array.from(ctx.getImageData(0, 0, iconSize, iconSize).data),
          width: iconSize,
          height: iconSize,
        }).catch(() => {});
      } catch {
        // Native icon synchronization is unavailable outside Tauri.
      }
    }, 150);
    return () => clearTimeout(timer);
  });
</script>

{#snippet speakerSvg()}
  <svg
    bind:this={svgEl}
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 100 100"
    width={size}
    height={size}
    class="speaker-icon-svg"
    class:playing={player.playing}
    class:glow={cfg.ambientGlow}
    aria-hidden="true"
  >
    <defs>
      <linearGradient id="{uid}-bg" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color={bgInnerHex} />
        <stop offset="100%" stop-color={bgOuterHex} />
      </linearGradient>
      <linearGradient id="{uid}-acc" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stop-color={accentBrightHex} />
        <stop offset="55%" stop-color={accentHex} />
        <stop offset="100%" stop-color={secondaryHex} />
      </linearGradient>
    </defs>
    <rect x="4" y="4" width="92" height="92" rx={badgeRx}
      fill="url(#{uid}-bg)" stroke="url(#{uid}-acc)" stroke-width="3.2" />
    <g class="speaker-mark">
      <Volume2 x={16} y={16} size={68} stroke="url(#{uid}-acc)" strokeWidth={1.75} />
    </g>
  </svg>
{/snippet}

{#if interactive}
  <button type="button" class="speaker-logo-btn" {onclick} {title} aria-label={title}>
    {@render speakerSvg()}
  </button>
{:else}
  <div class="speaker-logo-wrap" {title}>
    {@render speakerSvg()}
  </div>
{/if}

<style>
  .speaker-logo-wrap,
  .speaker-logo-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    padding: 0;
    border: none;
    background: transparent;
  }
  .speaker-logo-btn {
    cursor: pointer;
    border-radius: 12px;
    transition: transform 0.18s ease;
  }
  .speaker-logo-btn:hover { transform: scale(1.06); }
  .speaker-icon-svg {
    display: block;
    overflow: visible;
    transition: filter 0.22s ease;
  }
  .speaker-icon-svg.glow { filter: drop-shadow(0 4px 12px var(--accent-glow)); }
  .speaker-icon-svg.playing .speaker-mark :global(path:not(:first-child)) {
    animation: speaker-wave 0.6s ease-in-out infinite alternate;
  }
  @keyframes speaker-wave {
    from { opacity: 0.68; }
    to { opacity: 1; }
  }
</style>
