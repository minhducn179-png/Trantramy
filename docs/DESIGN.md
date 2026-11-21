# dlc — Design System (`Impeccable` × `Taste-Skill`)

## 1. Aesthetic Direction & Taste-Skill Dials
- **Domain:** High-Craft Audiophile Desktop Music Player & Streaming Deck (inspired by Roon, Teenage Engineering, Apple Music Dark, and Linear).
- **Taste-Skill Calibration:**
  - `DESIGN_VARIANCE: 6/10` — Asymmetric split-pane studio deck with editorial header hierarchy, ambient album glow, and clean tabular tracklist.
  - `MOTION_INTENSITY: 6/10` — Hardware-accelerated `cubic-bezier(0.22, 1, 0.36, 1)` transitions, live 3-bar audio equalizer animation on active tracks, smooth teleprompter lyrics focus, and subtle hover micro-interactions.
  - `VISUAL_DENSITY: 7/10` — Desktop cockpit density: scannable track rows with tabular metadata, zero card-nesting bloat, and hover-revealed secondary actions.

## 2. Theme Palettes
The user selects a palette family and flavor. `src/lib/palettes.ts` contains the source links and semantic color mappings. The default is Catppuccin Mocha (`#1E1E2E` / `#CBA6F7`). Neutral grays and light variants are intentional parts of the selected palette, not replaced by a universal tint.

| Family | Flavors |
| :--- | :--- |
| Catppuccin | Latte, Frappé, Macchiato, Mocha |
| One Dark Pro | Classic, Darker, Flat, Vivid, Mix |
| Tokyo Night | Night, Storm, Moon, Day |
| Dracula | Classic, Soft |
| Rosé Pine | Main, Moon, Dawn |
| Nord | Polar Night, Snow Storm |
| Gruvbox | Dark Soft, Dark Medium, Dark Hard, Light |
| Ayu | Dark, Mirage, Light |
| Kanagawa | Wave, Dragon, Lotus |
| Vesper | Dark |

Canvas, sidebar, surfaces, borders, text, accent, success and error map to the selected palette. Text colors and light-theme accents adjust only when needed for contrast. Palette selection preserves layout, wallpaper, glass effects, playback settings and privacy preferences. Manual color edits switch to custom colors. Auto mode selects a light/dark flavor within the family when one exists.

One Dark Pro Vivid is a labeled DLC adaptation, not a separate upstream theme. Mix shares its upstream workbench colors with Classic. Dracula Soft follows the upstream bright-color desaturation. Nord Snow Storm is a light semantic mapping of Nord's palette. Ayu uses the classic colors requested by the user rather than silently adopting newer upstream backgrounds.

## 3. Typography & Numeric Discipline (`Impeccable Typeset`)
- **Display & UI Sans (`--font-sans`):** `'Plus Jakarta Sans', 'Segoe UI Variable Display', 'Inter', system-ui, sans-serif`
- **Technical & Timecode Mono (`--font-mono`):** `'JetBrains Mono', 'Cascadia Mono', 'Consolas', monospace`
- **Mandatory Numeric Rule:** All track indices (`01`, `02`), durations (`03:45`), seek timestamps, and download percentages enforce `font-variant-numeric: tabular-nums`.

## 4. Anti-Patterns Strictly Banned
- **Zero raw OS emojis** as UI controls. Use the locally bundled Lucide Svelte icons consistently.
- **Zero generic browser `<input type="range">` styling** — replaced with custom layered progress/volume scrubbers with smooth fill tracks.
- **Zero nested card-in-card clutter** — track lists use proximity, hairline dividers, and tabular alignment.
