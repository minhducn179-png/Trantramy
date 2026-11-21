<script lang="ts">
  import { SlidersVertical, X, RotateCcw } from "@lucide/svelte";
  import {
    audioEngine,
    EQ_FREQUENCIES,
    EQ_PRESETS,
  } from "./audioEngine.svelte";
  import { player } from "./player.svelte";
  import { theme, oklchApproxToHex } from "./theme.svelte";

  let { open = $bindable(false) }: { open: boolean } = $props();

  let canvasEl: HTMLCanvasElement | null = $state(null);
  let animId = 0;
  const freqData = new Uint8Array(64);

  function fmtFreq(f: number): string {
    if (f >= 1000) return `${f / 1000}k`;
    return `${f}`;
  }

  function handleGainInput(idx: number, e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    audioEngine.setGain(idx, parseFloat(target.value));
  }

  function handlePreAmpInput(e: Event) {
    const target = e.currentTarget as HTMLInputElement;
    audioEngine.setPreAmp(parseFloat(target.value));
  }

  // Draw real-time spectrum and EQ curve matching main UI theme
  function drawVisualizer() {
    if (!canvasEl) {
      animId = requestAnimationFrame(drawVisualizer);
      return;
    }
    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;

    const w = canvasEl.width;
    const h = canvasEl.height;

    const accentHex = theme.activePalette?.colors.accent ?? oklchApproxToHex(
      theme.config.accentLightness,
      theme.config.accentChroma,
      theme.config.accentHue,
    );
    const accentGlowHex = theme.activePalette?.colors.accent ?? oklchApproxToHex(
      Math.min(0.86, theme.config.accentLightness + 0.08),
      theme.config.accentChroma,
      (theme.config.accentHue + 25) % 360,
    );
    const isLight =
      typeof document !== "undefined" &&
      document.documentElement.dataset.scheme === "light";

    ctx.clearRect(0, 0, w, h);

    // 1. Grid lines
    ctx.strokeStyle = isLight ? "rgba(0, 0, 0, 0.08)" : "rgba(255, 255, 255, 0.07)";
    ctx.lineWidth = 1;
    // 0dB center line
    const zeroY = h / 2;
    ctx.beginPath();
    ctx.moveTo(0, zeroY);
    ctx.lineTo(w, zeroY);
    ctx.stroke();

    // +6dB and -6dB lines
    ctx.strokeStyle = isLight ? "rgba(0, 0, 0, 0.04)" : "rgba(255, 255, 255, 0.035)";
    ctx.beginPath();
    ctx.moveTo(0, h * 0.25);
    ctx.lineTo(w, h * 0.25);
    ctx.moveTo(0, h * 0.75);
    ctx.lineTo(w, h * 0.75);
    ctx.stroke();

    // 2. Real-time audio FFT bars
    if (player.playing) {
      audioEngine.getFrequencyData(freqData);
      const barCount = 48;
      const barWidth = w / barCount - 1.5;
      for (let i = 0; i < barCount; i++) {
        const val = freqData[i] || 0;
        const barH = (val / 255) * (h * 0.72);
        const x = i * (barWidth + 1.5);
        const y = h - barH;

        const grad = ctx.createLinearGradient(0, y, 0, h);
        grad.addColorStop(0, `${accentHex}88`);
        grad.addColorStop(1, `${accentGlowHex}18`);
        ctx.fillStyle = grad;
        ctx.fillRect(x, y, barWidth, barH);
      }
    }

    // 3. Smooth EQ frequency response curve
    if (audioEngine.enabled) {
      ctx.beginPath();
      const points: { x: number; y: number }[] = [];
      const numBands = audioEngine.gains.length;

      for (let i = 0; i < numBands; i++) {
        const x = (i / (numBands - 1)) * (w - 40) + 20;
        const gain = audioEngine.gains[i];
        // -12dB -> h, +12dB -> 0, 0dB -> h/2
        const y = zeroY - (gain / 12) * (h * 0.42);
        points.push({ x, y });
      }

      if (points.length > 0) {
        ctx.moveTo(points[0].x, points[0].y);
        for (let i = 0; i < points.length - 1; i++) {
          const xc = (points[i].x + points[i + 1].x) / 2;
          const yc = (points[i].y + points[i + 1].y) / 2;
          ctx.quadraticCurveTo(points[i].x, points[i].y, xc, yc);
        }
        ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);

        ctx.strokeStyle = accentHex;
        ctx.lineWidth = 2.5;
        ctx.shadowColor = accentHex;
        ctx.shadowBlur = 10;
        ctx.stroke();
        ctx.shadowBlur = 0;

        // Draw points
        for (const p of points) {
          ctx.fillStyle = isLight ? "#0f172a" : "#ffffff";
          ctx.beginPath();
          ctx.arc(p.x, p.y, 3.2, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    animId = requestAnimationFrame(drawVisualizer);
  }

  $effect(() => {
    if (open) {
      animId = requestAnimationFrame(drawVisualizer);
    } else {
      cancelAnimationFrame(animId);
    }
  });

  function close() {
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="eq-backdrop" onclick={close} role="presentation">
    <div
      class="eq-dialog"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => {
        e.stopPropagation();
        if (e.key === "Escape") close();
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Equalizer & Audio Processor"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="eq-header">
        <div class="header-left">
          <div class="eq-icon-badge">
            <SlidersVertical size={20} />
          </div>
          <div>
            <h2 class="eq-title">Bộ chỉnh âm thanh (Equalizer)</h2>
            <p class="eq-sub">10-băng tần · Chuẩn hóa âm lượng · Chống vỡ tiếng</p>
          </div>
        </div>

        <div class="header-actions">
          <!-- Bypass EQ Toggle -->
          <button
            class="toggle-btn"
            class:active={audioEngine.enabled}
            onclick={() => audioEngine.toggleEnabled()}
            title={audioEngine.enabled ? "Bấm để tắt EQ" : "Bấm để bật EQ"}
          >
            <span class="dot"></span>
            <span>{audioEngine.enabled ? "EQ ĐANG BẬT" : "EQ ĐÃ TẮT"}</span>
          </button>

          <!-- Close button -->
          <button class="close-btn" onclick={close} title="Đóng bảng EQ" aria-label="Đóng">
            <X size={18} />
          </button>
        </div>
      </div>

      <!-- Realtime Visualizer & Curve Display -->
      <div class="eq-screen">
        <canvas
          bind:this={canvasEl}
          width="620"
          height="120"
          class="eq-canvas"
        ></canvas>
        <div class="screen-tags">
          <span class="db-tag top">+12 dB</span>
          <span class="db-tag mid">0 dB</span>
          <span class="db-tag btm">-12 dB</span>
        </div>
      </div>

      <!-- Presets Selector Chips -->
      <div class="presets-section">
        <div class="presets-label">Cài đặt sẵn (Presets):</div>
        <div class="presets-scroll">
          {#each Object.entries(EQ_PRESETS) as [key, item]}
            <button
              class="preset-chip"
              class:active={audioEngine.preset === key}
              onclick={() => audioEngine.applyPreset(key)}
            >
              {item.name}
            </button>
          {/each}
          {#if audioEngine.preset === "custom"}
            <span class="preset-chip active custom-chip">Tùy biến</span>
          {/if}
        </div>
      </div>

      <!-- 10 Vertical Sliders + PreAmp Slider -->
      <div class="sliders-frame">
        <!-- PreAmp Fader -->
        <div class="slider-col preamp-col">
          <span class="gain-val">{audioEngine.preAmp > 0 ? `+${audioEngine.preAmp}` : audioEngine.preAmp}</span>
          <div class="fader-track">
            <input
              type="range"
              min="-12"
              max="12"
              step="0.5"
              value={audioEngine.preAmp}
              oninput={handlePreAmpInput}
              class="fader-input"
              aria-label="Pre-Amp"
            />
          </div>
          <span class="freq-label preamp-lbl">PRE-AMP</span>
        </div>

        <div class="v-divider"></div>

        <!-- 10 EQ Faders -->
        <div class="eq-faders-grid">
          {#each EQ_FREQUENCIES as freq, i}
            <div class="slider-col">
              <span class="gain-val">
                {audioEngine.gains[i] > 0 ? `+${audioEngine.gains[i]}` : audioEngine.gains[i]}
              </span>
              <div class="fader-track">
                <input
                  type="range"
                  min="-12"
                  max="12"
                  step="0.5"
                  value={audioEngine.gains[i]}
                  oninput={(e) => handleGainInput(i, e)}
                  class="fader-input"
                  aria-label="{fmtFreq(freq)} Hz"
                />
              </div>
              <span class="freq-label">{fmtFreq(freq)}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Footer: Normalize & Reset Buttons -->
      <div class="eq-footer">
        <div class="footer-left">
          <!-- Auto Normalization Switch -->
          <label class="switch-row" title="Giữ âm lượng đồng đều giữa các bài to nhỏ khác nhau và chống rè tiếng">
            <input
              type="checkbox"
              checked={audioEngine.normalize}
              onchange={() => audioEngine.toggleNormalize()}
            />
            <span class="switch-box"></span>
            <span class="switch-text">
              <strong>Chuẩn hóa âm lượng (Auto Normalize)</strong>
              <small>Cân bằng âm lượng, bảo vệ tai và chống vỡ tiếng</small>
            </span>
          </label>
        </div>

        <div class="footer-right">
          <button class="reset-btn" onclick={() => audioEngine.reset()}>
            <RotateCcw size={14} />
            Đặt lại (Reset 0 dB)
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .eq-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: oklch(0 0 0 / 0.62);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .eq-dialog {
    width: 100%;
    max-width: 680px;
    max-height: calc(100dvh - 32px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-strong);
    border-radius: calc(var(--ui-radius, 12px) + 4px);
    box-shadow: 0 24px 60px oklch(0 0 0 / 0.65);
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    animation: scaleUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .eq-dialog > * {
    flex-shrink: 0;
  }

  @keyframes scaleUp {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  /* Header */
  .eq-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 12px;
    padding: 1.15rem 1.4rem 0.85rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
  }

  .header-left {
    min-width: 0;
    flex: 1 1 240px;
    display: flex;
    align-items: center;
    gap: 0.85rem;
  }

  .eq-icon-badge {
    width: 38px;
    height: 38px;
    border-radius: var(--ui-radius, 10px);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .eq-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.015em;
  }

  .eq-sub {
    margin: 0.15rem 0 0;
    font-size: 0.76rem;
    color: var(--text-secondary);
  }

  .header-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.35rem 0.75rem;
    border-radius: 999px;
    font-size: 0.73rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .toggle-btn .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .toggle-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent);
  }

  .toggle-btn.active .dot {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
  }

  .close-btn {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.38rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-3);
    border-color: var(--border-strong);
  }

  /* Screen Canvas */
  .eq-screen {
    position: relative;
    margin: 1rem 1.4rem 0.5rem;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--ui-radius, 12px);
    height: 120px;
    overflow: hidden;
  }

  .eq-canvas {
    width: 100%;
    height: 100%;
    display: block;
  }

  .screen-tags {
    position: absolute;
    right: 8px;
    top: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 6px 0;
    pointer-events: none;
  }

  .db-tag {
    font-size: 0.65rem;
    font-weight: 700;
    color: var(--text-muted);
    font-family: var(--font-mono);
    text-align: right;
  }

  /* Presets */
  .presets-section {
    padding: 0.5rem 1.4rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .presets-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .presets-scroll {
    display: flex;
    gap: 0.4rem;
    overflow-x: auto;
    scrollbar-width: none;
    padding-bottom: 2px;
  }

  .presets-scroll::-webkit-scrollbar {
    display: none;
  }

  .preset-chip {
    padding: 0.3rem 0.7rem;
    font-size: 0.72rem;
    font-weight: 600;
    border-radius: 7px;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.12s ease;
  }

  .preset-chip:hover {
    background: var(--bg-surface-2);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .preset-chip.active {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-weight: 700;
    box-shadow: 0 2px 10px var(--accent-glow);
  }

  .custom-chip {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent);
  }

  /* Sliders Frame */
  .sliders-frame {
    display: flex;
    align-items: center;
    padding: 0.8rem 1.4rem 1rem;
    gap: 0.8rem;
    overflow-x: auto;
  }

  .v-divider {
    width: 1px;
    height: 150px;
    background: var(--border-subtle);
  }

  .eq-faders-grid {
    flex: 1;
    min-width: 350px;
    display: grid;
    grid-template-columns: repeat(10, minmax(0, 1fr));
    gap: 0.35rem;
  }

  .slider-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
  }

  .preamp-col {
    width: 48px;
    flex-shrink: 0;
  }

  .gain-val {
    font-size: 0.68rem;
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--accent);
    min-height: 1rem;
  }

  .fader-track {
    height: 130px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .fader-input {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 6px;
    height: 120px;
    cursor: pointer;
    appearance: none;
    background: var(--bg-surface-3);
    border-radius: 999px;
    outline: none;
  }

  .fader-input::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-primary);
    border: 2.5px solid var(--accent);
    box-shadow: 0 1px 6px oklch(0 0 0 / 0.4);
    cursor: pointer;
    transition: transform 0.1s;
  }

  .fader-input::-webkit-slider-thumb:hover {
    transform: scale(1.18);
  }

  .freq-label {
    font-size: 0.68rem;
    font-weight: 700;
    color: var(--text-muted);
  }

  .preamp-lbl {
    color: var(--accent);
  }

  /* Footer */
  .eq-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 14px;
    padding: 0.85rem 1.4rem 1.1rem;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
  }

  .switch-row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    cursor: pointer;
    user-select: none;
  }

  .switch-row input {
    position: absolute;
    width: 1px;
    height: 1px;
    clip-path: inset(50%);
  }
  .switch-row input:focus-visible + .switch-box {
    outline: 2px solid var(--accent);
    outline-offset: 3px;
  }

  .switch-box {
    flex-shrink: 0;
    width: 38px;
    height: 21px;
    border-radius: 999px;
    background: var(--bg-surface-3);
    border: 1px solid var(--border-subtle);
    position: relative;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .switch-box::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: var(--text-secondary);
    transition: transform 0.15s ease, background 0.15s ease;
  }

  .switch-row input:checked + .switch-box {
    background: var(--accent-soft);
    border-color: var(--accent);
  }

  .switch-row input:checked + .switch-box::after {
    transform: translateX(17px);
    background: var(--accent);
  }

  .switch-text {
    display: flex;
    flex-direction: column;
  }

  .switch-text strong {
    font-size: 0.8rem;
    color: var(--text-primary);
  }

  .switch-text small {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.85rem;
    border-radius: 8px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .reset-btn:hover {
    background: var(--bg-surface-3);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
</style>
