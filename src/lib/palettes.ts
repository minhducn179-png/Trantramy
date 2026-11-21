export interface PaletteColors {
  base: string;
  surface: string;
  overlay: string;
  sidebar: string;
  text: string;
  secondary: string;
  muted: string;
  border: string;
  accent: string;
  success: string;
  danger: string;
}

export interface ThemePalette {
  id: string;
  family: string;
  name: string;
  scheme: "dark" | "light";
  colors: PaletteColors;
  note?: string;
}

export const THEME_FAMILIES = [
  { id: "catppuccin", name: "Catppuccin", description: "Pastel ấm, dịu mắt", default: "catppuccin-mocha", source: "https://catppuccin.com/palette/" },
  { id: "one-dark-pro", name: "One Dark Pro", description: "Xám than trung tính", default: "one-dark-pro-classic", source: "https://github.com/Binaryify/OneDark-Pro" },
  { id: "tokyo-night", name: "Tokyo Night", description: "Xanh đêm và ánh neon", default: "tokyo-night-night", source: "https://github.com/folke/tokyonight.nvim" },
  { id: "dracula", name: "Dracula", description: "Tím than, nhấn tương phản", default: "dracula-classic", source: "https://github.com/dracula/visual-studio-code" },
  { id: "rose-pine", name: "Rosé Pine", description: "Hồng đất và sắc trầm", default: "rose-pine-main", source: "https://rosepinetheme.com/palette/" },
  { id: "nord", name: "Nord", description: "Lạnh, tối giản Bắc Âu", default: "nord-polar-night", source: "https://www.nordtheme.com/docs/colors-and-palettes" },
  { id: "gruvbox", name: "Gruvbox", description: "Tông đất ấm, hoài cổ", default: "gruvbox-dark-medium", source: "https://github.com/morhetz/gruvbox" },
  { id: "ayu", name: "Ayu", description: "Cam hổ phách, sắc nét", default: "ayu-dark", source: "https://github.com/ayu-theme/ayu-colors" },
  { id: "kanagawa", name: "Kanagawa", description: "Mực tàu và sắc sóng biển", default: "kanagawa-wave", source: "https://github.com/rebelot/kanagawa.nvim" },
  { id: "vesper", name: "Vesper", description: "Đen than, cam đào và bạc hà", default: "vesper-dark", source: "https://github.com/raunofreiberg/vesper" },
];

// Semantic mapping for this player, not a claim that editor UI roles are identical.
// Catppuccin, Rosé Pine and Kanagawa use the upstream named palette colors.
export const THEME_PALETTES: ThemePalette[] = [
  { id: "catppuccin-latte", family: "catppuccin", name: "Latte", scheme: "light", colors: {
    base: "#eff1f5", surface: "#e6e9ef", overlay: "#ccd0da", sidebar: "#e6e9ef", text: "#4c4f69", secondary: "#5c5f77", muted: "#6c6f85", border: "#bcc0cc", accent: "#8839ef", success: "#40a02b", danger: "#d20f39",
  } },
  { id: "catppuccin-frappe", family: "catppuccin", name: "Frappé", scheme: "dark", colors: {
    base: "#303446", surface: "#414559", overlay: "#51576d", sidebar: "#292c3c", text: "#c6d0f5", secondary: "#b5bfe2", muted: "#a5adce", border: "#626880", accent: "#ca9ee6", success: "#a6d189", danger: "#e78284",
  } },
  { id: "catppuccin-macchiato", family: "catppuccin", name: "Macchiato", scheme: "dark", colors: {
    base: "#24273a", surface: "#363a4f", overlay: "#494d64", sidebar: "#1e2030", text: "#cad3f5", secondary: "#b8c0e0", muted: "#a5adcb", border: "#5b6078", accent: "#c6a0f6", success: "#a6da95", danger: "#ed8796",
  } },
  { id: "catppuccin-mocha", family: "catppuccin", name: "Mocha", scheme: "dark", colors: {
    base: "#1e1e2e", surface: "#313244", overlay: "#45475a", sidebar: "#181825", text: "#cdd6f4", secondary: "#bac2de", muted: "#a6adc8", border: "#585b70", accent: "#cba6f7", success: "#a6e3a1", danger: "#f38ba8",
  } },
  { id: "one-dark-pro-classic", family: "one-dark-pro", name: "Classic", scheme: "dark", colors: {
    base: "#282c34", surface: "#21252b", overlay: "#2c313a", sidebar: "#21252b", text: "#abb2bf", secondary: "#abb2bf", muted: "#7f848e", border: "#3e4452", accent: "#61afef", success: "#98c379", danger: "#e06c75",
  } },
  { id: "one-dark-pro-darker", family: "one-dark-pro", name: "Darker", scheme: "dark", colors: {
    base: "#23272e", surface: "#1e2227", overlay: "#2c313a", sidebar: "#1e2227", text: "#abb2bf", secondary: "#abb2bf", muted: "#7f848e", border: "#3e4452", accent: "#61afef", success: "#98c379", danger: "#e06c75",
  } },
  { id: "one-dark-pro-flat", family: "one-dark-pro", name: "Flat", scheme: "dark", colors: {
    base: "#282c34", surface: "#282c34", overlay: "#2c313a", sidebar: "#282c34", text: "#abb2bf", secondary: "#abb2bf", muted: "#7f848e", border: "#3e4452", accent: "#61afef", success: "#98c379", danger: "#e06c75",
  } },
  { id: "one-dark-pro-vivid", family: "one-dark-pro", name: "Vivid", scheme: "dark", note: "Bản tùy biến cho DLC: chữ sáng hơn và điểm nhấn rõ hơn; không phải theme riêng của upstream.", colors: {
    base: "#282c34", surface: "#21252b", overlay: "#2c313a", sidebar: "#21252b", text: "#d7dae0", secondary: "#abb2bf", muted: "#9da5b4", border: "#4b5263", accent: "#61afef", success: "#98c379", danger: "#e06c75",
  } },
  { id: "one-dark-pro-mix", family: "one-dark-pro", name: "Mix", scheme: "dark", note: "Mix giữ màu giao diện bản gốc; khác biệt upstream chủ yếu nằm ở tô màu mã nguồn.", colors: {
    base: "#282c34", surface: "#21252b", overlay: "#2c313a", sidebar: "#21252b", text: "#abb2bf", secondary: "#abb2bf", muted: "#7f848e", border: "#3e4452", accent: "#61afef", success: "#98c379", danger: "#e06c75",
  } },
  { id: "tokyo-night-night", family: "tokyo-night", name: "Night", scheme: "dark", colors: {
    base: "#1a1b26", surface: "#16161e", overlay: "#292e42", sidebar: "#16161e", text: "#c0caf5", secondary: "#a9b1d6", muted: "#565f89", border: "#3b4261", accent: "#7aa2f7", success: "#9ece6a", danger: "#f7768e",
  } },
  { id: "tokyo-night-storm", family: "tokyo-night", name: "Storm", scheme: "dark", colors: {
    base: "#24283b", surface: "#1f2335", overlay: "#292e42", sidebar: "#1f2335", text: "#c0caf5", secondary: "#a9b1d6", muted: "#565f89", border: "#3b4261", accent: "#7aa2f7", success: "#9ece6a", danger: "#f7768e",
  } },
  { id: "tokyo-night-moon", family: "tokyo-night", name: "Moon", scheme: "dark", colors: {
    base: "#222436", surface: "#1e2030", overlay: "#2f334d", sidebar: "#1e2030", text: "#c8d3f5", secondary: "#828bb8", muted: "#636da6", border: "#444a73", accent: "#82aaff", success: "#c3e88d", danger: "#ff757f",
  } },
  { id: "tokyo-night-day", family: "tokyo-night", name: "Day", scheme: "light", colors: {
    base: "#e6e7ed", surface: "#d6d8df", overlay: "#dcdee3", sidebar: "#d6d8df", text: "#343b59", secondary: "#565a6e", muted: "#707280", border: "#c1c2c7", accent: "#2959aa", success: "#33635c", danger: "#8c4351",
  } },
  { id: "dracula-classic", family: "dracula", name: "Classic", scheme: "dark", colors: {
    base: "#282a36", surface: "#343746", overlay: "#44475a", sidebar: "#21222c", text: "#f8f8f2", secondary: "#f8f8f2", muted: "#6272a4", border: "#6272a4", accent: "#bd93f9", success: "#50fa7b", danger: "#ff5555",
  } },
  // Upstream scripts/generate.js: desaturate bright colors by 20 HSL points.
  { id: "dracula-soft", family: "dracula", name: "Soft", scheme: "dark", colors: {
    base: "#282a36", surface: "#343746", overlay: "#44475a", sidebar: "#21222c", text: "#f6f6f4", secondary: "#f6f6f4", muted: "#6272a4", border: "#6272a4", accent: "#bf9eee", success: "#62e884", danger: "#ee6666",
  } },
  { id: "rose-pine-main", family: "rose-pine", name: "Main", scheme: "dark", colors: {
    base: "#191724", surface: "#1f1d2e", overlay: "#26233a", sidebar: "#191724", text: "#e0def4", secondary: "#908caa", muted: "#6e6a86", border: "#403d52", accent: "#ebbcba", success: "#9ccfd8", danger: "#eb6f92",
  } },
  { id: "rose-pine-moon", family: "rose-pine", name: "Moon", scheme: "dark", colors: {
    base: "#232136", surface: "#2a273f", overlay: "#393552", sidebar: "#232136", text: "#e0def4", secondary: "#908caa", muted: "#6e6a86", border: "#44415a", accent: "#ea9a97", success: "#9ccfd8", danger: "#eb6f92",
  } },
  { id: "rose-pine-dawn", family: "rose-pine", name: "Dawn", scheme: "light", colors: {
    base: "#faf4ed", surface: "#fffaf3", overlay: "#f2e9e1", sidebar: "#f4ede8", text: "#464261", secondary: "#797593", muted: "#9893a5", border: "#dfdad9", accent: "#d7827e", success: "#286983", danger: "#b4637a",
  } },
  { id: "nord-polar-night", family: "nord", name: "Polar Night", scheme: "dark", colors: {
    base: "#2e3440", surface: "#3b4252", overlay: "#434c5e", sidebar: "#2e3440", text: "#eceff4", secondary: "#e5e9f0", muted: "#d8dee9", border: "#4c566a", accent: "#88c0d0", success: "#a3be8c", danger: "#bf616a",
  } },
  { id: "nord-snow-storm", family: "nord", name: "Snow Storm", scheme: "light", note: "Ánh xạ sáng cho DLC từ các màu Snow Storm, Polar Night và Frost của Nord.", colors: {
    base: "#eceff4", surface: "#e5e9f0", overlay: "#d8dee9", sidebar: "#e5e9f0", text: "#2e3440", secondary: "#3b4252", muted: "#4c566a", border: "#d8dee9", accent: "#5e81ac", success: "#a3be8c", danger: "#bf616a",
  } },
  { id: "gruvbox-dark-soft", family: "gruvbox", name: "Dark Soft", scheme: "dark", colors: {
    base: "#32302f", surface: "#3c3836", overlay: "#504945", sidebar: "#32302f", text: "#ebdbb2", secondary: "#d5c4a1", muted: "#a89984", border: "#665c54", accent: "#fe8019", success: "#b8bb26", danger: "#fb4934",
  } },
  { id: "gruvbox-dark-medium", family: "gruvbox", name: "Dark Medium", scheme: "dark", colors: {
    base: "#282828", surface: "#3c3836", overlay: "#504945", sidebar: "#282828", text: "#ebdbb2", secondary: "#d5c4a1", muted: "#a89984", border: "#665c54", accent: "#fe8019", success: "#b8bb26", danger: "#fb4934",
  } },
  { id: "gruvbox-dark-hard", family: "gruvbox", name: "Dark Hard", scheme: "dark", colors: {
    base: "#1d2021", surface: "#3c3836", overlay: "#504945", sidebar: "#1d2021", text: "#ebdbb2", secondary: "#d5c4a1", muted: "#a89984", border: "#665c54", accent: "#fe8019", success: "#b8bb26", danger: "#fb4934",
  } },
  { id: "gruvbox-light", family: "gruvbox", name: "Light", scheme: "light", colors: {
    base: "#fbf1c7", surface: "#f2e5bc", overlay: "#ebdbb2", sidebar: "#f2e5bc", text: "#3c3836", secondary: "#504945", muted: "#7c6f64", border: "#d5c4a1", accent: "#af3a03", success: "#79740e", danger: "#9d0006",
  } },
  // Classic Ayu colors requested by the user; upstream's newer release changes its base.
  { id: "ayu-dark", family: "ayu", name: "Dark", scheme: "dark", colors: {
    base: "#0b0e14", surface: "#0f131a", overlay: "#141821", sidebar: "#0b0e14", text: "#bfbdb6", secondary: "#9b9eaa", muted: "#565b66", border: "#1b1f29", accent: "#ffb454", success: "#aad94c", danger: "#f07178",
  } },
  { id: "ayu-mirage", family: "ayu", name: "Mirage", scheme: "dark", colors: {
    base: "#1f2430", surface: "#242936", overlay: "#282e3b", sidebar: "#1f2430", text: "#cccac2", secondary: "#b8c0cc", muted: "#707a8c", border: "#3d424d", accent: "#ffcc66", success: "#d5ff80", danger: "#f28779",
  } },
  { id: "ayu-light", family: "ayu", name: "Light", scheme: "light", colors: {
    base: "#fafafa", surface: "#ffffff", overlay: "#f3f4f5", sidebar: "#f3f4f5", text: "#5c6166", secondary: "#6c7680", muted: "#8a9199", border: "#d9dce0", accent: "#ff9940", success: "#86b300", danger: "#f07171",
  } },
  { id: "kanagawa-wave", family: "kanagawa", name: "Wave", scheme: "dark", colors: {
    base: "#1f1f28", surface: "#2a2a37", overlay: "#363646", sidebar: "#181820", text: "#dcd7ba", secondary: "#c8c093", muted: "#727169", border: "#54546d", accent: "#7e9cd8", success: "#98bb6c", danger: "#e46876",
  } },
  { id: "kanagawa-dragon", family: "kanagawa", name: "Dragon", scheme: "dark", colors: {
    base: "#181616", surface: "#282727", overlay: "#393836", sidebar: "#12120f", text: "#c5c9c5", secondary: "#a6a69c", muted: "#737c73", border: "#625e5a", accent: "#8ba4b0", success: "#87a987", danger: "#c4746e",
  } },
  { id: "kanagawa-lotus", family: "kanagawa", name: "Lotus", scheme: "light", colors: {
    base: "#f2ecbc", surface: "#e7dba0", overlay: "#e4d794", sidebar: "#dcd5ac", text: "#545464", secondary: "#43436c", muted: "#716e61", border: "#c9cbd1", accent: "#4d699b", success: "#6f894e", danger: "#c84053",
  } },
  { id: "vesper-dark", family: "vesper", name: "Dark", scheme: "dark", colors: {
    base: "#101010", surface: "#1c1c1c", overlay: "#282828", sidebar: "#101010", text: "#ffffff", secondary: "#a0a0a0", muted: "#8b8b8b", border: "#282828", accent: "#ffc799", success: "#99ffe4", danger: "#ff8080",
  } },
];

export const DEFAULT_PALETTE_ID = "catppuccin-mocha";

export function getPalette(id: string | null | undefined): ThemePalette | undefined {
  return THEME_PALETTES.find((p) => p.id === id);
}

export function contrastRatio(a: string, b: string): number {
  const luminance = (hex: string) => {
    const rgb = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
      .map((v) => v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
    return rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
  };
  const l1 = luminance(a), l2 = luminance(b);
  return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

// Keep upstream colors unless text needs a contrast adjustment on a player surface.
export function readableColor(color: string, backgrounds: string[], minimum = 4.5): string {
  const score = (candidate: string) => Math.min(...backgrounds.map((bg) => contrastRatio(candidate, bg)));
  if (score(color) >= minimum) return color;
  const target = score("#ffffff") > score("#000000") ? 255 : 0;
  for (let step = 1; step <= 100; step++) {
    const candidate = "#" + [1, 3, 5].map((i) => {
      const value = parseInt(color.slice(i, i + 2), 16);
      return Math.round(value + (target - value) * step / 100).toString(16).padStart(2, "0");
    }).join("");
    if (score(candidate) >= minimum) return candidate;
  }
  return target === 255 ? "#ffffff" : "#000000";
}
