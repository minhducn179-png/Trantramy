import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { runInNewContext } from "node:vm";
import ts from "typescript";

const storage = new Map();
const properties = new Map();
let prefersLight = false;
let onSchemeChange;
const media = { get matches() { return prefersLight; }, addEventListener: (_, listener) => { onSchemeChange = listener; } };
const root = {
  dataset: {},
  style: {
    setProperty: (key, value) => properties.set(key, value),
    removeProperty: (key) => properties.delete(key),
  },
};
const cache = new Map();
function load(file) {
  if (cache.has(file)) return cache.get(file);
  const exports = {};
  cache.set(file, exports);
  const js = ts.transpileModule(readFileSync(file, "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  runInNewContext(js, {
    exports, $state: (value) => value,
    require: (name) => {
      if (name === "./palettes") return load(resolve("src/lib/settings/palettes.ts"));
      if (name === "@tauri-apps/api/core") return { invoke: async () => {} };
      if (name === "@tauri-apps/api/event") return { emit: async () => {} };
      throw new Error(`Unexpected import: ${name}`);
    },
    document: { documentElement: root },
    window: { matchMedia: () => media },
    localStorage: { getItem: (key) => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) },
    console, encodeURIComponent, setTimeout, clearTimeout,
  }, { filename: file });
  return exports;
}

const { THEME_FAMILIES, THEME_PALETTES, getPalette, contrastRatio } = load(resolve("src/lib/settings/palettes.ts"));
const { theme, normalizeConfig, encodeConfigCode, decodeConfigCode, hexToOklchApprox, oklchApproxToHex } = load(resolve("src/lib/settings/theme.svelte.ts"));
assert.equal(THEME_FAMILIES.length, 10);
assert.equal(THEME_PALETTES.length, 31);
assert.equal(new Set(THEME_PALETTES.map((p) => p.id)).size, THEME_PALETTES.length);
for (const f of THEME_FAMILIES) {
  assert.equal(getPalette(f.default)?.family, f.id);
  assert.match(f.source, /^https:\/\//);
}

storage.set("dlc.ui.config.v2", JSON.stringify({ presetId: "apple-liquid-glass", cornerRadius: 23, stealthMode: false, appleGlassMode: true }));
theme.init();
assert.equal(theme.activePalette.id, "catppuccin-mocha");
assert.equal(theme.config.cornerRadius, 23);
assert.equal(theme.config.stealthMode, false);
assert.equal(theme.config.appleGlassMode, true);
assert.equal(normalizeConfig({ presetId: "custom" }).paletteId, null);
assert.equal(normalizeConfig({ paletteId: "invalid" }).paletteId, null);

theme.update({ bgImage: "fixture://wallpaper", waveformStyle: "line", waveformBars: 96 });
for (const p of THEME_PALETTES) {
  assert.ok(THEME_FAMILIES.some((f) => f.id === p.family));
  for (const color of Object.values(p.colors)) {
    assert.match(color, /^#[0-9a-f]{6}$/);
    const { l, c, h } = hexToOklchApprox(color);
    assert.equal(oklchApproxToHex(l, c, h), color, `color round-trip ${color}`);
  }
  theme.applyPreset(p.id);
  assert.equal(theme.activePalette.id, p.id);
  assert.equal(root.dataset.scheme, p.scheme);
  assert.equal(root.dataset.palette, p.id);
  assert.equal(properties.get("--bg-overlay-tint"), p.colors.base);
  assert.equal(properties.get("--wave-idle"), p.colors.border);
  assert.equal(properties.get("--wave-idle-hover"), p.colors.muted);
  assert.equal(theme.config.cornerRadius, 23, "palette must preserve layout");
  assert.equal(theme.config.stealthMode, false, "palette must preserve privacy");
  assert.equal(theme.config.appleGlassMode, true, "palette must preserve glass");
  assert.equal(theme.config.bgImage, "fixture://wallpaper", "palette must preserve wallpaper");
  assert.equal(theme.config.waveformStyle, "line");
  assert.equal(theme.config.waveformBars, 96);
  const backgrounds = [p.colors.base, p.colors.surface, p.colors.overlay, p.colors.sidebar];
  for (const token of ["--text-primary", "--text-secondary", "--text-muted"]) {
    for (const bg of backgrounds) {
      assert.ok(contrastRatio(properties.get(token), bg) >= 4.5, `${p.id}: ${token} on ${bg}`);
    }
  }
  assert.ok(contrastRatio(properties.get("--accent"), properties.get("--accent-fg")) >= 4.5);
  const decoded = decodeConfigCode(encodeConfigCode(theme.config));
  assert.ok(decoded, p.id);
  assert.equal(decoded.paletteId, p.id, "style code must retain exact palette");
  theme.update({ cornerRadius: 19 });
  assert.equal(theme.activePalette.id, p.id, "non-color edits must not detach palette");
  theme.update({ cornerRadius: 23 });
}

theme.applyPreset("kanagawa-wave");
theme.setColorScheme("light");
assert.equal(theme.activePalette.id, "kanagawa-lotus");
theme.setColorScheme("dark");
assert.equal(theme.activePalette.id, "kanagawa-wave");
theme.setColorScheme("auto");
prefersLight = true;
onSchemeChange();
assert.equal(theme.activePalette.id, "kanagawa-lotus");
assert.equal(root.dataset.scheme, "light");
prefersLight = false;
onSchemeChange();
assert.equal(theme.activePalette.id, "kanagawa-wave");
assert.equal(root.dataset.scheme, "dark");

theme.applyPreset("vesper-dark");
theme.setColorScheme("light");
assert.equal(theme.config.paletteId, null, "unsupported scheme becomes custom");
assert.equal(root.dataset.scheme, "light");
theme.applyPreset("catppuccin-latte");
theme.update({ accentHue: 200 });
assert.equal(theme.config.paletteId, null, "manual color edit becomes custom");
theme.applyPreset("vesper-dark");
const profile = theme.saveProfile("Vesper check", "TEST-VESPER");
assert.equal(decodeConfigCode("TEST-VESPER", [profile]).paletteId, "vesper-dark");
assert.equal(JSON.parse(storage.get("dlc.ui.config.v2")).paletteId, "vesper-dark");
assert.equal(decodeConfigCode("DLC-invalid"), null);
assert.equal(decodeConfigCode("DLC-200.10.70-DP-MR68-CF-R1-PUNKNOWN"), null);
assert.equal(decodeConfigCode("DLC-238.17.70-DP-MR68-CF-R1").paletteId, null);
theme.resetDefault();
assert.equal(theme.activePalette.id, "catppuccin-mocha");
for (const p of THEME_PALETTES) {
  const decoded = decodeConfigCode(`DLC-${p.id.toUpperCase()}`);
  assert.equal(decoded.paletteId, p.id);
  assert.equal(encodeConfigCode(decoded), `DLC-${p.id.toUpperCase()}`);
}
console.log("PASS: 10 theme families, 31 variants, exact colors, contrast, persistence, codes, migration and appearance preservation");
