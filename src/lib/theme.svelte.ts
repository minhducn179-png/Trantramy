import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { THEME_PALETTES, THEME_FAMILIES, DEFAULT_PALETTE_ID, getPalette, contrastRatio, readableColor, type ThemePalette } from "./palettes";

export type CanvasMode = "deep" | "oled" | "graphite" | "midnight" | "warm-noir" | "custom";
export type WaveformStyle =
  | "mirrored"
  | "line"
  | "single"
  | "center-sym"
  | "bars-thick"
  | "led-matrix"
  | "neon-curve"
  | "dual-neon"
  | "dots-pulse"
  | "blocks-retro"
  | "flame-peak"
  | "sc-fine"
  | "minimal";
export type ColorScheme = "dark" | "light" | "auto";
export type UiDensity = "compact" | "comfortable" | "spacious";
export type BgGradient = "flat" | "vertical" | "diagonal" | "radial";
export type SidebarShade = "darker" | "same" | "lighter";
export type RadiusStyle = "sharp" | "rounded" | "soft";
export type WinControlsMode = "auto-hide" | "always";
export type BgImageFit = "cover" | "contain" | "fill";
export type BgImagePosition = "center" | "top" | "bottom" | "left" | "right";
export type IslandWaveMode = "bounce-progress" | "progress-static" | "bounce-real" | "scroll-wave";
export type WaveformBounceMode = "none" | "playhead" | "real-audio";

export interface SavedBgWallpaper {
  id: string;
  name: string;
  src: string;
  addedAt: string;
}

export interface BuiltinWallpaper {
  id: string;
  name: string;
  tag: string;
  src: string;
  accentHue: number;
  bgHue: number;
}

export interface UiExtras {
  paletteId: string | null;
  colorScheme: ColorScheme;
  bgGradient: BgGradient;
  bgHue2: number; // 0..360 gradient end color
  bgGradientStrength: number; // 0..1
  surfaceContrast: number; // 0.4..2.2 (card / panel separation)
  sidebarShade: SidebarShade;
  textLightness: number; // 0.78..1
  borderStrength: number; // 0.3..2
  bgImage: string | null; // custom background image path or URL
  bgImageDim: number; // 0.08..0.92 (overlay dimness over wallpaper)
  bgImageBlur: number; // 0..28 (wallpaper blur in px)
  bgImageBrightness: number; // 0.35..1.45 (wallpaper brightness)
  bgImageSaturation: number; // 0.3..1.85 (wallpaper saturation)
  bgImageSurfaceAlpha: number; // 0.14..0.95 (glass opacity of UI panels/cards)
  bgImageFit: BgImageFit;
  bgImagePosition: BgImagePosition;
  bgImageAutoTheme: boolean; // auto-sync accent & background tint when picking an image
  cornerRadius: number; // 0..28 px frame corner roundness
  stealthMode: boolean; // hide window from Overwolf, Outplayed, and game capture overlays
  autoIslandOnMinimize: boolean; // automatically show Dynamic Island when minimizing/hiding main window
  islandWaveMode: IslandWaveMode; // Dynamic Island wave animation & progress mode
  waveformBounce: WaveformBounceMode; // Main WaveformBar bounce behavior ("none" | "playhead" | "real-audio")
  islandBg: string; // Dynamic Island background color (hex)
  islandAccent: string; // Dynamic Island accent color (hex); "" = follow app accent
  islandText: string; // Dynamic Island text color (hex)
  islandOpacity: number; // 0.4..1
  islandRadius: number; // 8..44 px corner roundness (pill is capped at its half-height)
  islandArtShape: "square" | "circle"; // cover art shape
  islandThemeSync: boolean; // true = island colors follow app theme automatically
  waterDropletMode: boolean; // true = water droplet 3D glass surfaces, specular gloss
  waterRippleEffect: boolean; // true = interactive water ripple click effect
  appleGlassMode: boolean; // true = Apple Liquid Glass material system, squircle geometry & floating deck
}

export const EXTRA_DEFAULTS: UiExtras = {
  paletteId: null,
  colorScheme: "dark",
  bgGradient: "flat",
  bgHue2: 280,
  bgGradientStrength: 0.5,
  surfaceContrast: 1,
  sidebarShade: "same",
  textLightness: 0.965,
  borderStrength: 1,
  bgImage: null,
  bgImageDim: 0.48,
  bgImageBlur: 0,
  bgImageBrightness: 1,
  bgImageSaturation: 1.08,
  bgImageSurfaceAlpha: 0.54,
  bgImageFit: "cover",
  bgImagePosition: "center",
  bgImageAutoTheme: true,
  cornerRadius: 16,
  stealthMode: true,
  autoIslandOnMinimize: true,
  islandWaveMode: "bounce-progress",
  waveformBounce: "none",
  islandBg: "#000000",
  islandAccent: "",
  islandText: "#ffffff",
  islandOpacity: 1,
  islandRadius: 38,
  islandArtShape: "square",
  islandThemeSync: true,
  waterDropletMode: false,
  waterRippleEffect: false,
  appleGlassMode: false,
};

export type PresetConfig = Omit<UiConfig, "presetId" | keyof UiExtras> & Partial<UiExtras>;

export interface UiConfig extends UiExtras {
  presetId: string;
  accentHue: number; // 0..360
  accentChroma: number; // 0.05..0.26
  accentLightness: number; // 0.50..0.85
  canvasMode: CanvasMode;
  bgHue: number; // 0..360 (used when canvasMode === "custom" or tinted)
  bgChroma: number; // 0..0.14
  bgLightness: number; // 0.02..0.35
  waveformStyle: WaveformStyle;
  waveformBars: number; // 44 | 68 | 96 | 120
  density: UiDensity;
  radiusStyle: RadiusStyle;
  sidebarWidth: number; // 190..280
  ambientGlow: boolean;
  winControlsMode: WinControlsMode;
  fontScale: number; // 0.92 | 1 | 1.08
}

export interface ThemePreset {
  id: string;
  code: string;
  name: string;
  tagline: string;
  badge: string;
  previewAccent: string;
  previewBg: string;
  config: PresetConfig;
}

export interface SavedUserProfile {
  id: string;
  name: string;
  code: string;
  autoCode: string;
  createdAt: string;
  config: UiConfig;
}

export function hexToOklchApprox(hex: string): { l: number; c: number; h: number } {
  if (!/^#[a-f\d]{6}$/i.test(hex)) return { l: 0.1, c: 0.02, h: 240 };
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((v) => v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  const ll = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const l = 0.2104542553 * ll + 0.793617785 * m - 0.0040720468 * s;
  const a = 1.9779984951 * ll - 2.428592205 * m + 0.4505937099 * s;
  const bb = 0.0259040371 * ll + 0.7827717662 * m - 0.808675766 * s;
  const c = Math.hypot(a, bb);
  return { l, c, h: c < 0.000001 ? 0 : (Math.atan2(bb, a) * 180 / Math.PI + 360) % 360 };
}

export function oklchApproxToHex(l: number, c: number, h: number): string {
  const a = c * Math.cos(h * Math.PI / 180), b = c * Math.sin(h * Math.PI / 180);
  const ll = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return "#" + [
    4.0767416621 * ll - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * ll + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * ll - 0.7034186147 * m + 1.707614701 * s,
  ].map((v) => {
    const rgb = v <= 0.0031308 ? 12.92 * v : 1.055 * Math.max(0, v) ** (1 / 2.4) - 0.055;
    return Math.max(0, Math.min(255, Math.round(rgb * 255))).toString(16).padStart(2, "0");
  }).join("");
}

function paletteConfig(palette: ThemePalette) {
  const accent = hexToOklchApprox(palette.colors.accent);
  const bg = hexToOklchApprox(palette.colors.base);
  return {
    paletteId: palette.id,
    colorScheme: palette.scheme,
    accentHue: accent.h, accentChroma: accent.c, accentLightness: accent.l,
    canvasMode: "custom" as CanvasMode,
    bgHue: bg.h, bgChroma: bg.c, bgLightness: bg.l,
    bgGradient: "flat" as BgGradient,
    surfaceContrast: 1, sidebarShade: "same" as SidebarShade,
    textLightness: EXTRA_DEFAULTS.textLightness, borderStrength: 1,
  };
}

export const BUILTIN_PRESETS: ThemePreset[] = THEME_PALETTES.map((palette) => {
  const family = THEME_FAMILIES.find((f) => f.id === palette.family)!;
  return {
    id: palette.id,
    code: `DLC-${palette.id.toUpperCase()}`,
    name: `${family.name} ${palette.name}`,
    tagline: palette.note ?? family.description,
    badge: palette.scheme === "light" ? "SÁNG" : "TỐI",
    previewAccent: palette.colors.accent,
    previewBg: palette.colors.base,
    config: {
      ...paletteConfig(palette),
      waveformStyle: "mirrored", waveformBars: 68,
      density: "comfortable", radiusStyle: "rounded", sidebarWidth: 236,
      ambientGlow: true, winControlsMode: "auto-hide", fontScale: 1,
    },
  };
});

const STORAGE_KEY_CONFIG = "dlc.ui.config.v2";
const STORAGE_KEY_PROFILES = "dlc.ui.profiles.v2";
const STORAGE_KEY_WALLPAPERS = "dlc.ui.wallpapers.v1";

function svgDataUrl(svg: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

export const BUILTIN_WALLPAPERS: BuiltinWallpaper[] = [
  {
    id: "wp-apple-liquid",
    name: "Kính Lỏng Apple (Liquid Spectrum)",
    tag: "APPLE GLASS",
    accentHue: 342,
    bgHue: 280,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <linearGradient id="ap-bg" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#05050d"/>
          <stop offset="50%" stop-color="#0b0a1a"/>
          <stop offset="100%" stop-color="#14081e"/>
        </linearGradient>
        <radialGradient id="ap-glow1" cx="30%" cy="20%" r="65%">
          <stop offset="0%" stop-color="#ff2d55" stop-opacity="0.38"/>
          <stop offset="45%" stop-color="#af52de" stop-opacity="0.18"/>
          <stop offset="100%" stop-color="#05050d" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="ap-glow2" cx="78%" cy="70%" r="60%">
          <stop offset="0%" stop-color="#0071e3" stop-opacity="0.35"/>
          <stop offset="50%" stop-color="#5856d6" stop-opacity="0.15"/>
          <stop offset="100%" stop-color="#05050d" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="ap-ribbon1" x1="0%" y1="0%" x2="100%" y2="80%">
          <stop offset="0%" stop-color="#ff375f" stop-opacity="0.65"/>
          <stop offset="35%" stop-color="#bf5af2" stop-opacity="0.55"/>
          <stop offset="70%" stop-color="#5e5ce6" stop-opacity="0.5"/>
          <stop offset="100%" stop-color="#0a84ff" stop-opacity="0.6"/>
        </linearGradient>
        <filter id="ap-blur" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="60"/>
        </filter>
      </defs>
      <rect width="1600" height="900" fill="url(#ap-bg)"/>
      <rect width="1600" height="900" fill="url(#ap-glow1)"/>
      <rect width="1600" height="900" fill="url(#ap-glow2)"/>
      <g filter="url(#ap-blur)">
        <path d="M-100 620 C 320 780, 580 340, 960 480 C 1320 620, 1420 220, 1720 320" fill="none" stroke="url(#ap-ribbon1)" stroke-width="120" stroke-linecap="round"/>
        <path d="M-50 320 C 380 180, 720 540, 1180 340 C 1450 210, 1580 420, 1700 520" fill="none" stroke="#64d2ff" stroke-opacity="0.45" stroke-width="80" stroke-linecap="round"/>
      </g>
      <path d="M-80 620 C 340 770, 590 350, 970 480 C 1310 610, 1410 230, 1700 320" fill="none" stroke="#ffffff" stroke-opacity="0.5" stroke-width="3"/>
      <path d="M-40 330 C 390 190, 710 535, 1170 345 C 1440 220, 1570 415, 1680 515" fill="none" stroke="#ffffff" stroke-opacity="0.4" stroke-width="2.5"/>
    </svg>`),
  },
  {
    id: "wp-aqua-droplets",
    name: "Giọt Sương Đại Dương",
    tag: "AQUA DROPS",
    accentHue: 196,
    bgHue: 212,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <radialGradient id="oceanGlow" cx="50%" cy="30%" r="70%">
          <stop offset="0%" stop-color="#0ff" stop-opacity="0.35"/>
          <stop offset="35%" stop-color="#006699" stop-opacity="0.45"/>
          <stop offset="70%" stop-color="#041829" stop-opacity="0.95"/>
          <stop offset="100%" stop-color="#020a12"/>
        </radialGradient>
        <radialGradient id="dropLight" cx="35%" cy="30%" r="65%">
          <stop offset="0%" stop-color="#ffffff" stop-opacity="0.9"/>
          <stop offset="40%" stop-color="#80f0ff" stop-opacity="0.5"/>
          <stop offset="75%" stop-color="#005577" stop-opacity="0.3"/>
          <stop offset="100%" stop-color="#002233" stop-opacity="0.6"/>
        </radialGradient>
        <radialGradient id="dropSpec" cx="30%" cy="25%" r="35%">
          <stop offset="0%" stop-color="#ffffff" stop-opacity="0.95"/>
          <stop offset="100%" stop-color="#ffffff" stop-opacity="0"/>
        </radialGradient>
        <filter id="liquidGlow" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="6" result="blur"/>
          <feComposite in="SourceGraphic" in2="blur" operator="over"/>
        </filter>
      </defs>
      <rect width="1600" height="900" fill="#030d17"/>
      <rect width="1600" height="900" fill="url(#oceanGlow)"/>
      <path d="M-100 280 Q 300 160 800 240 T 1700 190 L 1700 480 Q 1200 380 700 460 T -100 410 Z" fill="#00ffff" fill-opacity="0.08" filter="url(#liquidGlow)"/>
      <path d="M-100 480 Q 400 340 900 440 T 1700 370 L 1700 700 Q 1100 560 600 670 T -100 610 Z" fill="#0088cc" fill-opacity="0.06" filter="url(#liquidGlow)"/>
      <g filter="url(#liquidGlow)">
        <ellipse cx="480" cy="320" rx="42" ry="38" fill="url(#dropLight)"/>
        <ellipse cx="468" cy="308" rx="14" ry="10" fill="url(#dropSpec)"/>
        <ellipse cx="496" cy="342" rx="10" ry="6" fill="#ffffff" fill-opacity="0.35"/>
        <ellipse cx="1140" cy="260" rx="55" ry="48" fill="url(#dropLight)"/>
        <ellipse cx="1124" cy="245" rx="18" ry="12" fill="url(#dropSpec)"/>
        <ellipse cx="1160" cy="288" rx="14" ry="8" fill="#ffffff" fill-opacity="0.4"/>
        <ellipse cx="820" cy="540" rx="34" ry="30" fill="url(#dropLight)"/>
        <ellipse cx="810" cy="530" rx="11" ry="8" fill="url(#dropSpec)"/>
        <ellipse cx="260" cy="610" rx="48" ry="42" fill="url(#dropLight)"/>
        <ellipse cx="245" cy="596" rx="16" ry="11" fill="url(#dropSpec)"/>
        <ellipse cx="1380" cy="620" rx="38" ry="34" fill="url(#dropLight)"/>
        <ellipse cx="1368" cy="609" rx="12" ry="9" fill="url(#dropSpec)"/>
        <ellipse cx="640" cy="180" rx="20" ry="18" fill="url(#dropLight)"/>
        <ellipse cx="634" cy="174" rx="7" ry="5" fill="url(#dropSpec)"/>
        <ellipse cx="980" cy="410" rx="24" ry="22" fill="url(#dropLight)"/>
        <ellipse cx="972" cy="403" rx="8" ry="6" fill="url(#dropSpec)"/>
        <ellipse cx="360" cy="450" rx="18" ry="16" fill="url(#dropLight)"/>
        <ellipse cx="355" cy="445" rx="6" ry="4" fill="url(#dropSpec)"/>
        <ellipse cx="1220" cy="460" rx="26" ry="24" fill="url(#dropLight)"/>
        <ellipse cx="1212" cy="452" rx="9" ry="7" fill="url(#dropSpec)"/>
        <ellipse cx="720" cy="710" rx="22" ry="20" fill="url(#dropLight)"/>
        <ellipse cx="713" cy="704" rx="7" ry="5" fill="url(#dropSpec)"/>
        <circle cx="530" cy="240" r="8" fill="url(#dropLight)"/>
        <circle cx="528" cy="237" r="3" fill="#fff"/>
        <circle cx="780" cy="310" r="7" fill="url(#dropLight)"/>
        <circle cx="1060" cy="340" r="9" fill="url(#dropLight)"/>
        <circle cx="1280" cy="210" r="8" fill="url(#dropLight)"/>
        <circle cx="180" cy="390" r="9" fill="url(#dropLight)"/>
        <circle cx="920" cy="620" r="11" fill="url(#dropLight)"/>
        <circle cx="1040" cy="730" r="10" fill="url(#dropLight)"/>
        <circle cx="430" cy="720" r="8" fill="url(#dropLight)"/>
      </g>
    </svg>`),
  },
  {
    id: "wp-cyber-city",
    name: "Cyberpunk Neon Night",
    tag: "SYNTHWAVE",
    accentHue: 326,
    bgHue: 282,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <linearGradient id="sky" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#070518"/>
          <stop offset="45%" stop-color="#1d0b3d"/>
          <stop offset="80%" stop-color="#5c1366"/>
          <stop offset="100%" stop-color="#ff2a85"/>
        </linearGradient>
        <radialGradient id="sun" cx="50%" cy="58%" r="28%">
          <stop offset="0%" stop-color="#fff07c"/>
          <stop offset="45%" stop-color="#ff5e7e"/>
          <stop offset="80%" stop-color="#b817a6" stop-opacity="0.6"/>
          <stop offset="100%" stop-color="#1d0b3d" stop-opacity="0"/>
        </radialGradient>
        <linearGradient id="floor" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#19082c"/>
          <stop offset="100%" stop-color="#05030d"/>
        </linearGradient>
      </defs>
      <rect width="1600" height="900" fill="url(#sky)"/>
      <circle cx="800" cy="520" r="240" fill="url(#sun)"/>
      <g fill="#0b051b">
        <rect x="80" y="340" width="95" height="260"/>
        <rect x="190" y="290" width="110" height="310"/>
        <rect x="315" y="370" width="85" height="230"/>
        <rect x="415" y="260" width="130" height="340"/>
        <rect x="560" y="330" width="90" height="270"/>
        <rect x="950" y="320" width="105" height="280"/>
        <rect x="1070" y="250" width="125" height="350"/>
        <rect x="1210" y="350" width="95" height="250"/>
        <rect x="1320" y="280" width="115" height="320"/>
        <rect x="1450" y="360" width="90" height="240"/>
      </g>
      <rect y="580" width="1600" height="320" fill="url(#floor)"/>
      <g stroke="#00f0ff" stroke-opacity="0.38" stroke-width="2">
        <line x1="0" y1="580" x2="1600" y2="580" stroke="#ff2a85" stroke-opacity="0.85" stroke-width="3"/>
        <line x1="0" y1="615" x2="1600" y2="615"/>
        <line x1="0" y1="660" x2="1600" y2="660"/>
        <line x1="0" y1="720" x2="1600" y2="720"/>
        <line x1="0" y1="800" x2="1600" y2="800"/>
        <line x1="800" y1="580" x2="-200" y2="900"/>
        <line x1="800" y1="580" x2="200" y2="900"/>
        <line x1="800" y1="580" x2="500" y2="900"/>
        <line x1="800" y1="580" x2="800" y2="900"/>
        <line x1="800" y1="580" x2="1100" y2="900"/>
        <line x1="800" y1="580" x2="1400" y2="900"/>
        <line x1="800" y1="580" x2="1800" y2="900"/>
      </g>
    </svg>`),
  },
  {
    id: "wp-aurora-borealis",
    name: "Cực Quang Bắc Cực",
    tag: "AURORA",
    accentHue: 170,
    bgHue: 225,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#030b17"/>
          <stop offset="50%" stop-color="#071d2c"/>
          <stop offset="100%" stop-color="#0b1026"/>
        </linearGradient>
        <radialGradient id="aur1" cx="30%" cy="25%" r="55%">
          <stop offset="0%" stop-color="#00ffa3" stop-opacity="0.55"/>
          <stop offset="45%" stop-color="#00b8ff" stop-opacity="0.28"/>
          <stop offset="100%" stop-color="#030b17" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="aur2" cx="75%" cy="35%" r="50%">
          <stop offset="0%" stop-color="#7b2cbf" stop-opacity="0.5"/>
          <stop offset="50%" stop-color="#00f5d4" stop-opacity="0.22"/>
          <stop offset="100%" stop-color="#030b17" stop-opacity="0"/>
        </radialGradient>
      </defs>
      <rect width="1600" height="900" fill="url(#bg)"/>
      <rect width="1600" height="900" fill="url(#aur1)"/>
      <rect width="1600" height="900" fill="url(#aur2)"/>
      <path d="M0 320 Q 420 140 820 290 T 1600 210 L 1600 480 Q 1180 330 780 450 T 0 430 Z" fill="#00f5d4" fill-opacity="0.16"/>
      <path d="M0 680 L 290 470 L 560 630 L 890 420 L 1220 620 L 1430 490 L 1600 590 L 1600 900 L 0 900 Z" fill="#061220"/>
      <path d="M0 740 L 420 540 L 760 700 L 1120 510 L 1600 710 L 1600 900 L 0 900 Z" fill="#030912"/>
    </svg>`),
  },
  {
    id: "wp-lofi-nebula",
    name: "Tinh Vân Vũ Trụ Lo-Fi",
    tag: "COSMOS",
    accentHue: 252,
    bgHue: 255,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <linearGradient id="sp" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#070716"/>
          <stop offset="50%" stop-color="#141136"/>
          <stop offset="100%" stop-color="#241038"/>
        </linearGradient>
        <radialGradient id="neb1" cx="72%" cy="28%" r="42%">
          <stop offset="0%" stop-color="#6c5ce7" stop-opacity="0.62"/>
          <stop offset="55%" stop-color="#a29bfe" stop-opacity="0.22"/>
          <stop offset="100%" stop-color="#070716" stop-opacity="0"/>
        </radialGradient>
        <radialGradient id="neb2" cx="24%" cy="68%" r="48%">
          <stop offset="0%" stop-color="#00cec9" stop-opacity="0.42"/>
          <stop offset="60%" stop-color="#0984e3" stop-opacity="0.18"/>
          <stop offset="100%" stop-color="#070716" stop-opacity="0"/>
        </radialGradient>
      </defs>
      <rect width="1600" height="900" fill="url(#sp)"/>
      <rect width="1600" height="900" fill="url(#neb1)"/>
      <rect width="1600" height="900" fill="url(#neb2)"/>
      <circle cx="1160" cy="260" r="110" fill="#1e1b4b" stroke="#a29bfe" stroke-opacity="0.45" stroke-width="3"/>
      <ellipse cx="1160" cy="260" rx="210" ry="38" fill="none" stroke="#74b9ff" stroke-opacity="0.45" stroke-width="5" transform="rotate(-18 1160 260)"/>
      <g fill="#ffffff" fill-opacity="0.75">
        <circle cx="210" cy="140" r="2.2"/><circle cx="480" cy="95" r="1.8"/><circle cx="750" cy="180" r="2.5"/>
        <circle cx="940" cy="110" r="1.5"/><circle cx="1410" cy="160" r="2"/><circle cx="320" cy="390" r="1.8"/>
        <circle cx="620" cy="440" r="2.2"/><circle cx="1320" cy="520" r="2.4"/><circle cx="180" cy="680" r="1.6"/>
      </g>
    </svg>`),
  },
  {
    id: "wp-sakura-twilight",
    name: "Hoàng Hôn Sakura",
    tag: "ANIME DUSK",
    accentHue: 348,
    bgHue: 320,
    src: svgDataUrl(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 900" width="1600" height="900">
      <defs>
        <linearGradient id="dusk" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stop-color="#120924"/>
          <stop offset="48%" stop-color="#38164c"/>
          <stop offset="78%" stop-color="#8b2665"/>
          <stop offset="100%" stop-color="#f26a8d"/>
        </linearGradient>
        <radialGradient id="moon" cx="68%" cy="42%" r="22%">
          <stop offset="0%" stop-color="#ffe5ec" stop-opacity="0.95"/>
          <stop offset="45%" stop-color="#ff8fab" stop-opacity="0.45"/>
          <stop offset="100%" stop-color="#38164c" stop-opacity="0"/>
        </radialGradient>
      </defs>
      <rect width="1600" height="900" fill="url(#dusk)"/>
      <circle cx="1088" cy="378" r="190" fill="url(#moon)"/>
      <path d="M0 640 Q 360 530 740 620 T 1600 570 L 1600 900 L 0 900 Z" fill="#230c33"/>
      <path d="M0 730 Q 480 650 940 720 T 1600 690 L 1600 900 L 0 900 Z" fill="#12051c"/>
      <g fill="#ffb3c6" fill-opacity="0.65">
        <ellipse cx="280" cy="220" rx="9" ry="5" transform="rotate(25 280 220)"/>
        <ellipse cx="460" cy="310" rx="7" ry="4" transform="rotate(-15 460 310)"/>
        <ellipse cx="690" cy="190" rx="10" ry="5" transform="rotate(40 690 190)"/>
        <ellipse cx="860" cy="290" rx="8" ry="4.5" transform="rotate(12 860 290)"/>
        <ellipse cx="1290" cy="240" rx="9" ry="4.5" transform="rotate(-28 1290 240)"/>
      </g>
    </svg>`),
  },
];

export async function extractPaletteFromImage(
  imgSrc: string,
): Promise<{ accentHue: number; accentChroma: number; accentLightness: number; bgHue: number; bgHue2: number } | null> {
  if (typeof window === "undefined" || typeof document === "undefined") return null;
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    const timer = setTimeout(() => resolve(null), 4000);
    img.onload = () => {
      clearTimeout(timer);
      try {
        const canvas = document.createElement("canvas");
        const size = 48;
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");
        if (!ctx) {
          resolve(null);
          return;
        }
        ctx.drawImage(img, 0, 0, size, size);
        const data = ctx.getImageData(0, 0, size, size).data;
        // Bucket hues into 24 bins (15 deg each) weighted by saturation & mid-luminance
        const bins = new Float64Array(24);
        const hueSum = new Float64Array(24);
        for (let i = 0; i < data.length; i += 4) {
          const a = data[i + 3];
          if (a < 128) continue;
          const r = data[i] / 255;
          const g = data[i + 1] / 255;
          const b = data[i + 2] / 255;
          const max = Math.max(r, g, b);
          const min = Math.min(r, g, b);
          const d = max - min;
          if (d < 0.08) continue; // skip near-grayscale pixels
          let h = 0;
          if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) * 60;
          else if (max === g) h = ((b - r) / d + 2) * 60;
          else h = ((r - g) / d + 4) * 60;
          const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
          const weight = d * (1 - Math.abs(lum - 0.52));
          const bin = Math.floor(h / 15) % 24;
          bins[bin] += weight;
          hueSum[bin] += h * weight;
        }
        let bestBin = 0;
        let secondBin = 0;
        for (let b = 0; b < 24; b++) {
          if (bins[b] > bins[bestBin]) {
            secondBin = bestBin;
            bestBin = b;
          } else if (b !== bestBin && bins[b] > bins[secondBin]) {
            secondBin = b;
          }
        }
        if (bins[bestBin] <= 0.01) {
          resolve(null);
          return;
        }
        const primaryHue = Math.round(hueSum[bestBin] / bins[bestBin]) % 360;
        const secondaryHue =
          bins[secondBin] > 0.01
            ? Math.round(hueSum[secondBin] / bins[secondBin]) % 360
            : (primaryHue + 38) % 360;
        resolve({
          accentHue: primaryHue,
          accentChroma: 0.19,
          accentLightness: 0.72,
          bgHue: primaryHue,
          bgHue2: secondaryHue,
        });
      } catch {
        resolve(null);
      }
    };
    img.onerror = () => {
      clearTimeout(timer);
      resolve(null);
    };
    img.src = imgSrc;
  });
}

const DEFAULT_CONFIG: UiConfig = {
  ...EXTRA_DEFAULTS,
  ...BUILTIN_PRESETS.find((p) => p.id === DEFAULT_PALETTE_ID)!.config,
  presetId: DEFAULT_PALETTE_ID,
};

const LEGACY_PRESET_IDS = new Set([
  "apple-liquid-glass", "aqua-droplet", "sc-ocean", "sc-orange", "spotify-emerald",
  "oled-zen", "cyber-neon", "hifi-gold", "crimson-wave",
]);

export function normalizeConfig(c: Partial<UiConfig>): UiConfig {
  const merged = { ...DEFAULT_CONFIG, ...c };
  if (c.paletteId === undefined) {
    merged.paletteId = null;
    if (LEGACY_PRESET_IDS.has(c.presetId ?? "")) {
      Object.assign(merged, paletteConfig(getPalette(DEFAULT_PALETTE_ID)!));
      merged.presetId = DEFAULT_PALETTE_ID;
    }
  } else if (!getPalette(c.paletteId)) {
    merged.paletteId = null;
  }
  if (typeof c.cornerRadius !== "number" || Number.isNaN(c.cornerRadius)) {
    merged.cornerRadius =
      merged.radiusStyle === "sharp" ? 4 : merged.radiusStyle === "soft" ? 24 : 16;
  } else {
    merged.cornerRadius = Math.max(0, Math.min(28, Math.round(c.cornerRadius)));
  }
  return merged;
}

function presetToConfig(p: ThemePreset): UiConfig {
  const radiusFromStyle =
    p.config.radiusStyle === "sharp" ? 4 : p.config.radiusStyle === "soft" ? 24 : 16;
  return { presetId: p.id, ...EXTRA_DEFAULTS, cornerRadius: radiusFromStyle, ...p.config };
}

const GRAD_CODES: BgGradient[] = ["flat", "vertical", "diagonal", "radial"];
const SHADE_CODES: SidebarShade[] = ["darker", "same", "lighter"];
const SCHEME_CODES: ColorScheme[] = ["dark", "light", "auto"];

function extrasSuffix(cfg: UiConfig): string {
  const x = normalizeConfig(cfg);
  const d = EXTRA_DEFAULTS;
  const expectedRadius =
    x.radiusStyle === "sharp" ? 4 : x.radiusStyle === "soft" ? 24 : 16;
  const isDefault =
    x.colorScheme === d.colorScheme &&
    x.bgGradient === d.bgGradient &&
    Math.round(x.bgHue2) === Math.round(d.bgHue2) &&
    Math.round(x.bgGradientStrength * 100) === Math.round(d.bgGradientStrength * 100) &&
    Math.round(x.surfaceContrast * 100) === Math.round(d.surfaceContrast * 100) &&
    x.sidebarShade === d.sidebarShade &&
    Math.round(x.textLightness * 1000) === Math.round(d.textLightness * 1000) &&
    Math.round(x.borderStrength * 100) === Math.round(d.borderStrength * 100) &&
    Math.round(x.cornerRadius) === expectedRadius;
  if (isDefault) return "";
  return `-X${GRAD_CODES.indexOf(x.bgGradient)}.${Math.round(x.bgHue2)}.${Math.round(x.bgGradientStrength * 100)}.${Math.round(x.surfaceContrast * 100)}.${SHADE_CODES.indexOf(x.sidebarShade)}.${Math.round(x.textLightness * 1000)}.${Math.round(x.borderStrength * 100)}.${SCHEME_CODES.indexOf(x.colorScheme)}.${Math.round(x.cornerRadius)}`;
}

const CANVAS_MAP: Record<CanvasMode, string> = {
  deep: "DP",
  oled: "OL",
  graphite: "GR",
  midnight: "MN",
  "warm-noir": "WN",
  custom: "CU",
};

const WF_MAP: Record<WaveformStyle, string> = {
  mirrored: "MR",
  line: "LN",
  single: "SG",
  "center-sym": "CS",
  "bars-thick": "BT",
  "led-matrix": "LM",
  "neon-curve": "NC",
  "dual-neon": "DN",
  "dots-pulse": "DP",
  "blocks-retro": "BR",
  "flame-peak": "FP",
  "sc-fine": "SF",
  minimal: "MN",
};

const DENSITY_MAP: Record<UiDensity, string> = {
  compact: "CP",
  comfortable: "CF",
  spacious: "SP",
};

const RADIUS_MAP: Record<RadiusStyle, string> = {
  sharp: "R0",
  rounded: "R1",
  soft: "R2",
};

export function encodeConfigCode(cfg: UiConfig): string {
  const palette = getPalette(cfg.paletteId);
  const suffix = extrasSuffix(cfg);
  const paletteSuffix = palette ? `-P${palette.id.replaceAll("-", "_").toUpperCase()}` : "";
  for (const p of BUILTIN_PRESETS) {
    const c = p.config;
    if (
      cfg.paletteId === c.paletteId &&
      suffix === extrasSuffix(presetToConfig(p)) &&
      Math.round(c.accentHue) === Math.round(cfg.accentHue) &&
      Math.round(c.accentChroma * 100) === Math.round(cfg.accentChroma * 100) &&
      Math.round(c.accentLightness * 100) === Math.round(cfg.accentLightness * 100) &&
      c.canvasMode === cfg.canvasMode &&
      c.waveformStyle === cfg.waveformStyle &&
      c.waveformBars === cfg.waveformBars &&
      c.density === cfg.density &&
      c.radiusStyle === cfg.radiusStyle &&
      c.sidebarWidth === cfg.sidebarWidth &&
      c.ambientGlow === cfg.ambientGlow &&
      c.winControlsMode === cfg.winControlsMode &&
      Math.round(c.fontScale * 100) === Math.round(cfg.fontScale * 100)
    ) {
      return p.code;
    }
  }

  const h = Math.round(cfg.accentHue);
  const c = Math.round(cfg.accentChroma * 100);
  const l = Math.round(cfg.accentLightness * 100);

  const bh = Math.round(cfg.bgHue ?? 242);
  const bc = Math.round((cfg.bgChroma ?? 0.012) * 1000);
  const bl = Math.round((cfg.bgLightness ?? 0.095) * 1000);
  const cv =
    cfg.canvasMode === "custom"
      ? `CU.${bh}.${bc}.${bl}`
      : (CANVAS_MAP[cfg.canvasMode] ?? "DP");

  const wf = `${WF_MAP[cfg.waveformStyle] ?? "MR"}${cfg.waveformBars}`;
  const dn = DENSITY_MAP[cfg.density] ?? "CF";
  const rd = RADIUS_MAP[cfg.radiusStyle] ?? "R1";

  const isLayoutDefault =
    Math.round(cfg.sidebarWidth) === 236 &&
    cfg.ambientGlow === true &&
    cfg.winControlsMode === "auto-hide" &&
    Math.round(cfg.fontScale * 100) === 100;

  if (isLayoutDefault) {
    return `DLC-${h}.${c}.${l}-${cv}-${wf}-${dn}-${rd}${suffix}${paletteSuffix}`;
  }

  const sb = `S${Math.round(cfg.sidebarWidth)}`;
  const gl = cfg.ambientGlow ? "G1" : "G0";
  const wc = cfg.winControlsMode === "always" ? "W1" : "W0";
  const fs = `F${Math.round(cfg.fontScale * 100)}`;

  return `DLC-${h}.${c}.${l}-${cv}-${wf}-${dn}-${rd}-${sb}-${gl}-${wc}-${fs}${suffix}${paletteSuffix}`;
}

export function decodeConfigCode(
  raw: string,
  savedProfiles: SavedUserProfile[] = [],
): UiConfig | null {
  const clean = raw.trim().toUpperCase();
  if (!clean) return null;

  // 1. Check user saved profiles FIRST (by user's custom code, autoCode, or profile name)
  const saved = savedProfiles.find(
    (s) =>
      s.code.toUpperCase() === clean ||
      s.autoCode?.toUpperCase() === clean ||
      s.name.trim().toUpperCase() === clean,
  );
  if (saved) {
    return normalizeConfig({ ...saved.config });
  }

  // 2. Check built-in preset codes or IDs
  const builtin = BUILTIN_PRESETS.find(
    (p) =>
      p.code.toUpperCase() === clean ||
      p.id.toUpperCase() === clean ||
      p.code.replace("DLC-", "") === clean,
  );
  if (builtin) {
    return presetToConfig(builtin);
  }

  // 3. Parse dynamic code: DLC-238.17.70-DP-MR68-CF-R1[-S236-G1-W0-F100][-X...]
  if (!clean.startsWith("DLC-")) return null;
  const rawParts = clean.slice(4).split("-");
  if (rawParts.length < 4) return null;

  const xPart = rawParts.find((p) => p.startsWith("X"));
  const palettePart = rawParts.find((p) => p.startsWith("P"));
  const palette = palettePart ? getPalette(palettePart.slice(1).toLowerCase().replaceAll("_", "-")) : undefined;
  if (palettePart && !palette) return null;
  const parts = rawParts.filter((p) => !p.startsWith("X") && !p.startsWith("P"));

  const [hclPart, cvPart, wfPart, dnPart, rdPart, sbPart, glPart, wcPart, fsPart] = parts;
  const hcl = hclPart.split(".").map((n) => Number(n));
  if (hcl.length !== 3 || hcl.some((n) => Number.isNaN(n))) return null;

  const accentHue = Math.max(0, Math.min(360, hcl[0]));
  const accentChroma = Math.max(0.04, Math.min(0.28, hcl[1] / 100));
  const accentLightness = Math.max(0.45, Math.min(0.88, hcl[2] / 100));

  let canvasMode: CanvasMode = "deep";
  let bgHue = accentHue;
  let bgChroma = 0.012;
  let bgLightness = 0.095;

  if (cvPart?.startsWith("CU.")) {
    canvasMode = "custom";
    const bgNums = cvPart
      .slice(3)
      .split(".")
      .map((n) => Number(n));
    if (bgNums.length === 3 && !bgNums.some((n) => Number.isNaN(n))) {
      bgHue = Math.max(0, Math.min(360, bgNums[0]));
      bgChroma = Math.max(0, Math.min(0.18, bgNums[1] / 1000));
      bgLightness = Math.max(0.01, Math.min(0.98, bgNums[2] / 1000));
    }
  } else {
    const invCanvas: Record<string, CanvasMode> = {
      DP: "deep",
      OL: "oled",
      GR: "graphite",
      MN: "midnight",
      WN: "warm-noir",
    };
    canvasMode = invCanvas[cvPart] ?? "deep";
  }

  const wfPrefix = wfPart?.slice(0, 2) ?? "MR";
  const wfBarsNum = Number(wfPart?.slice(2) || 68);
  const invWf: Record<string, WaveformStyle> = {
    MR: "mirrored",
    LN: "line",
    SG: "single",
    CS: "center-sym",
    BT: "bars-thick",
    LM: "led-matrix",
    NC: "neon-curve",
    DN: "dual-neon",
    DP: "dots-pulse",
    BR: "blocks-retro",
    FP: "flame-peak",
    SF: "sc-fine",
    MN: "minimal",
  };
  const waveformStyle = invWf[wfPrefix] ?? "mirrored";
  const waveformBars =
    !Number.isNaN(wfBarsNum) && wfBarsNum >= 24 && wfBarsNum <= 240 ? Math.round(wfBarsNum) : 68;

  const invDn: Record<string, UiDensity> = {
    CP: "compact",
    CF: "comfortable",
    SP: "spacious",
  };
  const density = invDn[dnPart] ?? "comfortable";

  const invRd: Record<string, RadiusStyle> = {
    R0: "sharp",
    R1: "rounded",
    R2: "soft",
  };
  const radiusStyle = invRd[rdPart] ?? "rounded";

  const sbNum = Number(sbPart?.replace("S", "") || 236);
  const sidebarWidth = !Number.isNaN(sbNum) ? Math.max(190, Math.min(280, sbNum)) : 236;

  const ambientGlow = glPart ? glPart === "G1" : true;
  const winControlsMode: WinControlsMode = wcPart === "W1" ? "always" : "auto-hide";

  const fsNum = Number(fsPart?.replace("F", "") || 100) / 100;
  const fontScale = !Number.isNaN(fsNum) ? Math.max(0.88, Math.min(1.15, fsNum)) : 1;

  const extras: UiExtras = {
    ...EXTRA_DEFAULTS,
    cornerRadius: radiusStyle === "sharp" ? 4 : radiusStyle === "soft" ? 24 : 16,
  };
  if (xPart?.startsWith("X")) {
    const n = xPart
      .slice(1)
      .split(".")
      .map((v) => Number(v));
    if (n.length >= 7 && !n.some((v) => Number.isNaN(v))) {
      extras.bgGradient = GRAD_CODES[n[0]] ?? "flat";
      extras.bgHue2 = Math.max(0, Math.min(360, n[1]));
      extras.bgGradientStrength = Math.max(0, Math.min(1, n[2] / 100));
      extras.surfaceContrast = Math.max(0.4, Math.min(2.2, n[3] / 100));
      extras.sidebarShade = SHADE_CODES[n[4]] ?? "same";
      extras.textLightness = Math.max(0.78, Math.min(1, n[5] / 1000));
      extras.borderStrength = Math.max(0.3, Math.min(2, n[6] / 100));
      if (n.length >= 8) {
        extras.colorScheme = SCHEME_CODES[n[7]] ?? "dark";
      }
      if (n.length >= 9) {
        extras.cornerRadius = Math.max(0, Math.min(28, Math.round(n[8])));
      }
    }
  }

  return {
    presetId: "custom",
    ...extras,
    paletteId: palette?.id ?? null,
    accentHue,
    accentChroma,
    accentLightness,
    canvasMode,
    bgHue,
    bgChroma,
    bgLightness,
    waveformStyle,
    waveformBars,
    density,
    radiusStyle,
    sidebarWidth,
    ambientGlow,
    winControlsMode,
    fontScale,
  };
}

class ThemeStore {
  config = $state<UiConfig>({ ...DEFAULT_CONFIG });
  savedProfiles = $state<SavedUserProfile[]>([]);
  savedWallpapers = $state<SavedBgWallpaper[]>([]);
  initialized = false;
  systemLight = $state(false);

  get activePalette(): ThemePalette | undefined {
    const palette = getPalette(this.config.paletteId);
    if (palette && this.config.colorScheme === "auto") {
      const scheme = this.systemLight ? "light" : "dark";
      return THEME_PALETTES.find((p) => p.family === palette.family && p.scheme === scheme) ?? palette;
    }
    return palette;
  }

  get currentCode(): string {
    const auto = encodeConfigCode(this.config);
    const matched = this.savedProfiles.find((p) => p.autoCode === auto);
    if (matched) return matched.code;
    return auto;
  }

  init() {
    if (this.initialized || typeof window === "undefined") return;
    this.initialized = true;
    this.systemLight = window.matchMedia?.("(prefers-color-scheme: light)").matches ?? false;

    try {
      const rawCfg =
        localStorage.getItem(STORAGE_KEY_CONFIG) ?? localStorage.getItem("dlc.ui.config.v1");
      if (rawCfg) {
        const parsed = JSON.parse(rawCfg);
        this.config = normalizeConfig(parsed);
      }
    } catch {
      // ignore
    }

    try {
      const rawProfiles =
        localStorage.getItem(STORAGE_KEY_PROFILES) ?? localStorage.getItem("dlc.ui.profiles.v1");
      if (rawProfiles) {
        this.savedProfiles = JSON.parse(rawProfiles);
      }
    } catch {
      // ignore
    }

    try {
      const rawWp = localStorage.getItem(STORAGE_KEY_WALLPAPERS);
      if (rawWp) {
        this.savedWallpapers = JSON.parse(rawWp);
      }
    } catch {
      // ignore
    }

    this.applyToDom();

    if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
      try {
        const mql = window.matchMedia("(prefers-color-scheme: light)");
        mql.addEventListener?.("change", () => {
          this.systemLight = mql.matches;
          if (this.config.colorScheme === "auto") {
            this.applyToDom();
          }
        });
      } catch {
        // ignore
      }
    }
  }

  setColorScheme(scheme: ColorScheme) {
    const current = this.activePalette;
    if (current) {
      const variant = scheme === "auto" ? current :
        current.scheme === scheme ? current :
        THEME_PALETTES.find((p) => p.family === current.family && p.scheme === scheme);
      if (variant) {
        this.update({ ...paletteConfig(variant), colorScheme: scheme, presetId: variant.id });
        return;
      }
    }
    this.update({ colorScheme: scheme });
  }

  private persist() {
    if (typeof window === "undefined") return;
    try {
      localStorage.setItem(STORAGE_KEY_CONFIG, JSON.stringify(this.config));
      emit("dlc-theme-updated", this.config).catch(() => {});
    } catch {
      // ignore
    }
  }

  private persistProfiles() {
    if (typeof window === "undefined") return;
    try {
      localStorage.setItem(STORAGE_KEY_PROFILES, JSON.stringify(this.savedProfiles));
    } catch {
      // ignore
    }
  }

  private persistWallpapers() {
    if (typeof window === "undefined") return;
    try {
      localStorage.setItem(STORAGE_KEY_WALLPAPERS, JSON.stringify(this.savedWallpapers));
    } catch {
      // ignore
    }
  }

  async setCustomBgImage(
    src: string | null,
    name?: string,
    displayUrlForPalette?: string,
    forceSyncColors = false,
  ) {
    if (!src || !src.trim()) {
      this.update({ bgImage: null });
      return;
    }
    const cleanSrc = src.trim();
    const isBuiltin = BUILTIN_WALLPAPERS.some((w) => w.src === cleanSrc);
    if (!isBuiltin && !this.savedWallpapers.some((w) => w.src === cleanSrc)) {
      const item: SavedBgWallpaper = {
        id: `wp-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 5)}`,
        name: name?.trim() || `Ảnh nền #${this.savedWallpapers.length + 1}`,
        src: cleanSrc,
        addedAt: new Date().toLocaleDateString("vi-VN"),
      };
      this.savedWallpapers = [item, ...this.savedWallpapers].slice(0, 24);
      this.persistWallpapers();
    }

    this.update({ bgImage: cleanSrc });

    if (forceSyncColors || this.config.bgImageAutoTheme) {
      await this.syncColorsFromBgImage(displayUrlForPalette || cleanSrc);
    }
  }

  async syncColorsFromBgImage(displayUrl: string): Promise<boolean> {
    if (!displayUrl) return false;
    const builtin = BUILTIN_WALLPAPERS.find((w) => w.src === displayUrl || w.src === this.config.bgImage);
    if (builtin) {
      this.update({
        canvasMode: "custom",
        accentHue: builtin.accentHue,
        accentChroma: 0.2,
        accentLightness: 0.72,
        bgHue: builtin.bgHue,
        bgChroma: 0.032,
        bgLightness: 0.09,
        bgHue2: (builtin.accentHue + 36) % 360,
      });
      return true;
    }
    const pal = await extractPaletteFromImage(displayUrl);
    if (!pal) return false;
    this.update({
      canvasMode: "custom",
      accentHue: pal.accentHue,
      accentChroma: pal.accentChroma,
      accentLightness: pal.accentLightness,
      bgHue: pal.bgHue,
      bgChroma: 0.03,
      bgLightness: 0.09,
      bgHue2: pal.bgHue2,
    });
    return true;
  }

  removeSavedWallpaper(id: string): string | null {
    const found = this.savedWallpapers.find((w) => w.id === id);
    this.savedWallpapers = this.savedWallpapers.filter((w) => w.id !== id);
    this.persistWallpapers();
    if (found && this.config.bgImage === found.src) {
      this.update({ bgImage: null });
    }
    return found?.src ?? null;
  }

  update(patch: Partial<UiConfig>) {
    const nextPatch: Partial<UiConfig> = { ...patch };
    const colorKeys: (keyof UiConfig)[] = [
      "accentHue", "accentChroma", "accentLightness", "canvasMode", "bgHue", "bgChroma",
      "bgLightness", "bgGradient", "bgHue2", "bgGradientStrength", "surfaceContrast",
      "sidebarShade", "textLightness", "borderStrength", "colorScheme",
    ];
    if (patch.paletteId === undefined && colorKeys.some((key) => key in patch)) {
      nextPatch.paletteId = null;
    }
    if (typeof nextPatch.cornerRadius === "number" && !Number.isNaN(nextPatch.cornerRadius)) {
      const r = Math.max(0, Math.min(28, Math.round(nextPatch.cornerRadius)));
      nextPatch.cornerRadius = r;
      if (nextPatch.radiusStyle === undefined) {
        nextPatch.radiusStyle = r <= 8 ? "sharp" : r >= 20 ? "soft" : "rounded";
      }
    } else if (nextPatch.radiusStyle !== undefined && nextPatch.cornerRadius === undefined) {
      nextPatch.cornerRadius =
        nextPatch.radiusStyle === "sharp" ? 4 : nextPatch.radiusStyle === "soft" ? 24 : 16;
    }
    this.config = {
      ...this.config,
      ...nextPatch,
      presetId: nextPatch.presetId ?? "custom",
    };
    this.applyToDom();
    this.persist();
  }

  setCustomBackgroundHex(hex: string) {
    const { l, c, h } = hexToOklchApprox(hex);
    this.update({
      canvasMode: "custom",
      colorScheme: l >= 0.52 ? "light" : "dark",
      bgHue: h,
      bgChroma: Math.min(0.18, c),
      bgLightness: Math.max(0.01, Math.min(0.98, l)),
    });
  }

  setCustomAccentHex(hex: string) {
    const { l, c, h } = hexToOklchApprox(hex);
    this.update({
      accentHue: h,
      accentChroma: Math.max(0.08, Math.min(0.25, c || 0.16)),
      accentLightness: Math.max(0.55, Math.min(0.82, l || 0.7)),
    });
  }

  applyPreset(presetId: string) {
    const palette = getPalette(presetId);
    if (!palette) return;
    this.config = { ...this.config, ...paletteConfig(palette), presetId };
    this.applyToDom();
    this.persist();
  }

  applyCode(code: string): boolean {
    const decoded = decodeConfigCode(code, this.savedProfiles);
    if (!decoded) return false;
    this.config = decoded;
    this.applyToDom();
    this.persist();
    return true;
  }

  saveProfile(name: string, customCodeInput = "", externalConfig?: UiConfig): SavedUserProfile {
    const cfgCopy: UiConfig = externalConfig ? { ...externalConfig } : { ...this.config };
    const autoCode = encodeConfigCode(cfgCopy);
    const formattedCustomCode = customCodeInput
      .trim()
      .toUpperCase()
      .replace(/\s+/g, "-");
    const finalCode = formattedCustomCode || autoCode;
    const cleanName =
      name.trim() ||
      (formattedCustomCode ? `Phong cách ${formattedCustomCode}` : `Cấu hình #${this.savedProfiles.length + 1}`);

    // If a profile with the same custom code already exists, update it in place
    const existingIdx = this.savedProfiles.findIndex(
      (p) => p.code.toUpperCase() === finalCode,
    );

    const item: SavedUserProfile = {
      id: existingIdx >= 0 ? this.savedProfiles[existingIdx].id : `prof-${Date.now().toString(36)}`,
      name: cleanName,
      code: finalCode,
      autoCode,
      createdAt: new Date().toLocaleDateString("vi-VN"),
      config: cfgCopy,
    };

    if (existingIdx >= 0) {
      const copy = [...this.savedProfiles];
      copy[existingIdx] = item;
      this.savedProfiles = copy;
    } else {
      this.savedProfiles = [item, ...this.savedProfiles];
    }

    this.persistProfiles();
    return item;
  }

  deleteProfile(id: string) {
    this.savedProfiles = this.savedProfiles.filter((p) => p.id !== id);
    this.persistProfiles();
  }

  resetDefault() {
    const found = BUILTIN_PRESETS.find((p) => p.id === DEFAULT_PALETTE_ID)!;
    this.config = presetToConfig(found);
    this.applyToDom();
    this.persist();
  }

  applyToDom() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const c = this.config;
    const h = c.accentHue;
    const ch = c.accentChroma;
    const l = c.accentLightness;

    // Surface / Canvas palette — unified derivation so every background option is customizable
    const x = normalizeConfig(c);
    const palette = this.activePalette;
    const hasBgImage = Boolean(x.bgImage && x.bgImage.trim());
    type Base = { l: number; c: number; h: number; steps: [number, number, number]; floor: number };
    const MODE_BASE: Record<Exclude<CanvasMode, "custom">, Base> = {
      deep: { l: 0.095, c: 0.014, h, steps: [0.05, 0.095, 0.15], floor: 0.005 },
      oled: { l: 0.02, c: 0, h, steps: [0.08, 0.13, 0.185], floor: 0.008 },
      graphite: { l: 0.11, c: 0.006, h, steps: [0.05, 0.095, 0.15], floor: 0.005 },
      midnight: { l: 0.1, c: 0.026, h: (h + 20) % 360, steps: [0.052, 0.1, 0.155], floor: 0.01 },
      "warm-noir": { l: 0.1, c: 0.014, h: 60, steps: [0.052, 0.1, 0.155], floor: 0.008 },
    };
    const base: Base =
      c.canvasMode === "custom"
        ? {
            l: c.bgLightness ?? 0.095,
            c: c.bgChroma ?? 0.02,
            h: c.bgHue ?? h,
            steps: [0.05, 0.095, 0.15],
            floor: 0.005,
          }
        : MODE_BASE[c.canvasMode];

    // Light / dark scheme resolution — also auto-detects when user sets a bright custom background (L >= 0.52)
    const prefersLight =
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: light)").matches;
    const schemeWantsLight = x.colorScheme === "light" || (x.colorScheme === "auto" && prefersLight);

    const LIGHT_CANVAS: Record<CanvasMode, number> = {
      deep: 0.962,
      oled: 0.985,
      graphite: 0.952,
      midnight: 0.95,
      "warm-noir": 0.962,
      custom:
        base.l >= 0.52
          ? Math.min(0.985, base.l)
          : Math.min(0.985, 0.88 + (Math.max(0, base.l) / 0.52) * 0.1),
    };
    const canvasL = schemeWantsLight ? LIGHT_CANVAS[c.canvasMode] : base.l;
    const isLight = palette ? palette.scheme === "light" : canvasL >= 0.52;

    const k = Math.max(0.4, Math.min(2.2, x.surfaceContrast));
    const L = (v: number) => Math.max(0, Math.min(0.995, v));

    const tintC = isLight ? Math.min(0.08, base.c * 0.9 + 0.004) : base.c;
    const chromaFor = (mult: number) =>
      isLight ? Math.min(0.09, tintC * mult) : Math.min(0.2, Math.max(base.floor, base.c * mult));
    const canvas = `oklch(${canvasL} ${tintC} ${base.h})`;

    const s1 = isLight
      ? L(Math.max(0.58, Math.min(0.995, canvasL + 0.03)))
      : L(base.l + base.steps[0] * k);
    const s2 = isLight
      ? L(Math.max(0.54, canvasL - 0.03 * k))
      : L(base.l + base.steps[1] * k);
    const s3 = isLight
      ? L(Math.max(0.5, canvasL - 0.068 * k))
      : L(base.l + base.steps[2] * k);

    const minSurfL = Math.min(canvasL, s1, s2, s3);
    const maxSurfL = Math.max(canvasL, s1, s2, s3);

    // Contrast-safe Accent palette: never let accent blend into light or same-color backgrounds
    const effectiveAccentL = isLight
      ? Math.max(0.32, Math.min(l, minSurfL - 0.36, 0.52))
      : Math.min(0.88, Math.max(l, maxSurfL + 0.36, 0.62));
    const effectiveAccentChroma = isLight ? Math.max(ch, 0.16) : ch;
    const accentFg = effectiveAccentL > 0.62 ? `oklch(0.10 0.02 ${h})` : "#ffffff";

    root.style.setProperty("--accent", `oklch(${effectiveAccentL} ${effectiveAccentChroma} ${h})`);
    root.style.setProperty(
      "--accent-hover",
      isLight
        ? `oklch(${Math.max(0.26, effectiveAccentL - 0.06)} ${effectiveAccentChroma} ${(h - 6 + 360) % 360})`
        : `oklch(${Math.min(0.9, effectiveAccentL + 0.06)} ${effectiveAccentChroma} ${(h - 6 + 360) % 360})`,
    );
    root.style.setProperty("--accent-fg", accentFg);
    root.style.setProperty(
      "--accent-soft",
      `oklch(${effectiveAccentL} ${effectiveAccentChroma} ${h} / ${isLight ? 0.14 : 0.16})`,
    );
    root.style.setProperty(
      "--accent-glow",
      `oklch(${effectiveAccentL} ${effectiveAccentChroma} ${h} / ${c.ambientGlow ? (isLight ? 0.24 : 0.34) : 0.08})`,
    );
    root.style.setProperty(
      "--sc-tone",
      `oklch(${isLight ? effectiveAccentL : Math.min(0.84, effectiveAccentL + 0.02)} ${effectiveAccentChroma} ${h})`,
    );
    root.style.setProperty("--sc-soft", `oklch(${effectiveAccentL} ${effectiveAccentChroma} ${h} / 0.16)`);

    const g2 = Math.max(0, Math.min(1, x.bgGradientStrength));
    const color2 = isLight
      ? `oklch(${L(Math.max(0.55, canvasL - 0.01 - g2 * 0.07))} ${Math.min(0.12, 0.025 + g2 * 0.07)} ${x.bgHue2})`
      : `oklch(${L(base.l + 0.015 + g2 * 0.1)} ${Math.min(0.2, 0.03 + g2 * 0.1)} ${x.bgHue2})`;
    let appBg = canvas;
    if (x.bgGradient === "vertical") appBg = `linear-gradient(180deg, ${canvas} 0%, ${color2} 100%)`;
    else if (x.bgGradient === "diagonal") appBg = `linear-gradient(135deg, ${canvas} 0%, ${color2} 100%)`;
    else if (x.bgGradient === "radial")
      appBg = `radial-gradient(ellipse 90% 70% at 15% 0%, ${color2} 0%, ${canvas} 65%)`;

    const sbL = isLight
      ? x.sidebarShade === "darker"
        ? L(Math.max(0.52, canvasL - 0.05))
        : x.sidebarShade === "lighter"
          ? L(Math.min(0.995, canvasL + 0.03))
          : canvasL
      : x.sidebarShade === "darker"
        ? Math.max(0.01, base.l * 0.55)
        : x.sidebarShade === "lighter"
          ? L(base.l + 0.035)
          : base.l;

    if (hasBgImage) {
      const rawAlpha = Math.max(0.12, Math.min(0.95, x.bgImageSurfaceAlpha ?? 0.54));
      const alpha = isLight ? Math.max(0.76, rawAlpha) : rawAlpha;
      const canvasAlpha = isLight
        ? Math.max(0.68, Math.min(0.92, alpha * 0.85))
        : Math.max(0.08, Math.min(0.88, alpha * 0.62));
      root.style.setProperty("--bg-canvas", `oklch(${canvasL} ${tintC} ${base.h} / ${canvasAlpha})`);
      root.style.setProperty("--bg-surface-1", `oklch(${s1} ${chromaFor(1.1)} ${base.h} / ${alpha})`);
      root.style.setProperty(
        "--bg-surface-2",
        `oklch(${s2} ${chromaFor(1.2)} ${base.h} / ${Math.min(0.96, alpha + 0.12)})`,
      );
      root.style.setProperty(
        "--bg-surface-3",
        `oklch(${s3} ${chromaFor(1.3)} ${base.h} / ${Math.min(0.98, alpha + 0.2)})`,
      );
      root.style.setProperty(
        "--bg-glass",
        `oklch(${isLight ? L(canvasL + 0.02) : L(base.l + 0.015)} ${chromaFor(1)} ${base.h} / ${Math.min(0.94, alpha + 0.14)})`,
      );
      root.style.setProperty("--bg-app", "transparent");
      root.style.setProperty("--bg-overlay-tint", appBg);
      const sbAlpha =
        x.sidebarShade === "same"
          ? Math.max(isLight ? 0.65 : 0.06, alpha * (isLight ? 0.82 : 0.42))
          : x.sidebarShade === "darker"
            ? Math.min(0.92, alpha + 0.1)
            : alpha;
      root.style.setProperty("--bg-sidebar", `oklch(${sbL} ${tintC} ${base.h} / ${sbAlpha})`);
      const effDim = isLight
        ? Math.max(0.58, Math.min(0.92, x.bgImageDim ?? 0.48))
        : Math.max(0.05, Math.min(0.92, x.bgImageDim ?? 0.48));
      root.style.setProperty("--bg-image-dim", `${effDim}`);
      root.style.setProperty("--bg-image-blur", `${Math.max(0, Math.min(28, x.bgImageBlur ?? 0))}px`);
      root.style.setProperty(
        "--bg-image-brightness",
        `${Math.max(0.35, Math.min(1.45, x.bgImageBrightness ?? 1))}`,
      );
      root.style.setProperty(
        "--bg-image-saturation",
        `${Math.max(0.3, Math.min(1.85, x.bgImageSaturation ?? 1.08))}`,
      );
      root.style.setProperty("--bg-image-fit", x.bgImageFit || "cover");
      root.style.setProperty("--bg-image-pos", x.bgImagePosition || "center");
      root.dataset.bgImage = "on";
    } else {
      root.style.setProperty("--bg-canvas", canvas);
      root.style.setProperty("--bg-surface-1", `oklch(${s1} ${chromaFor(1.1)} ${base.h})`);
      root.style.setProperty("--bg-surface-2", `oklch(${s2} ${chromaFor(1.2)} ${base.h})`);
      root.style.setProperty("--bg-surface-3", `oklch(${s3} ${chromaFor(1.3)} ${base.h})`);
      root.style.setProperty(
        "--bg-glass",
        `oklch(${isLight ? L(canvasL + 0.02) : L(base.l + 0.015)} ${chromaFor(1)} ${base.h} / ${isLight ? 0.9 : 0.88})`,
      );
      root.style.setProperty("--bg-app", appBg);
      root.style.setProperty(
        "--bg-sidebar",
        x.bgGradient !== "flat" && x.sidebarShade === "same"
          ? "transparent"
          : `oklch(${sbL} ${tintC} ${base.h})`,
      );
      root.dataset.bgImage = "off";
    }

    const bs = Math.max(0.3, Math.min(2, x.borderStrength));
    root.style.setProperty(
      "--border-subtle",
      isLight
        ? `oklch(${L(minSurfL - 0.14)} ${Math.max(0.015, tintC)} ${base.h} / ${Math.min(1, 0.85 * bs)})`
        : `oklch(${L(base.l + 0.14)} ${Math.max(0.015, base.c)} ${base.h} / ${Math.min(1, 0.6 * bs)})`,
    );
    root.style.setProperty(
      "--border-strong",
      isLight
        ? `oklch(${L(minSurfL - 0.26)} ${Math.max(0.025, tintC * 1.2)} ${base.h} / ${Math.min(1, 0.92 * bs)})`
        : `oklch(${L(base.l + 0.24)} ${Math.max(0.03, base.c * 1.3)} ${base.h} / ${Math.min(1, 0.75 * bs)})`,
    );
    root.style.setProperty(
      "--stage-glow",
      `oklch(${effectiveAccentL} ${effectiveAccentChroma} ${h} / ${c.ambientGlow ? (isLight ? 0.1 : 0.09) : 0})`,
    );

    // Contrast-guaranteed Text tones: always readable even if background has the same hue or custom brightness
    const tl = Math.max(0.78, Math.min(1, x.textLightness));
    if (isLight) {
      const pL = Math.max(0.08, Math.min(0.18, minSurfL - 0.5, 1.06 - tl));
      const secL = Math.max(0.22, Math.min(0.33, minSurfL - 0.36, pL + 0.13));
      const mutL = Math.max(0.34, Math.min(0.44, minSurfL - 0.24, secL + 0.11));
      root.style.setProperty("--text-primary", `oklch(${pL} 0.022 ${base.h})`);
      root.style.setProperty("--text-secondary", `oklch(${secL} 0.025 ${base.h})`);
      root.style.setProperty("--text-muted", `oklch(${mutL} 0.02 ${base.h})`);
    } else {
      const pL = Math.max(tl, Math.min(0.99, maxSurfL + 0.52));
      const secL = Math.max(tl - 0.2, Math.min(0.9, maxSurfL + 0.36));
      const mutL = Math.max(tl - 0.36, Math.min(0.8, maxSurfL + 0.24));
      root.style.setProperty("--text-primary", `oklch(${pL} 0.008 ${base.h})`);
      root.style.setProperty("--text-secondary", `oklch(${secL} 0.018 ${base.h})`);
      root.style.setProperty("--text-muted", `oklch(${mutL} 0.018 ${base.h})`);
    }

    if (palette) {
      const p = palette.colors;
      const backgrounds = [p.base, p.surface, p.overlay, p.sidebar];
      const accent = isLight ? readableColor(p.accent, backgrounds) : p.accent;
      const alpha = hasBgImage ? Math.max(isLight ? 0.88 : 0.12, Math.min(0.95, x.bgImageSurfaceAlpha)) : 1;
      const translucent = (hex: string, opacity: number) =>
        `color-mix(in srgb, ${hex} ${opacity * 100}%, transparent)`;
      const tokens: Record<string, string> = {
        "bg-canvas": translucent(p.base, hasBgImage ? alpha * 0.85 : 1),
        "bg-surface-1": translucent(p.surface, alpha),
        "bg-surface-2": translucent(p.overlay, Math.min(1, alpha + 0.1)),
        "bg-surface-3": translucent(p.overlay, Math.min(1, alpha + 0.18)),
        "bg-glass": translucent(p.surface, Math.min(0.94, alpha)),
        "bg-sidebar": translucent(p.sidebar, alpha),
        "bg-app": hasBgImage ? "transparent" : p.base,
        "bg-overlay-tint": p.base,
        "border-subtle": p.border,
        "border-strong": readableColor(p.border, backgrounds, 3),
        "text-primary": readableColor(p.text, backgrounds),
        "text-secondary": readableColor(p.secondary, backgrounds),
        "text-muted": readableColor(p.muted, backgrounds),
        "accent": accent,
        "accent-fg": contrastRatio(accent, "#ffffff") > contrastRatio(accent, "#000000") ? "#ffffff" : "#000000",
        "accent-hover": `color-mix(in srgb, ${accent} 90%, ${isLight ? "black" : "white"})`,
        "accent-soft": translucent(accent, isLight ? 0.12 : 0.16),
        "accent-glow": translucent(accent, c.ambientGlow ? 0.25 : 0.08),
        "stage-glow": translucent(accent, c.ambientGlow ? 0.09 : 0),
        "sc-tone": accent,
        "sc-soft": translucent(accent, 0.16),
        "signal-online": readableColor(p.success, backgrounds),
        "signal-online-soft": translucent(p.success, 0.15),
        "danger": readableColor(p.danger, backgrounds),
      };
      for (const [name, value] of Object.entries(tokens)) root.style.setProperty(`--${name}`, value);
    }
    root.dataset.palette = palette?.id ?? "custom";

    // Idle waveform colours adapt to the scheme
    root.style.setProperty("--wave-idle", palette?.colors.border ?? (isLight ? "oklch(0.68 0.025 240)" : "oklch(0.32 0.03 240)"));
    root.style.setProperty(
      "--wave-idle-2",
      palette ? `color-mix(in srgb, ${palette.colors.border} 65%, transparent)` :
        isLight ? "oklch(0.76 0.02 240 / 0.8)" : "oklch(0.24 0.025 240 / 0.65)",
    );
    root.style.setProperty("--wave-idle-hover", palette?.colors.muted ?? (isLight ? "oklch(0.55 0.035 240)" : "oklch(0.4 0.04 240)"));
    root.style.colorScheme = isLight ? "light" : "dark";
    root.dataset.scheme = isLight ? "light" : "dark";

    const r = Math.max(
      0,
      Math.min(
        28,
        Math.round(
          x.cornerRadius ??
            (c.radiusStyle === "sharp" ? 4 : c.radiusStyle === "soft" ? 24 : 16),
        ),
      ),
    );
    const rLg = Math.round(r * 1.25);
    const rMd = Math.round(r * 0.88);
    const rSm = Math.round(r * 0.68);
    const rBar = r === 0 ? 0 : r >= 16 ? 999 : Math.round(r * 1.35);

    root.style.setProperty("--ui-radius", `${r}px`);
    root.style.setProperty("--ui-radius-lg", `${rLg}px`);
    root.style.setProperty("--ui-radius-md", `${rMd}px`);
    root.style.setProperty("--ui-radius-sm", `${rSm}px`);
    root.style.setProperty("--ui-radius-bar", `${rBar}px`);
    root.style.setProperty("--sidebar-w", `${c.sidebarWidth}px`);
    root.style.setProperty("--font-scale", `${c.fontScale}`);
    root.style.fontSize = `${Math.round(c.fontScale * 100)}%`;

    root.dataset.density = c.density;
    root.dataset.radius = c.radiusStyle;
    root.dataset.wfStyle = c.waveformStyle;
    root.dataset.glow = c.ambientGlow ? "on" : "off";
    const isDroplet = Boolean(x.waterDropletMode);
    root.dataset.droplet = isDroplet ? "on" : "off";
    if (isDroplet) {
      root.style.setProperty(
        "--droplet-gloss",
        "inset 0 1.5px 2px 0 rgba(255, 255, 255, 0.48), inset 0 -2px 6px 0 rgba(0, 0, 0, 0.35), 0 8px 30px -4px rgba(0, 180, 255, 0.22)",
      );
      root.style.setProperty(
        "--droplet-highlight",
        "linear-gradient(180deg, rgba(255,255,255,0.38) 0%, rgba(255,255,255,0.08) 35%, rgba(255,255,255,0) 100%)",
      );
      root.style.setProperty("--droplet-border", "1px solid rgba(255, 255, 255, 0.32)");
    } else {
      root.style.removeProperty("--droplet-gloss");
      root.style.removeProperty("--droplet-highlight");
      root.style.removeProperty("--droplet-border");
    }

    const isAppleGlass = Boolean(x.appleGlassMode);
    root.dataset.appleGlass = isAppleGlass ? "on" : "off";
    const isRipple = Boolean(x.waterRippleEffect);
    root.dataset.ripple = isRipple ? "on" : "off";
    const stealth = x.stealthMode ?? true;
    root.dataset.stealth = stealth ? "on" : "off";
    invoke("set_stealth_mode", { enabled: stealth }).catch(() => {
      // ignore if outside Tauri runtime
    });
  }
}

export const theme = new ThemeStore();
