<script lang="ts">
  import { Check, Moon, Sun } from "@lucide/svelte";
  import { theme } from "./theme.svelte";
  import { THEME_FAMILIES, THEME_PALETTES, getPalette } from "./palettes";

  const active = $derived(theme.activePalette);
  const familyId = $derived(active?.family ?? "catppuccin");
  const family = $derived(THEME_FAMILIES.find((f) => f.id === familyId)!);
  const flavors = $derived(THEME_PALETTES.filter((p) => p.family === familyId));
</script>

<div class="theme-picker">
  <div class="picker-head">
    <div>
      <h3>Chọn bộ màu</h3>
      <p>{active ? `${family.name} · ${active.name}` : "Đang dùng màu tùy chỉnh"}</p>
    </div>
    <span class="picker-count">{THEME_FAMILIES.length} bộ · {THEME_PALETTES.length} biến thể</span>
  </div>

  <div class="family-grid" role="group" aria-label="Bộ theme">
    {#each THEME_FAMILIES as f (f.id)}
      {@const preview = getPalette(f.default)!}
      <button
        type="button"
        class="family-option"
        class:selected={active?.family === f.id}
        aria-label={f.name}
        aria-pressed={active?.family === f.id}
        onclick={() => theme.applyPreset(f.default)}
      >
        <span class="family-info"><strong>{f.name}</strong><small>{f.description}</small></span>
        <span class="family-swatch" aria-hidden="true">
          <span style:background={preview.colors.base}></span>
          <span style:background={preview.colors.surface}></span>
          <span style:background={preview.colors.accent}></span>
        </span>
      </button>
    {/each}
  </div>

  <div class="flavor-header"><strong>Biến thể {family.name}</strong><span>Áp dụng ngay khi chọn</span></div>
  <div class="flavor-list" role="group" aria-label="Biến thể {family.name}">
    {#each flavors as p (p.id)}
      <button
        type="button"
        class="flavor-option"
        class:selected={active?.id === p.id}
        aria-label="{family.name} {p.name}"
        aria-pressed={active?.id === p.id}
        onclick={() => theme.applyPreset(p.id)}
      >
        {#if p.scheme === "light"}<Sun size={14} />{:else}<Moon size={14} />{/if}
        <span>{p.name}</span>
        <small>{p.scheme === "light" ? "Sáng" : "Tối"}</small>
        {#if active?.id === p.id}<Check size={14} />{/if}
      </button>
    {/each}
  </div>

  {#if active}
    <div class="palette-detail" aria-live="polite">
      <span><i style:background={active.colors.base}></i>Nền <code>{active.colors.base.toUpperCase()}</code></span>
      <span><i style:background={active.colors.accent}></i>Nhấn <code>{active.colors.accent.toUpperCase()}</code></span>
      <span>Mã <code>DLC-{active.id.toUpperCase()}</code></span>
    </div>
    {#if active.note}<p class="palette-note">{active.note}</p>{/if}
  {/if}
  <p class="palette-note">Đổi bộ màu không đổi bố cục, hình nền hay hiệu ứng kính. Màu chữ và màu nhấn sáng được cân chỉnh khi cần đủ tương phản.</p>
</div>

<style>
  .theme-picker { display: flex; flex-direction: column; gap: 14px; min-width: 0; }
  .picker-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; }
  h3 { margin: 0; font-size: 16px; }
  p { margin: 4px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.5; }
  .picker-count { color: var(--text-muted); font-size: 12px; }
  .family-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(220px, 100%), 1fr)); gap: 8px; }
  button { font: inherit; cursor: pointer; color: var(--text-primary); border: 1px solid var(--border-subtle); background: var(--bg-canvas); transition: border-color 0.15s ease, background-color 0.15s ease; }
  button:hover { border-color: var(--border-strong); background: var(--bg-surface-2); }
  button:focus-visible { outline: 2px solid var(--text-primary); outline-offset: 3px; }
  button.selected { border-color: var(--accent); background: var(--accent-soft); }
  .family-option { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 12px; border-radius: var(--ui-radius-sm, 10px); min-width: 0; text-align: left; }
  .family-info { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
  .family-info strong { font-size: 13px; }
  .family-info small { color: var(--text-secondary); font-size: 11.5px; line-height: 1.4; }
  .family-swatch { display: flex; flex: none; border: 1px solid var(--border-subtle); border-radius: 6px; overflow: hidden; }
  .family-swatch span { width: 12px; height: 28px; }
  .flavor-header { display: flex; justify-content: space-between; gap: 10px; flex-wrap: wrap; padding-top: 4px; font-size: 12px; }
  .flavor-header span { color: var(--text-muted); }
  .flavor-list { display: flex; flex-wrap: wrap; gap: 8px; }
  .flavor-option { display: inline-flex; align-items: center; gap: 7px; padding: 9px 12px; min-height: 36px; border-radius: var(--ui-radius-sm, 10px); font-size: 12px; }
  .flavor-option small { color: var(--text-secondary); font-size: 11px; }
  .palette-detail { display: flex; flex-wrap: wrap; gap: 10px 18px; color: var(--text-secondary); font-size: 11.5px; line-height: 1.6; }
  .palette-detail > span { display: inline-flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .palette-detail i { display: inline-block; width: 12px; height: 12px; border: 1px solid var(--border-strong); border-radius: 3px; }
  code { font: inherit; font-variant-numeric: tabular-nums; overflow-wrap: anywhere; }
  .palette-note { margin: 0; font-size: 11.5px; color: var(--text-muted); }
  @media (prefers-reduced-motion: reduce) { button { transition: none; } }
</style>
