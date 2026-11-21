<script lang="ts">
  import { CircleGauge, Palette, Image, AudioLines, PictureInPicture2, Music2, Archive, RotateCcw, Plus, SlidersVertical, ListMusic, Droplet, Moon, Sun, Monitor, Check, Copy, Sparkles, Waves, X, Circle, CircleCheck } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api, coverSrc } from "$lib/api";
  import { player } from "$lib/player.svelte";
  import {
    theme,
    BUILTIN_WALLPAPERS,
    oklchApproxToHex,
    hexToOklchApprox,
    type ColorScheme,
    type BgGradient,
    type CanvasMode,
    type WaveformStyle,
    type UiDensity,
    type RadiusStyle,
    type WinControlsMode,
    type BgImageFit,
    type BgImagePosition,
    type IslandWaveMode,
    type WaveformBounceMode,
  } from "$lib/theme.svelte";
  import WaveformBar from "./WaveformBar.svelte";
  import AppSpeakerLogo from "./AppSpeakerLogo.svelte";
  import ThemePicker from "./ThemePicker.svelte";
  import LyricsSettings from "./LyricsSettings.svelte";

  let {
    onopenequalizer,
    onopenspotify,
    onopenkaraoke,
    ontoggleisland,
    islandActive = false,
  }: {
    onopenequalizer?: () => void;
    onopenspotify?: () => void;
    onopenkaraoke?: () => void;
    ontoggleisland?: () => void;
    islandActive?: boolean;
  } = $props();

  let codeInput = $state("");
  let codeMsg = $state<{ text: string; ok: boolean } | null>(null);
  let copiedCode = $state<string | null>(null);

  // Save custom profile state (Name + Custom User Code)
  let newProfileName = $state("");
  let newProfileCustomCode = $state("");
  let demoPos = $state(42);

  type SettingsTab = "general" | "colors" | "wallpaper" | "player" | "island" | "audio" | "lyrics" | "profiles";
  const TABS: { id: SettingsTab; label: string; icon: typeof CircleGauge }[] = [
    { id: "general", label: "Tổng quan", icon: CircleGauge },
    { id: "colors", label: "Màu sắc", icon: Palette },
    { id: "wallpaper", label: "Hình nền", icon: Image },
    { id: "player", label: "Trình phát & Sóng nhạc", icon: AudioLines },
    { id: "island", label: "Dynamic Island", icon: PictureInPicture2 },
    { id: "audio", label: "Âm thanh", icon: Music2 },
    { id: "lyrics", label: "Lời bài hát & AI", icon: ListMusic },
    { id: "profiles", label: "Mã kiểu & Sao lưu", icon: Archive },
  ];
  let tab = $state<SettingsTab>(
    (typeof localStorage !== "undefined" &&
      (localStorage.getItem("dlc.settings.tab") as SettingsTab)) ||
      "general",
  );
  function selectTab(id: SettingsTab) {
    tab = id;
    try {
      localStorage.setItem("dlc.settings.tab", id);
    } catch {
      // ignore
    }
  }

  const HUE_SWATCHES = [
    { label: "Xanh biển (Ocean)", hue: 238, chroma: 0.17, light: 0.7 },
    { label: "Cam SoundCloud", hue: 42, chroma: 0.21, light: 0.69 },
    { label: "Xanh Spotify", hue: 152, chroma: 0.19, light: 0.75 },
    { label: "Băng tuyết (Cyan)", hue: 205, chroma: 0.15, light: 0.76 },
    { label: "Tím Indigo", hue: 272, chroma: 0.19, light: 0.7 },
    { label: "Hồng Cyberpunk", hue: 328, chroma: 0.23, light: 0.71 },
    { label: "Đỏ Crimson", hue: 22, chroma: 0.22, light: 0.66 },
    { label: "Vàng Hi-Fi", hue: 76, chroma: 0.16, light: 0.77 },
  ];

  const BG_SWATCHES: { label: string; mode: CanvasMode; h: number; c: number; l: number }[] = [
    { label: "Đen OLED (#000)", mode: "oled", h: 240, c: 0, l: 0.02 },
    { label: "Đen Biển Sâu", mode: "deep", h: 242, c: 0.012, l: 0.095 },
    { label: "Xám Than Phòng Thu", mode: "graphite", h: 240, c: 0.005, l: 0.11 },
    { label: "Xanh Đêm (Midnight)", mode: "custom", h: 255, c: 0.04, l: 0.11 },
    { label: "Tím Thẳm (Violet Noir)", mode: "custom", h: 295, c: 0.045, l: 0.11 },
    { label: "Đỏ Rượu (Crimson Dark)", mode: "custom", h: 18, c: 0.045, l: 0.11 },
    { label: "Rêu Rừng (Emerald Dark)", mode: "custom", h: 155, c: 0.038, l: 0.11 },
    { label: "Nâu Ấm (Warm Vinyl)", mode: "warm-noir", h: 60, c: 0.015, l: 0.1 },
  ];

  const WF_OPTIONS: { id: WaveformStyle; name: string; tag: string; desc: string }[] = [
    {
      id: "sc-fine",
      name: "SoundCloud mảnh nhiều thanh",
      tag: "SC FINE",
      desc: "Giống SoundCloud nhưng thanh mảnh như sợi tóc, hiển thị rất nhiều thanh",
    },
    {
      id: "mirrored",
      name: "Sóng đối xứng SoundCloud",
      tag: "SOUNDCLOUD",
      desc: "2 tầng sóng phản chiếu trên - dưới đặc trưng của SoundCloud",
    },
    {
      id: "line",
      name: "Thanh trượt tối giản",
      tag: "MINIMAL SLIDER",
      desc: "Thanh tiến trình ngang mượt mà kèm nút chấm tròn thanh lịch",
    },
    {
      id: "single",
      name: "Sóng đơn hiện đại",
      tag: "MODERN",
      desc: "Các cột sóng bo tròn hướng từ đáy lên gọn gàng",
    },
    {
      id: "center-sym",
      name: "Sóng đối xứng trung tâm",
      tag: "STUDIO DAW",
      desc: "Sóng tỏa đều cân xứng từ trục tâm ngang kiểu phòng thu",
    },
    {
      id: "led-matrix",
      name: "Đèn nháy phân đoạn LED",
      tag: "HI-FI VU",
      desc: "Cột sóng chia đốt đèn LED xếp tầng kèm đỉnh giữ nhịp",
    },
    {
      id: "neon-curve",
      name: "Sóng lượn mượt Analog",
      tag: "LIQUID WAVE",
      desc: "Đường cong sóng lượn mượt liền mạch kèm vùng phát sáng",
    },
    {
      id: "dual-neon",
      name: "Sóng kép giao thoa Stereo",
      tag: "DUAL WAVE",
      desc: "2 dải sóng lượn chồng lớp đan xen phát sáng theo nhịp nhạc",
    },
    {
      id: "dots-pulse",
      name: "Chuỗi hạt ánh sáng",
      tag: "DOT MATRIX",
      desc: "Các hạt ngọc phát sáng nhấp nhô theo biên độ giai điệu",
    },
    {
      id: "blocks-retro",
      name: "Khối Pixel bậc thang",
      tag: "8-BIT RETRO",
      desc: "Khối vuông sắc nét có đỉnh tách rời kiểu Synthwave",
    },
    {
      id: "flame-peak",
      name: "Sóng vòm cực quang",
      tag: "AURORA",
      desc: "Đỉnh sóng vòm phát sáng trắng-xanh rực rỡ kèm bóng phản quang",
    },
    {
      id: "bars-thick",
      name: "Cột sóng dày Sub-Bass",
      tag: "BASS BLOCK",
      desc: "Cột sóng bản rộng nổi khối mạnh mẽ",
    },
    {
      id: "minimal",
      name: "Sóng mảnh tối giản",
      tag: "MINIMAL",
      desc: "Dao động thấp, nhẹ nhàng và tinh gọn",
    },
  ];

  function selectWave(id: WaveformStyle) {
    if (id === "sc-fine" && theme.config.waveformBars < 140) {
      theme.update({ waveformStyle: id, waveformBars: 180 });
    } else {
      theme.update({ waveformStyle: id });
    }
  }

  const GRADIENT_OPTIONS: { id: BgGradient; label: string }[] = [
    { id: "flat", label: "Màu phẳng" },
    { id: "vertical", label: "Dọc" },
    { id: "diagonal", label: "Chéo" },
    { id: "radial", label: "Quầng sáng" },
  ];

  const GRADIENT_PRESETS: { label: string; h1: number; c1: number; l1: number; h2: number; mode: BgGradient; s: number }[] = [
    { label: "Aurora", h1: 255, c1: 0.035, l1: 0.1, h2: 310, mode: "radial", s: 0.7 },
    { label: "Hoàng hôn", h1: 20, c1: 0.03, l1: 0.1, h2: 335, mode: "diagonal", s: 0.65 },
    { label: "Đại dương", h1: 240, c1: 0.03, l1: 0.09, h2: 190, mode: "vertical", s: 0.6 },
    { label: "Rừng đêm", h1: 155, c1: 0.03, l1: 0.09, h2: 205, mode: "diagonal", s: 0.55 },
    { label: "Tím mộng", h1: 295, c1: 0.04, l1: 0.1, h2: 340, mode: "radial", s: 0.8 },
    { label: "Than hồng", h1: 25, c1: 0.015, l1: 0.08, h2: 45, mode: "vertical", s: 0.5 },
  ];

  const bg2Hex = $derived(
    oklchApproxToHex(0.3 + (theme.config.bgGradientStrength ?? 0.5) * 0.2, 0.12, theme.config.bgHue2 ?? 280),
  );

  const accentHex = $derived(
    theme.activePalette?.colors.accent ?? oklchApproxToHex(
      theme.config.accentLightness,
      theme.config.accentChroma,
      theme.config.accentHue,
    ),
  );

  const bgHex = $derived(
    theme.activePalette?.colors.base ?? oklchApproxToHex(
      theme.config.bgLightness ?? 0.095,
      theme.config.bgChroma ?? 0.012,
      theme.config.bgHue ?? 242,
    ),
  );

  async function copyText(txt: string, tag: string) {
    try {
      await navigator.clipboard.writeText(txt);
      copiedCode = tag;
      setTimeout(() => {
        if (copiedCode === tag) copiedCode = null;
      }, 1800);
    } catch {
      // fallback
    }
  }

  function handleApplyCode() {
    if (!codeInput.trim()) return;
    const raw = codeInput.trim();
    const ok = theme.applyCode(raw);
    if (ok) {
      codeMsg = { text: `Đã áp dụng mã giao diện: ${raw.toUpperCase()}`, ok: true };
      codeInput = "";
    } else {
      codeMsg = {
        text: "Mã không tồn tại. Hãy nhập mã cài đặt sẵn (VD: DLC-SC-BLUE), mã bạn đã lưu, hoặc mã DLC-...",
        ok: false,
      };
    }
    setTimeout(() => {
      codeMsg = null;
    }, 3500);
  }

  function handleSaveProfile() {
    const saved = theme.saveProfile(newProfileName, newProfileCustomCode);
    newProfileName = "";
    newProfileCustomCode = "";
    codeMsg = {
      text: `Đã lưu phong cách "${saved.name}" với mã code: ${saved.code}`,
      ok: true,
    };
    setTimeout(() => {
      codeMsg = null;
    }, 3500);
  }

  // Discord-style Background Image state & handlers
  let bgUrlInput = $state("");
  let bgUploadBusy = $state(false);
  let bgNotice = $state<{ text: string; ok: boolean } | null>(null);
  let bgDropActive = $state(false);

  function showBgNotice(text: string, ok = true) {
    bgNotice = { text, ok };
    setTimeout(() => {
      if (bgNotice?.text === text) bgNotice = null;
    }, 3500);
  }

  async function pickLocalBgImage() {
    try {
      const f = await open({
        multiple: false,
        filters: [
          {
            name: "Ảnh & GIF phông nền",
            extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg"],
          },
        ],
      });
      if (typeof f !== "string") return;
      bgUploadBusy = true;
      const savedPath = await api.saveBackgroundImage(f);
      const rawName = f.split(/[/\\]/).pop()?.replace(/\.[^.]+$/, "") || "Ảnh nền từ máy";
      await theme.setCustomBgImage(savedPath, rawName, coverSrc(savedPath));
      showBgNotice(`Đã áp dụng phông nền: ${rawName}`);
    } catch (e) {
      showBgNotice(`Không thể tải ảnh: ${e}`, false);
    } finally {
      bgUploadBusy = false;
    }
  }

  async function applyBgUrlInput() {
    const url = bgUrlInput.trim();
    if (!url) return;
    bgUploadBusy = true;
    try {
      const savedPath = await api.saveBackgroundImage(url);
      await theme.setCustomBgImage(savedPath, "Ảnh từ liên kết", coverSrc(savedPath));
      bgUrlInput = "";
      showBgNotice("Đã áp dụng ảnh phông nền từ liên kết!");
    } catch {
      await theme.setCustomBgImage(url, "Ảnh từ liên kết", url);
      bgUrlInput = "";
      showBgNotice("Đã áp dụng ảnh phông nền từ liên kết!");
    } finally {
      bgUploadBusy = false;
    }
  }

  async function handleBgFileDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    bgDropActive = false;
    const dt = e.dataTransfer;
    if (!dt) return;
    if (dt.files && dt.files.length > 0) {
      const file = dt.files[0];
      bgUploadBusy = true;
      try {
        const ext = file.name.split(".").pop() || "png";
        const buf = await file.arrayBuffer();
        const savedPath = await api.saveBackgroundBytes(ext, Array.from(new Uint8Array(buf)));
        const cleanName = file.name.replace(/\.[^.]+$/, "");
        await theme.setCustomBgImage(savedPath, cleanName, coverSrc(savedPath));
        showBgNotice(`Đã áp dụng phông nền: ${cleanName}`);
      } catch (err) {
        showBgNotice(`Không thể lưu ảnh: ${err}`, false);
      } finally {
        bgUploadBusy = false;
      }
      return;
    }
    const textUrl = dt.getData("text/uri-list") || dt.getData("text/plain");
    if (textUrl && (textUrl.startsWith("http://") || textUrl.startsWith("https://"))) {
      bgUrlInput = textUrl.trim();
      await applyBgUrlInput();
    }
  }

  async function handleDeleteSavedWallpaper(id: string) {
    const removedSrc = theme.removeSavedWallpaper(id);
    if (removedSrc && !removedSrc.startsWith("http") && !removedSrc.startsWith("data:")) {
      try {
        await api.deleteBackgroundImage(removedSrc);
      } catch {
        // ignore
      }
    }
  }

  async function syncColorFromActiveWallpaper() {
    if (!theme.config.bgImage) return;
    const ok = await theme.syncColorsFromBgImage(coverSrc(theme.config.bgImage));
    if (ok) {
      showBgNotice("Đã đồng bộ màu chủ đạo & sắc độ nền theo ảnh hiện tại!");
    } else {
      showBgNotice("Đã giữ nguyên bảng màu hiện tại.", true);
    }
  }
</script>

<div class="settings-shell">
  <nav class="settings-nav" aria-label="Danh mục cài đặt">
    <div class="settings-nav-title">Cài đặt</div>
    {#each TABS as t (t.id)}
      <button
        type="button"
        class="settings-tab"
        class:active={tab === t.id}
        onclick={() => selectTab(t.id)}
      >
        <span class="tab-ico"><t.icon size={16} /></span>
        <span>{t.label}</span>
      </button>
    {/each}
  </nav>

<div class="settings-stage">
  {#if tab === "lyrics"}<LyricsSettings />{/if}
  {#if tab === "general"}
  <!-- 1. Quick Config Code & Instant State Switcher -->
  <section class="code-hub-card">
    <div class="hub-top">
      <div class="hub-brand-preview">
        <AppSpeakerLogo size={74} />
        <div>
          <span class="kicker">BIỂU TƯỢNG & MÃ GIAO DIỆN</span>
          <h3>Biểu tượng loa & Trạng thái giao diện đồng bộ</h3>
          <p class="sub">
            Biểu tượng loa trên thanh bên, thanh tác vụ và khay hệ thống tự động đổi <strong>màu nền và nét biểu tượng</strong> theo bộ màu bạn chọn.
          </p>
        </div>
      </div>

      <button class="reset-btn" onclick={() => theme.resetDefault()} title="Khôi phục mặc định">
        <RotateCcw size={14} />
        <span>Về mặc định</span>
      </button>
    </div>

    <div class="code-bars-grid">
      <!-- Current Active Code -->
      <div class="active-code-box">
        <span class="box-label">Mã giao diện hiện tại</span>
        <div class="code-row">
          <input
            type="text"
            class="live-code"
            readonly
            value={theme.currentCode}
            title={theme.currentCode}
            aria-label="Mã cấu hình hiện tại"
            onclick={(e) => e.currentTarget.select()}
          />
          <button
            type="button"
            class="copy-btn"
            onclick={() => copyText(theme.currentCode, "current")}
          >
            {#if copiedCode === "current"}<Check size={14} /> Đã chép{:else}Sao chép mã{/if}
          </button>
        </div>
      </div>

      <!-- Paste & Apply Code -->
      <form
        class="apply-code-box"
        onsubmit={(e) => {
          e.preventDefault();
          handleApplyCode();
        }}
      >
        <span class="box-label">Nhập mã để đổi giao diện</span>
        <div class="code-row">
          <input
            type="text"
            class="code-input"
            placeholder="Mã tự đặt (VD: MY-VIP) hoặc DLC-..."
            bind:value={codeInput}
          />
          <button type="submit" class="apply-btn" disabled={!codeInput.trim()}>
            Áp dụng
          </button>
        </div>
      </form>
    </div>

    {#if codeMsg}
      <div class="code-feedback" class:err={!codeMsg.ok}>
        {codeMsg.text}
      </div>
    {/if}

    <ThemePicker />
  </section>

  <!-- Layout, Scale & Controls Card -->
  <section class="cfg-card radius-customizer-card">
    <div class="card-header">
      <div>
        <span class="kicker">BỐ CỤC & TÙY BIẾN HỆ THỐNG</span>
        <h3>Độ bo góc, cỡ chữ & điều khiển cửa sổ</h3>
      </div>
      <span class="radius-live-badge">{Math.round(theme.config.cornerRadius ?? 16)} px</span>
    </div>

    <div class="radius-customizer-grid">
      <div class="radius-controls-col">
        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ bo góc khung (0px Vuông → 28px Tròn)</span>
            <span class="ctrl-val">{Math.round(theme.config.cornerRadius ?? 16)} px</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0"
            max="28"
            step="1"
            value={theme.config.cornerRadius ?? 16}
            oninput={(e) => theme.update({ cornerRadius: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <span class="ctrl-label">Chọn nhanh mức độ bo góc:</span>
          <div class="seg-group">
            {#each [
              { val: 0, label: "Vuông (0px)" },
              { val: 6, label: "Nhẹ (6px)" },
              { val: 12, label: "Vừa (12px)" },
              { val: 16, label: "Chuẩn (16px)" },
              { val: 24, label: "Siêu tròn (24px)" },
            ] as rp}
              <button
                type="button"
                class:active={Math.abs((theme.config.cornerRadius ?? 16) - rp.val) <= 1}
                onclick={() => theme.update({ cornerRadius: rp.val })}
              >
                {rp.label}
              </button>
            {/each}
          </div>
        </div>
      </div>

      <!-- Live Mini Frame Preview -->
      <div class="radius-live-preview" aria-label="Xem trước độ bo tròn khung">
        <div class="rp-tag-bar">
          <span class="rp-chip active">Tất cả</span>
          <span class="rp-chip">Yêu thích</span>
          <span class="rp-chip">Khám phá</span>
        </div>
        <div class="rp-cards-row">
          <div class="rp-song-card">
            <div class="rp-thumb"></div>
            <div class="rp-lines">
              <span class="rp-l1"></span>
              <span class="rp-l2"></span>
            </div>
          </div>
          <div class="rp-info-box">
            <span class="rp-kicker">KHUNG THÔNG TIN</span>
            <strong>Bo góc: {Math.round(theme.config.cornerRadius ?? 16)}px</strong>
            <small>Đồng bộ toàn ứng dụng</small>
          </div>
        </div>
      </div>
    </div>

    <div class="control-block">
      <span class="ctrl-label">Mật độ bố cục giao diện</span>
      <div class="seg-group">
        {#each [
          { id: "compact", label: "Gọn gàng" },
          { id: "comfortable", label: "Tiêu chuẩn" },
          { id: "spacious", label: "Rộng rãi" },
        ] as dn}
          <button
            type="button"
            class:active={theme.config.density === dn.id}
            onclick={() => theme.update({ density: dn.id as UiDensity })}
          >
            {dn.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="control-block">
      <span class="ctrl-label">Cỡ chữ tổng thể</span>
      <div class="seg-group">
        {#each [
          { val: 0.92, label: "Nhỏ (92%)" },
          { val: 1, label: "Chuẩn (100%)" },
          { val: 1.08, label: "Lớn (108%)" },
        ] as fs}
          <button
            type="button"
            class:active={Math.abs(theme.config.fontScale - fs.val) < 0.02}
            onclick={() => theme.update({ fontScale: fs.val })}
          >
            {fs.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="control-block">
      <span class="ctrl-label">Nút điều khiển cửa sổ (Thu nhỏ / Phóng to / Đóng)</span>
      <div class="seg-group">
        {#each [
          { id: "auto-hide", label: "Tự ẩn khi rời chuột" },
          { id: "always", label: "Luôn hiển thị" },
        ] as wc}
          <button
            type="button"
            class:active={theme.config.winControlsMode === wc.id}
            onclick={() => theme.update({ winControlsMode: wc.id as WinControlsMode })}
          >
            {wc.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="toggle-row">
      <div>
        <strong>Chế độ ẩn danh (Stealth Mode)</strong>
        <small>Ngăn các phần mềm quay màn hình, OBS, Discord, Overwolf nhận diện cửa sổ và Dynamic Island</small>
      </div>
      <button
        type="button"
        class="switch-btn"
        class:on={theme.config.stealthMode ?? true}
        aria-label="Bật tắt chế độ ẩn danh"
        aria-pressed={theme.config.stealthMode ?? true}
        onclick={() => theme.update({ stealthMode: !(theme.config.stealthMode ?? true) })}
      >
        <span></span>
      </button>
    </div>

    <!-- Ngôn ngữ thiết kế Kính lỏng & Hiệu ứng Giọt nước (Liquid Glass & Droplet Design) -->
    <div class="droplet-custom-card" style="margin-top: 14px; padding: 16px; border-radius: var(--ui-radius, 16px); background: linear-gradient(135deg, rgba(255, 255, 255, 0.05) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-surface-2); border: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 14px;">
      <div>
        <strong style="display: flex; align-items: center; gap: 8px; font-size: 14px;">
          <Sparkles size={16} />
          Thiết kế Giọt Nước & Kính Lỏng (Liquid Glass & Droplet UI)
        </strong>
        <small style="color: var(--text-muted); font-size: 11.5px;">Tùy chỉnh phong cách Kính lỏng Apple, hiệu ứng nổi khối giọt nước 3D và gợn sóng tương tác theo sở thích</small>
      </div>

      <!-- Item 1: Apple Liquid Glass Style -->
      <div class="toggle-row" style="background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.12); border-radius: 12px; padding: 10px 14px;">
        <div>
          <strong style="display: flex; align-items: center; gap: 6px;">
            <Sparkles size={14} />
            Phong cách Apple Liquid Glass (Kính lỏng Apple)
          </strong>
          <small>Giao diện kính lỏng liền mạch xuyên suốt, không chia khúc, thanh phát nhạc êm ái, góc Squircle mềm và hiệu ứng co giãn đàn hồi</small>
        </div>
        <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
          <button
            type="button"
            class="apply-btn"
            onclick={() => {
              theme.update({ appleGlassMode: true, waterDropletMode: false, cornerRadius: 24 });
              const wp = BUILTIN_WALLPAPERS.find(w => w.id === "wp-apple-liquid");
              if (wp) theme.update({ bgImage: wp.src, bgImageDim: 0.35, bgImageBlur: 4, waterRippleEffect: false });
            }}
            title="Bật vật liệu kính và hình nền Apple, giữ bộ màu đang chọn"
          >
            Áp dụng kính & nền
          </button>
          <button
            type="button"
            class="switch-btn"
            class:on={theme.config.appleGlassMode ?? false}
            aria-label="Bật tắt phong cách Apple Liquid Glass"
            onclick={() => theme.update({ appleGlassMode: !(theme.config.appleGlassMode ?? false) })}
          >
            <span></span>
          </button>
        </div>
      </div>

      <!-- Item 2: 3D Water Droplet Surface -->
      <div class="toggle-row" style="background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(0, 220, 255, 0.2); border-radius: 12px; padding: 10px 14px;">
        <div>
          <strong style="display: flex; align-items: center; gap: 6px;">
            <Droplet size={14} />
            Bề mặt Giọt nước 3D (3D Liquid Droplets)
          </strong>
          <small>Tạo bề mặt thủy tinh 3D căng bóng như giọt sương, phủ bóng mọng nước và phản quang nổi khối (không tạo gợn sóng)</small>
        </div>
        <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
          <button
            type="button"
            class="apply-btn"
            onclick={() => {
              theme.update({ waterDropletMode: true, appleGlassMode: false, cornerRadius: 22 });
              const wp = BUILTIN_WALLPAPERS.find(w => w.id === "wp-aqua-droplets");
              if (wp) theme.update({ bgImage: wp.src, bgImageDim: 0.4, bgImageBlur: 0, waterRippleEffect: false });
            }}
            title="Bật hiệu ứng giọt nước và hình nền, giữ bộ màu đang chọn"
          >
            Áp dụng trọn bộ
          </button>
          <button
            type="button"
            class="switch-btn"
            class:on={theme.config.waterDropletMode ?? false}
            aria-label="Bật tắt giao diện bề mặt giọt nước"
            onclick={() => theme.update({ waterDropletMode: !(theme.config.waterDropletMode ?? false) })}
          >
            <span></span>
          </button>
        </div>
      </div>

      <!-- Item 3: Interactive Water Ripple Click -->
      <div class="toggle-row" style="background: rgba(255, 255, 255, 0.03); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 10px 14px;">
        <div>
          <strong style="display: flex; align-items: center; gap: 6px;">
            <Waves size={14} />
            Hiệu ứng gợn sóng khi nhấp chuột (Water Ripple Click)
          </strong>
          <small>Lan tỏa vòng sóng nước mở rộng tự nhiên và phản quang khi click chuột vào bất kỳ đâu trên màn hình</small>
        </div>
        <button
          type="button"
          class="switch-btn"
          class:on={theme.config.waterRippleEffect ?? false}
          aria-label="Bật tắt hiệu ứng gợn sóng khi nhấp chuột"
          onclick={() => theme.update({ waterRippleEffect: !(theme.config.waterRippleEffect ?? false) })}
        >
          <span></span>
        </button>
      </div>
    </div>
  </section>
  {/if}

  {#if tab === "island"}
  <section class="cfg-card">
    <div class="card-header">
      <div>
        <span class="kicker">DYNAMIC ISLAND</span>
        <h3>Cửa sổ nổi Mini Player</h3>
      </div>
    </div>

    <div class="toggle-row">
      <div>
        <strong>Chế độ ẩn danh (Stealth Mode)</strong>
        <small>Ngăn các phần mềm quay màn hình, OBS, Discord, Overwolf nhận diện cửa sổ và Dynamic Island</small>
      </div>
      <button
        type="button"
        class="switch-btn"
        class:on={theme.config.stealthMode ?? true}
        aria-label="Bật tắt chế độ ẩn danh"
        aria-pressed={theme.config.stealthMode ?? true}
        onclick={() => theme.update({ stealthMode: !(theme.config.stealthMode ?? true) })}
      >
        <span></span>
      </button>
    </div>

    <div class="toggle-row">
      <div>
        <strong>Tự động hiển thị khi thu nhỏ ứng dụng</strong>
        <small>Hiển thị thanh Dynamic Island nổi khi bạn thu nhỏ hoặc đóng cửa sổ chính</small>
      </div>
      <button
        type="button"
        class="switch-btn"
        class:on={theme.config.autoIslandOnMinimize ?? true}
        aria-label="Tự động bật Dynamic Island khi thu nhỏ"
        onclick={() =>
          theme.update({ autoIslandOnMinimize: !(theme.config.autoIslandOnMinimize ?? true) })}
      >
        <span></span>
      </button>
    </div>

    <div class="toggle-row">
      <div>
        <strong>Mở nhanh Dynamic Island &amp; Lời bài hát</strong>
        <small>Bật cửa sổ nổi mini player hoặc mở phòng thu lời bài hát đồng bộ</small>
      </div>
      <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
        <button
          type="button"
          class="reset-btn"
          class:active={islandActive}
          style={islandActive ? "background: var(--accent-soft); border-color: var(--accent); color: var(--accent); font-weight: 700;" : ""}
          onclick={() => (ontoggleisland ? ontoggleisland() : api.toggleIsland(undefined, theme.config.stealthMode))}
        >
          {#if islandActive}<CircleCheck size={14} /> Đang bật Island{:else}<Circle size={14} /> Bật Island{/if}
        </button>
        {#if onopenkaraoke}
          <button
            type="button"
            class="reset-btn"
            onclick={onopenkaraoke}
          >
            Trang lời bài hát
          </button>
        {/if}
      </div>
    </div>

    <div class="control-block">
      <span class="ctrl-label">Chuyển động sóng trên Dynamic Island:</span>
      <div class="seg-group">
        {#each [
          { id: "progress-static", label: "Sóng tĩnh · Sáng dần" },
          { id: "bounce-progress", label: "Nảy nhịp điệu + Sáng dần" },
          { id: "bounce-real", label: "Chỉ nảy nhịp điệu" },
          { id: "scroll-wave", label: "Sóng cuộn thời gian thực" },
        ] as iwm}
          <button
            type="button"
            class:active={(theme.config.islandWaveMode ?? "bounce-progress") === iwm.id}
            onclick={() => theme.update({ islandWaveMode: iwm.id as IslandWaveMode, waveformBounce: (iwm.id === "progress-static" ? "none" : "real-audio") as WaveformBounceMode })}
          >
            {iwm.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="control-block">
      <span class="ctrl-label">Tùy biến diện mạo Dynamic Island</span>
      <label class="toggle-row" style="padding: 6px 0;">
        <div>
          <strong>Đồng bộ màu theo giao diện ứng dụng</strong>
          <small>Tự động phối màu nền, chữ và màu nhấn theo bảng màu của ứng dụng</small>
        </div>
        <input type="checkbox" checked={theme.config.islandThemeSync ?? true} onchange={(e) => theme.update({ islandThemeSync: e.currentTarget.checked })} />
      </label>

      {#if !(theme.config.islandThemeSync ?? true)}
        <div class="seg-group" style="flex-wrap:wrap;gap:12px;align-items:center;margin: 8px 0;">
          <label style="display:flex;align-items:center;gap:6px;font-size:12px">Màu nền
            <input type="color" value={theme.config.islandBg ?? "#000000"} oninput={(e) => theme.update({ islandBg: e.currentTarget.value })} />
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:12px">Màu nhấn
            <input type="color" value={theme.config.islandAccent || "#4da3ff"} oninput={(e) => theme.update({ islandAccent: e.currentTarget.value })} />
          </label>
          <label style="display:flex;align-items:center;gap:6px;font-size:12px">Màu chữ
            <input type="color" value={theme.config.islandText ?? "#ffffff"} oninput={(e) => theme.update({ islandText: e.currentTarget.value })} />
          </label>
          <button type="button" onclick={() => theme.update({ islandAccent: "" })} class:active={!theme.config.islandAccent}>Theo màu nhấn app</button>
        </div>
      {/if}

      <div class="slider-head" style="margin-top: 10px;"><span class="ctrl-label">Độ trong suốt</span><span class="ctrl-val">{Math.round((theme.config.islandOpacity ?? 1) * 100)}%</span></div>
      <input type="range" class="std-slider" min="0.4" max="1" step="0.02" value={theme.config.islandOpacity ?? 1} oninput={(e) => theme.update({ islandOpacity: +e.currentTarget.value })} />

      <div class="slider-head"><span class="ctrl-label">Độ bo góc</span><span class="ctrl-val">{theme.config.islandRadius ?? 38}px</span></div>
      <input type="range" class="std-slider" min="8" max="44" step="1" value={theme.config.islandRadius ?? 38} oninput={(e) => theme.update({ islandRadius: +e.currentTarget.value })} />

      <div class="seg-group" style="margin-top: 8px;">
        <button type="button" class:active={(theme.config.islandArtShape ?? "square") === "square"} onclick={() => theme.update({ islandArtShape: "square" })}>Ảnh bìa vuông bo góc</button>
        <button type="button" class:active={theme.config.islandArtShape === "circle"} onclick={() => theme.update({ islandArtShape: "circle" })}>Ảnh bìa tròn</button>
      </div>

      <button type="button" class="reset-btn" onclick={() => theme.update({ islandBg: "#000000", islandAccent: "", islandText: "#ffffff", islandOpacity: 1, islandRadius: 38, islandArtShape: "square", islandThemeSync: true })} style="margin-top:10px;font-size:12px;padding:5px 14px;border-radius:8px;border:1px solid var(--border);background:transparent;color:var(--text-secondary);cursor:pointer">Đặt lại mặc định</button>
    </div>
  </section>
  {/if}

  {#if tab === "profiles"}
  <!-- 2. Save Custom Style with User-Defined Code -->
  <section class="cfg-card">
    <div class="card-header">
      <div>
        <span class="kicker">MÃ GIAO DIỆN & SAO LƯU</span>
        <h3>Lưu phong cách cá nhân và tạo mã chia sẻ</h3>
      </div>
    </div>

    <form
      class="custom-code-creator"
      onsubmit={(e) => {
        e.preventDefault();
        handleSaveProfile();
      }}
    >
      <div class="field-col">
        <label for="prof-name">Tên phong cách</label>
        <input
          id="prof-name"
          type="text"
          placeholder="VD: Không gian Chill đêm…"
          bind:value={newProfileName}
        />
      </div>

      <div class="field-col">
        <label for="prof-code">Mã định danh (Tùy chọn)</label>
        <input
          id="prof-code"
          type="text"
          class="mono-input"
          placeholder="VD: MY-CHILL, VIP-01 (Để trống để tự tạo)"
          bind:value={newProfileCustomCode}
        />
      </div>

      <button type="submit" class="save-custom-btn">
        <Plus size={15} />
        <span>Lưu phong cách</span>
      </button>
    </form>

    {#if theme.savedProfiles.length === 0}
      <p class="empty-profiles">
        Chưa có phong cách nào được lưu. Bạn có thể phối màu nền, màu nhấn, thanh sóng nhạc rồi đặt <strong>Tên & Mã riêng</strong> ở trên để lưu lại và chuyển đổi nhanh bất cứ lúc nào!
      </p>
    {:else}
      <div class="saved-grid">
        {#each theme.savedProfiles as prof (prof.id)}
          <div class="saved-item">
            <div class="saved-meta">
              <div class="saved-title-row">
                <span
                  class="dual-swatch"
                  style="--bg-s: oklch({prof.config.bgLightness ?? 0.095} {prof.config.bgChroma ?? 0.012} {prof.config.bgHue ?? 242}); --ac-s: oklch({prof.config.accentLightness} {prof.config.accentChroma} {prof.config.accentHue})"
                ></span>
                <strong>{prof.name}</strong>
                <small>{prof.createdAt}</small>
              </div>
              <div class="saved-code-row">
                <span class="code-badge">CODE: {prof.code}</span>
              </div>
            </div>

            <div class="saved-actions">
              <button
                class="s-act primary"
                onclick={() => theme.applyCode(prof.code)}
                title="Áp dụng phong cách này"
              >
                Áp dụng
              </button>
              <button
                class="s-act"
                onclick={() => copyText(prof.code, prof.id)}
                title="Sao chép mã {prof.code}"
              >
                {#if copiedCode === prof.id}<Check size={12} />{:else}Copy mã{/if}
              </button>
              <button
                class="s-act"
                onclick={() => copyText(prof.autoCode || prof.code, prof.id + "-full")}
                title="Sao chép mã đầy đủ để chia sẻ"
              >
                {#if copiedCode === prof.id + "-full"}<Check size={12} />{:else}Mã đầy đủ{/if}
              </button>
              <button
                class="s-act danger"
                onclick={() => theme.deleteProfile(prof.id)}
                title="Xóa trạng thái này"
              >
                <X size={14} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
  {/if}

  {#if tab === "audio"}
  <!-- Audio Engine & Integration Card -->
  <section class="cfg-card audio-hub-card">
    <div class="card-header">
      <div>
        <span class="kicker">ÂM THANH & TÍCH HỢP NGUỒN PHÁT</span>
        <h3>Bộ xử lý âm thanh chuyên nghiệp & Nhập Spotify</h3>
      </div>
    </div>
    <div class="audio-hub-body">
      <div class="audio-tool-box">
        <div class="tool-meta">
          <div class="tool-icon eq-icon">
            <SlidersVertical size={20} />
          </div>
          <div>
            <strong>Equalizer 10 Băng Tần & Chuẩn hóa âm lượng</strong>
            <p>Tùy biến tần số âm thanh, tăng âm trầm (Bass), làm sáng giọng hát (Vocal) và chống vỡ tiếng tự động.</p>
          </div>
        </div>
        <button type="button" class="tool-btn" onclick={onopenequalizer}>
          Mở Equalizer
        </button>
      </div>

      <div class="audio-tool-box">
        <div class="tool-meta">
          <div class="tool-icon sp-icon">
            <ListMusic size={20} />
          </div>
          <div>
            <strong>Nhập Playlist từ Spotify</strong>
            <p>Dán link bất kỳ từ Spotify để tự động đồng bộ hóa danh sách phát và ghép nguồn âm thanh chất lượng cao.</p>
          </div>
        </div>
        <button type="button" class="tool-btn sp-btn" onclick={onopenspotify}>
          Nhập từ Spotify
        </button>
      </div>
    </div>
  </section>
  {/if}

  {#if tab === "wallpaper"}
  <!-- Discord-Style Custom Background Wallpaper & Glassmorphism Card -->
  <section class="cfg-card discord-bg-card">
    <div class="card-header">
      <div>
        <span class="kicker">PHÔNG NỀN ẢNH & HIỆU ỨNG KÍNH MỜ · PHONG CÁCH DISCORD</span>
        <h3>Tùy biến Ảnh nền / GIF & Lớp kính trong suốt</h3>
      </div>

      <div class="bg-header-actions">
        <button
          type="button"
          class="bg-upload-primary-btn"
          onclick={pickLocalBgImage}
          disabled={bgUploadBusy}
        >
          <Image size={15} />
          <span>{bgUploadBusy ? "Đang tải ảnh…" : "+ Thêm ảnh từ máy"}</span>
        </button>

        {#if theme.config.bgImage}
          <button
            type="button"
            class="bg-action-pill"
            onclick={syncColorFromActiveWallpaper}
            title="Tự động trích xuất tông màu nổi bật từ ảnh nền để đổi màu loa, sóng nhạc và nút bấm"
          >
            <Droplet size={14} />
            <span>Đồng bộ màu theo ảnh</span>
          </button>

          <button
            type="button"
            class="bg-action-pill danger"
            onclick={() => theme.setCustomBgImage(null)}
            title="Tắt ảnh phông nền, quay về nền màu tiêu chuẩn"
          >
            <X size={14} /><span>Gỡ ảnh nền</span>
          </button>
        {/if}
      </div>
    </div>

    <!-- Top Row: Live Banner Dropzone + URL Input & Quick Galleries -->
    <div class="discord-bg-top-grid">
      <!-- Left: Interactive Preview & Drag-and-Drop Banner -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="bg-preview-dropzone"
        class:drop-active={bgDropActive}
        class:has-img={Boolean(theme.config.bgImage)}
        ondragenter={(e) => {
          e.preventDefault();
          e.stopPropagation();
          bgDropActive = true;
        }}
        ondragover={(e) => {
          e.preventDefault();
          e.stopPropagation();
          bgDropActive = true;
        }}
        ondragleave={(e) => {
          e.preventDefault();
          e.stopPropagation();
          bgDropActive = false;
        }}
        ondrop={handleBgFileDrop}
      >
        {#if theme.config.bgImage}
          <img
            class="bgp-img"
            src={coverSrc(theme.config.bgImage)}
            alt="Phông nền hiện tại"
            style="object-fit: {theme.config.bgImageFit || 'cover'}; object-position: {theme.config.bgImagePosition || 'center'}; filter: blur({Math.round((theme.config.bgImageBlur ?? 0) * 0.45)}px) brightness({theme.config.bgImageBrightness ?? 1}) saturate({theme.config.bgImageSaturation ?? 1.08});"
          />
          <div
            class="bgp-dim"
            style="opacity: {theme.config.bgImageDim ?? 0.48};"
          ></div>
          <div class="bgp-mock-ui">
            <div class="bgp-mock-sidebar" style="opacity: {theme.config.bgImageSurfaceAlpha ?? 0.54}">
              <AppSpeakerLogo size={26} />
              <div class="bgp-lines">
                <span></span>
                <span></span>
              </div>
            </div>
            <div class="bgp-mock-stage">
              <div class="bgp-mock-card" style="opacity: {theme.config.bgImageSurfaceAlpha ?? 0.54}">
                <span class="bgp-badge">ĐANG BẬT PHÔNG NỀN DISCORD</span>
                <strong>Giao diện kính mờ (Frosted Glass)</strong>
                <small>Kéo thả ảnh khác vào đây hoặc bấm để đổi ảnh</small>
              </div>
            </div>
          </div>
          <button
            type="button"
            class="bgp-change-btn"
            onclick={pickLocalBgImage}
          >
            Đổi ảnh khác
          </button>
        {:else}
          <div class="bgp-empty" onclick={pickLocalBgImage} onkeydown={(e) => e.key === "Enter" && pickLocalBgImage()} role="button" tabindex="0">
            <div class="bgp-empty-icon">
              <Image size={26} />
            </div>
            <strong>Bấm để chọn ảnh từ máy hoặc kéo thả ảnh / GIF vào đây</strong>
            <small>Hỗ trợ PNG, JPG, WEBP, GIF động, AVIF — Tự động làm trong suốt các bảng giao diện kiểu Discord</small>
          </div>
        {/if}
      </div>

      <!-- Right: Paste Image URL + Built-in & User Saved Galleries -->
      <div class="bg-gallery-col">
        <form
          class="bg-url-form"
          onsubmit={(e) => {
            e.preventDefault();
            applyBgUrlInput();
          }}
        >
          <input
            type="text"
            class="bg-url-input"
            placeholder="Dán link ảnh hoặc GIF trực tuyến (https://...)"
            bind:value={bgUrlInput}
          />
          <button
            type="submit"
            class="bg-url-btn"
            disabled={!bgUrlInput.trim() || bgUploadBusy}
          >
            Áp dụng link
          </button>
        </form>

        {#if bgNotice}
          <div class="code-feedback" class:err={!bgNotice.ok}>
            {bgNotice.text}
          </div>
        {/if}

        <!-- Built-in Discord-Style Wallpapers -->
        <div class="wp-section">
          <div class="wp-sec-head">
            <span class="ctrl-label">Phông nền nghệ thuật dựng sẵn (1-Click):</span>
          </div>
          <div class="wp-thumbs-grid">
            {#each BUILTIN_WALLPAPERS as bwp (bwp.id)}
              <button
                type="button"
                class="wp-thumb-card"
                class:active={theme.config.bgImage === bwp.src}
                onclick={() => theme.setCustomBgImage(bwp.src, bwp.name, bwp.src)}
                title="Áp dụng phông nền {bwp.name}"
              >
                <img src={bwp.src} alt={bwp.name} />
                <div class="wp-thumb-meta">
                  <strong>{bwp.name}</strong>
                  <span>{bwp.tag}</span>
                </div>
              </button>
            {/each}
          </div>
        </div>

        <!-- User's Uploaded Wallpapers Gallery -->
        {#if theme.savedWallpapers.length > 0}
          <div class="wp-section">
            <div class="wp-sec-head">
              <span class="ctrl-label">Bộ sưu tập ảnh bạn đã thêm ({theme.savedWallpapers.length}):</span>
            </div>
            <div class="wp-thumbs-grid user-wps">
              {#each theme.savedWallpapers as uwp (uwp.id)}
                <div
                  class="wp-thumb-card user-card"
                  class:active={theme.config.bgImage === uwp.src}
                >
                  <button
                    type="button"
                    class="wp-user-select"
                    onclick={() => theme.setCustomBgImage(uwp.src, uwp.name, coverSrc(uwp.src))}
                    title="Sử dụng ảnh {uwp.name}"
                  >
                    <img src={coverSrc(uwp.src)} alt={uwp.name} />
                    <div class="wp-thumb-meta">
                      <strong>{uwp.name}</strong>
                      <span>{uwp.addedAt}</span>
                    </div>
                  </button>
                  <button
                    type="button"
                    class="wp-user-del"
                    onclick={() => handleDeleteSavedWallpaper(uwp.id)}
                    title="Xóa ảnh này khỏi bộ sưu tập"
                    aria-label="Xóa ảnh {uwp.name}"
                  >
                    <X size={14} />
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Bottom Row: Discord Glassmorphism & Wallpaper Controls -->
    <div class="discord-bg-controls-grid" class:ctrl-disabled={!theme.config.bgImage}>
      <div class="control-block">
        <div class="slider-head">
          <span class="ctrl-label">Độ đục kính giao diện (Glass Card Opacity)</span>
          <span class="ctrl-val">{Math.round((theme.config.bgImageSurfaceAlpha ?? 0.54) * 100)}%</span>
        </div>
        <input
          type="range"
          class="std-slider"
          min="0.14"
          max="0.94"
          step="0.02"
          value={theme.config.bgImageSurfaceAlpha ?? 0.54}
          oninput={(e) => theme.update({ bgImageSurfaceAlpha: +e.currentTarget.value })}
        />
        <small class="ctrl-sub-hint">Giảm xuống để các bảng, danh sách nhạc và thanh bên trong suốt hơn</small>
      </div>

      <div class="control-block">
        <div class="slider-head">
          <span class="ctrl-label">Độ tối lớp phủ nền (Background Dim / Tint)</span>
          <span class="ctrl-val">{Math.round((theme.config.bgImageDim ?? 0.48) * 100)}%</span>
        </div>
        <input
          type="range"
          class="std-slider"
          min="0.08"
          max="0.9"
          step="0.02"
          value={theme.config.bgImageDim ?? 0.48}
          oninput={(e) => theme.update({ bgImageDim: +e.currentTarget.value })}
        />
        <small class="ctrl-sub-hint">Chỉnh độ phủ màu tối lên ảnh để chữ luôn sắc nét, dễ đọc</small>
      </div>

      <div class="control-block">
        <div class="slider-head">
          <span class="ctrl-label">Độ nhòe ảnh nền (Backdrop Blur)</span>
          <span class="ctrl-val">{Math.round(theme.config.bgImageBlur ?? 0)} px</span>
        </div>
        <input
          type="range"
          class="std-slider"
          min="0"
          max="28"
          step="1"
          value={theme.config.bgImageBlur ?? 0}
          oninput={(e) => theme.update({ bgImageBlur: +e.currentTarget.value })}
        />
      </div>

      <div class="control-block">
        <div class="slider-head">
          <span class="ctrl-label">Độ sáng & Độ rực màu ảnh (Brightness / Saturation)</span>
          <span class="ctrl-val">
            {Math.round((theme.config.bgImageBrightness ?? 1) * 100)}% · {Math.round((theme.config.bgImageSaturation ?? 1.08) * 100)}%
          </span>
        </div>
        <div class="dual-sliders">
          <input
            type="range"
            class="std-slider"
            title="Độ sáng ảnh nền"
            min="0.4"
            max="1.4"
            step="0.05"
            value={theme.config.bgImageBrightness ?? 1}
            oninput={(e) => theme.update({ bgImageBrightness: +e.currentTarget.value })}
          />
          <input
            type="range"
            class="std-slider"
            title="Độ tươi màu ảnh nền"
            min="0.4"
            max="1.8"
            step="0.05"
            value={theme.config.bgImageSaturation ?? 1.08}
            oninput={(e) => theme.update({ bgImageSaturation: +e.currentTarget.value })}
          />
        </div>
      </div>

      <div class="control-block">
        <span class="ctrl-label">Kiểu co giãn ảnh nền</span>
        <div class="seg-group">
          {#each [
            { id: "cover", label: "Phủ kín" },
            { id: "contain", label: "Vừa khung" },
            { id: "fill", label: "Kéo đầy" },
          ] as fit}
            <button
              type="button"
              class:active={(theme.config.bgImageFit || "cover") === fit.id}
              onclick={() => theme.update({ bgImageFit: fit.id as BgImageFit })}
            >
              {fit.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="control-block">
        <span class="ctrl-label">Vị trí trọng tâm ảnh</span>
        <div class="seg-group">
          {#each [
            { id: "center", label: "Chính giữa" },
            { id: "top", label: "Phía trên" },
            { id: "bottom", label: "Phía dưới" },
          ] as pos}
            <button
              type="button"
              class:active={(theme.config.bgImagePosition || "center") === pos.id}
              onclick={() => theme.update({ bgImagePosition: pos.id as BgImagePosition })}
            >
              {pos.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="toggle-row">
      <div>
        <strong>Tự động đồng bộ tông màu giao diện khi chọn ảnh nền mới</strong>
        <small>Tự trích xuất màu chủ đạo của bức ảnh để phối màu cho chiếc loa, sóng nhạc và viền sáng</small>
      </div>
      <button
        type="button"
        class="switch-btn"
        class:on={theme.config.bgImageAutoTheme}
        aria-label="Tự động đồng bộ tông màu giao diện theo ảnh nền"
        onclick={() => theme.update({ bgImageAutoTheme: !theme.config.bgImageAutoTheme })}
      >
        <span></span>
      </button>
    </div>
  </section>
  {/if}

  <!-- 3. Deep Customization Grid: Free Background Color + Free Accent Color + Waveform / Progress Bar -->
  <div class="tab-single">
    <!-- Left Column: Free Background Color & Accent Color -->
    {#if tab === "colors"}
    <section class="cfg-card">
      <ThemePicker />
    </section>
    <section class="cfg-card">
      <div class="card-header">
        <div>
          <span class="kicker">MÀU SẮC NỀN & MÀU CHỦ ĐẠO</span>
          <h3>Tùy chỉnh màu nền & màu nhấn</h3>
        </div>
      </div>

            <!-- Light / Dark / Auto Mode Control -->
      <div class="scheme-toggle-box">
        <div class="scheme-head">
          <div class="scheme-title-group">
            <span class="scheme-tag">CHẾ ĐỘ HIỂN THỊ</span>
            <strong class="scheme-label">Màu nền Sáng / Tối</strong>
            <small class="scheme-hint">Sáng / Tối chọn biến thể cùng bộ nếu có, nếu không sẽ tạo màu tùy chỉnh. Tự động theo Windows với bộ có cả hai tông.</small>
          </div>
          <div class="scheme-seg">
            <button
              type="button"
              class="scheme-btn"
              class:active={theme.config.colorScheme === "dark"}
              onclick={() => theme.setColorScheme("dark")}
              title="Chế độ nền Tối (Dark mode)"
            >
              <Moon size={15} />
              <span>Tối</span>
            </button>
            <button
              type="button"
              class="scheme-btn"
              class:active={theme.config.colorScheme === "light"}
              onclick={() => theme.setColorScheme("light")}
              title="Chế độ nền Sáng (Light mode)"
            >
              <Sun size={15} />
              <span>Sáng</span>
            </button>
            <button
              type="button"
              class="scheme-btn"
              class:active={theme.config.colorScheme === "auto"}
              onclick={() => theme.setColorScheme("auto")}
              title="Tự động theo hệ thống Windows"
            >
              <Monitor size={15} />
              <span>Tự động</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Free Background Color Section -->
      <div class="color-group-box">
        <div class="cg-head">
          <div>
            <strong>1. Màu sắc toàn bộ Nền (Background Color)</strong>
            <small>Chọn màu hoặc kéo thanh trượt sẽ chuyển sang bảng màu tùy chỉnh</small>
          </div>
          <label class="color-picker-pill" title="Bấm để mở bảng chọn màu nền tự do">
            <input
              type="color"
              value={bgHex}
              oninput={(e) => theme.setCustomBackgroundHex(e.currentTarget.value)}
            />
            <span class="cp-dot" style="background: {bgHex}"></span>
            <span>Chọn màu nền</span>
          </label>
        </div>

        <div class="swatch-row">
          {#each BG_SWATCHES as bsw}
            <button
              type="button"
              class="color-swatch bg-sw"
              class:active={
                theme.config.canvasMode === bsw.mode &&
                (bsw.mode !== "custom" || Math.abs((theme.config.bgHue ?? 0) - bsw.h) <= 8)
              }
              style="--sw-color: oklch({bsw.l} {bsw.c} {bsw.h})"
              title={bsw.label}
              onclick={() =>
                theme.update({
                  canvasMode: bsw.mode,
                  bgHue: bsw.h,
                  bgChroma: bsw.c,
                  bgLightness: bsw.l,
                })}
            >
              <span></span>
            </button>
          {/each}
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Tông màu nền (Background Hue)</span>
            <span class="ctrl-val">{Math.round(theme.config.bgHue ?? 242)}°</span>
          </div>
          <input
            type="range"
            class="hue-slider"
            min="0"
            max="360"
            step="1"
            value={theme.config.bgHue ?? 242}
            oninput={(e) =>
              theme.update({
                canvasMode: "custom",
                bgHue: +e.currentTarget.value,
                bgChroma: Math.max(0.02, theme.config.bgChroma ?? 0.025),
              })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ đậm màu nền (Background Tint)</span>
            <span class="ctrl-val">{Math.round((theme.config.bgChroma ?? 0.012) * 1000) / 10}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0"
            max="0.14"
            step="0.005"
            value={theme.config.bgChroma ?? 0.012}
            oninput={(e) =>
              theme.update({
                canvasMode: "custom",
                bgChroma: +e.currentTarget.value,
              })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ sáng / tối của nền (Background Lightness)</span>
            <span class="ctrl-val">{Math.round((theme.config.bgLightness ?? 0.095) * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.01"
            max="0.96"
            step="0.005"
            value={theme.config.bgLightness ?? 0.095}
            oninput={(e) => {
              const val = +e.currentTarget.value;
              theme.update({
                canvasMode: "custom",
                colorScheme: val >= 0.52 ? "light" : "dark",
                bgLightness: val,
              });
            }}
          />
        </div>
      </div>

      <!-- Advanced Background: gradient, panels, sidebar, text, borders -->
      <div class="color-group-box">
        <div class="cg-head">
          <div>
            <strong>Phông nền nâng cao (Gradient & Lớp giao diện)</strong>
            <small>Phối 2 màu nền, chỉnh độ tách lớp, thanh bên, màu chữ và viền</small>
          </div>
        </div>

        <div class="control-block">
          <span class="ctrl-label">Mẫu phông nền gradient dựng sẵn</span>
          <div class="grad-presets">
            {#each GRADIENT_PRESETS as gp}
              <button
                type="button"
                class="grad-chip"
                title={gp.label}
                style="--g1: oklch({gp.l1} {gp.c1} {gp.h1}); --g2: oklch({gp.l1 + 0.1} 0.1 {gp.h2})"
                onclick={() =>
                  theme.update({
                    canvasMode: "custom",
                    bgHue: gp.h1,
                    bgChroma: gp.c1,
                    bgLightness: gp.l1,
                    bgHue2: gp.h2,
                    bgGradient: gp.mode,
                    bgGradientStrength: gp.s,
                  })}
              >
                <span class="grad-swatch"></span>
                <span>{gp.label}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="control-block">
          <span class="ctrl-label">Kiểu phối màu nền</span>
          <div class="seg-group">
            {#each GRADIENT_OPTIONS as go}
              <button
                type="button"
                class:active={theme.config.bgGradient === go.id}
                onclick={() => theme.update({ bgGradient: go.id })}
              >
                {go.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="control-block" class:ctrl-disabled={theme.config.bgGradient === "flat"}>
          <div class="slider-head">
            <span class="ctrl-label">Màu thứ 2 của phông nền (Gradient Hue)</span>
            <label class="color-picker-pill small" title="Chọn màu thứ 2 của gradient">
              <input
                type="color"
                value={bg2Hex}
                oninput={(e) => theme.update({ bgHue2: hexToOklchApprox(e.currentTarget.value).h })}
              />
              <span class="cp-dot" style="background: {bg2Hex}"></span>
              <span>{Math.round(theme.config.bgHue2)}°</span>
            </label>
          </div>
          <input
            type="range"
            class="hue-slider"
            min="0"
            max="360"
            step="1"
            disabled={theme.config.bgGradient === "flat"}
            value={theme.config.bgHue2}
            oninput={(e) => theme.update({ bgHue2: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block" class:ctrl-disabled={theme.config.bgGradient === "flat"}>
          <div class="slider-head">
            <span class="ctrl-label">Độ đậm của gradient</span>
            <span class="ctrl-val">{Math.round(theme.config.bgGradientStrength * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0"
            max="1"
            step="0.05"
            disabled={theme.config.bgGradient === "flat"}
            value={theme.config.bgGradientStrength}
            oninput={(e) => theme.update({ bgGradientStrength: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ tách lớp thẻ / bảng (Surface Contrast)</span>
            <span class="ctrl-val">{Math.round(theme.config.surfaceContrast * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.4"
            max="2.2"
            step="0.05"
            value={theme.config.surfaceContrast}
            oninput={(e) => theme.update({ surfaceContrast: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <span class="ctrl-label">Nền thanh bên trái</span>
          <div class="seg-group">
            {#each [
              { id: "darker", label: "Tối hơn" },
              { id: "same", label: "Cùng phông nền" },
              { id: "lighter", label: "Sáng hơn" },
            ] as sh}
              <button
                type="button"
                class:active={theme.config.sidebarShade === sh.id}
                onclick={() => theme.update({ sidebarShade: sh.id as "darker" | "same" | "lighter" })}
              >
                {sh.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ sáng chữ (Text Brightness)</span>
            <span class="ctrl-val">{Math.round(theme.config.textLightness * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.78"
            max="1"
            step="0.005"
            value={theme.config.textLightness}
            oninput={(e) => theme.update({ textLightness: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ rõ của đường viền (Border)</span>
            <span class="ctrl-val">{Math.round(theme.config.borderStrength * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.3"
            max="2"
            step="0.05"
            value={theme.config.borderStrength}
            oninput={(e) => theme.update({ borderStrength: +e.currentTarget.value })}
          />
        </div>
      </div>

      <!-- Free Accent Color Section -->
      <div class="color-group-box">
        <div class="cg-head">
          <div>
            <strong>2. Màu nhấn chủ đạo (Accent & Sóng nhạc)</strong>
            <small>Đổi màu nút phát, thanh tiến trình, sóng nhạc và viền sáng</small>
          </div>
          <label class="color-picker-pill" title="Bấm để mở bảng chọn màu nhấn tự do">
            <input
              type="color"
              value={accentHex}
              oninput={(e) => theme.setCustomAccentHex(e.currentTarget.value)}
            />
            <span class="cp-dot" style="background: {accentHex}"></span>
            <span>Chọn màu nhấn</span>
          </label>
        </div>

        <div class="swatch-row">
          {#each HUE_SWATCHES as sw}
            <button
              type="button"
              class="color-swatch"
              class:active={Math.abs(theme.config.accentHue - sw.hue) <= 4}
              style="--sw-color: oklch({sw.light} {sw.chroma} {sw.hue})"
              title={sw.label}
              onclick={() =>
                theme.update({
                  accentHue: sw.hue,
                  accentChroma: sw.chroma,
                  accentLightness: sw.light,
                })}
            >
              <span></span>
            </button>
          {/each}
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Sắc độ màu nhấn (Accent Hue)</span>
            <span class="ctrl-val">{Math.round(theme.config.accentHue)}°</span>
          </div>
          <input
            type="range"
            class="hue-slider"
            min="0"
            max="360"
            step="1"
            value={theme.config.accentHue}
            oninput={(e) => theme.update({ accentHue: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ rực màu nhấn (Chroma)</span>
            <span class="ctrl-val">{Math.round(theme.config.accentChroma * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.06"
            max="0.25"
            step="0.01"
            value={theme.config.accentChroma}
            oninput={(e) => theme.update({ accentChroma: +e.currentTarget.value })}
          />
        </div>

        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Độ sáng màu nhấn (Lightness)</span>
            <span class="ctrl-val">{Math.round(theme.config.accentLightness * 100)}%</span>
          </div>
          <input
            type="range"
            class="std-slider"
            min="0.52"
            max="0.82"
            step="0.01"
            value={theme.config.accentLightness}
            oninput={(e) => theme.update({ accentLightness: +e.currentTarget.value })}
          />
        </div>
      </div>

      <div class="toggle-row">
        <div>
          <strong>Hiệu ứng phát sáng nền (Ambient Glow)</strong>
          <small>Tạo quầng sáng dịu theo tông màu chủ đạo</small>
        </div>
        <button
          type="button"
          class="switch-btn"
          class:on={theme.config.ambientGlow}
          aria-label="Bật tắt hiệu ứng phát sáng nền"
          onclick={() => theme.update({ ambientGlow: !theme.config.ambientGlow })}
        >
          <span></span>
        </button>
      </div>
    </section>
    {/if}

    <!-- Right Column: Waveform / Progress Bar & Playback -->
    {#if tab === "player"}
    <section class="cfg-card">
      <div class="card-header">
        <div>
          <span class="kicker">TRÌNH PHÁT & SÓNG NHẠC</span>
          <h3>Kiểu dáng thanh tiến trình & chuyển động sóng nhạc</h3>
        </div>
      </div>

      <div class="toggle-row">
        <div>
          <strong>Tự động phát bài liên quan khi hết danh sách (Autoplay Radio)</strong>
          <small>Khi phát xong bài cuối cùng, ứng dụng sẽ tự động tìm kiếm và phát các bài hát có phong cách tương tự theo bài đang phát và lịch sử nghe nhạc của bạn.</small>
        </div>
        <button
          type="button"
          class="switch-btn"
          class:on={player.autoRelated}
          aria-label="Tự động phát bài liên quan khi hết danh sách"
          onclick={() => player.toggleAutoRelated()}
        >
          <span></span>
        </button>
      </div>

      <!-- Live Waveform / Line Bar Preview -->
      <div class="wf-preview-box">
        <div class="wfp-head">
          <span>XEM TRƯỚC TRỰC TIẾP (BẤM HOẶC KÉO ĐỂ THỬ)</span>
          <code>
            {theme.config.waveformStyle === "line"
              ? "THANH TRƯỢT TỐI GIẢN"
              : `${theme.config.waveformStyle.toUpperCase()} · ${theme.config.waveformBars} CỘT`}
          </code>
        </div>
        <WaveformBar
          seed="dlc-customizer-preview"
          position={demoPos}
          duration={100}
          playing={true}
          height={46}
          onseek={(s) => (demoPos = s)}
        />
      </div>

      <div class="control-block">
        <span class="ctrl-label">Bộ sưu tập 13 kiểu Sóng nhạc & Thanh thời lượng</span>
        <div class="wf-grid-options">
          {#each WF_OPTIONS as wf}
            <button
              type="button"
              class="wf-opt-card"
              class:active={theme.config.waveformStyle === wf.id}
              onclick={() => selectWave(wf.id)}
            >
              <div class="wfo-top">
                <strong>{wf.name}</strong>
                <span class="wfo-tag">{wf.tag}</span>
              </div>
              <div class="wfo-preview" aria-hidden="true">
                <WaveformBar
                  seed="wfo-{wf.id}"
                  styleOverride={wf.id}
                  barsCount={wf.id === "sc-fine" ? 70 : 28}
                  position={55}
                  duration={100}
                  height={26}
                />
              </div>
              <small>{wf.desc}</small>
            </button>
          {/each}
        </div>
      </div>

      <div class="control-block">
        <span class="ctrl-label">Chế độ chuyển động của sóng nhạc (WaveformBar)</span>
        <div class="seg-group">
          {#each [
            { id: "none", label: "Sóng tĩnh · Sáng dần theo thời gian" },
            { id: "real-audio", label: "Nảy nhịp điệu + Sáng dần" },
            { id: "playhead", label: "Nảy nhẹ tại vạch phát" },
          ] as wbm}
            <button
              type="button"
              class:active={(theme.config.waveformBounce ?? "none") === wbm.id}
              onclick={() => theme.update({ waveformBounce: wbm.id as WaveformBounceMode, islandWaveMode: (wbm.id === "none" ? "progress-static" : "bounce-progress") as IslandWaveMode })}
            >
              {wbm.label}
            </button>
          {/each}
        </div>
      </div>

      <div class="control-block" class:ctrl-disabled={theme.config.waveformStyle === "line"}>
        <div class="slider-head">
          <span class="ctrl-label">Số lượng cột sóng nhạc</span>
          <span class="ctrl-val">
            {theme.config.waveformStyle === "line" ? "—" : `${theme.config.waveformBars} sóng`}
          </span>
        </div>
        <input
          type="range"
          class="std-slider"
          min="24"
          max="220"
          step="2"
          disabled={theme.config.waveformStyle === "line"}
          value={theme.config.waveformBars}
          oninput={(e) => theme.update({ waveformBars: +e.currentTarget.value })}
        />
        <div class="seg-group">
          {#each [36, 68, 96, 128, 160, 200] as count}
            <button
              type="button"
              disabled={theme.config.waveformStyle === "line"}
              class:active={theme.config.waveformBars === count}
              onclick={() => theme.update({ waveformBars: count })}
            >
              {count}
            </button>
          {/each}
        </div>
      </div>
    </section>
    {/if}
  </div>
</div>
</div>

<style>
  .settings-stage {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px 4px 28px;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
  }

  .kicker {
    font-family: var(--font-sans);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--accent);
    display: block;
  }

  /* 1. Code Hub Card */
  .code-hub-card {
    background: linear-gradient(135deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 14px);
    padding: clamp(14px, 1.8vw, 20px);
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 14px 34px oklch(0 0 0 / 0.45);
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }
  .hub-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    min-width: 0;
  }
  .hub-top > div {
    flex: 1 1 260px;
    min-width: 0;
  }
  .hub-top h3 {
    margin: 4px 0 4px;
    font-size: clamp(15.5px, 1.7vw, 18px);
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .sub {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.45;
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    flex: none;
    white-space: nowrap;
    outline: none !important;
    transition: all 0.15s ease;
  }
  .reset-btn:hover {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
    transform: translateY(-1px);
  }
  .reset-btn:active {
    transform: translateY(0);
  }

  .code-bars-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 12px;
    min-width: 0;
    width: 100%;
  }
  .active-code-box,
  .apply-code-box {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }
  .box-label {
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .code-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    width: 100%;
  }
  .live-code {
    flex: 1 1 0%;
    width: 0;
    min-width: 0;
    height: 32px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 700;
    color: var(--accent);
    background: var(--bg-surface-1);
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    outline: none;
    box-sizing: border-box;
  }
  .code-input {
    flex: 1 1 0%;
    width: 0;
    min-width: 0;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11.5px;
    outline: none;
    box-sizing: border-box;
  }
  .code-input:focus {
    border-color: var(--accent);
  }

  .copy-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    flex: none;
    white-space: nowrap;
    outline: none !important;
    transition: all 0.15s ease;
  }
  .copy-btn:hover {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
    transform: translateY(-1px);
  }
  .copy-btn:active {
    transform: translateY(0);
  }

  .apply-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    flex: none;
    white-space: nowrap;
    outline: none !important;
    box-shadow: none !important;
    transition: all 0.15s ease;
  }
  .apply-btn:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }
  .apply-btn:active:not(:disabled) {
    transform: translateY(0);
  }
  .apply-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    transform: none;
  }

  .code-feedback {
    padding: 8px 12px;
    border-radius: 8px;
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    color: var(--accent-hover);
    font-size: 12px;
    font-weight: 600;
  }
  .code-feedback.err {
    background: oklch(0.67 0.19 25 / 0.15);
    border-color: var(--danger);
    color: var(--danger);
  }

  /* Generic Config Card */
  .cfg-card {
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: var(--ui-radius, 14px);
    padding: clamp(14px, 1.8vw, 18px);
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-subtle);
  }
  .card-header h3 {
    margin: 3px 0 0;
    font-size: 16px;
    font-weight: 700;
  }

  /* Audio Hub Card */
  .audio-hub-card {
    background: linear-gradient(135deg, var(--bg-surface-1) 0%, var(--accent-soft) 100%);
  }
  .audio-hub-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .audio-tool-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 14px;
    border-radius: var(--ui-radius, 12px);
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
  }
  .tool-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .tool-icon {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .tool-icon.eq-icon,
  .tool-icon.sp-icon {
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px solid var(--accent);
  }
  .tool-meta strong {
    display: block;
    font-size: 13px;
    color: var(--text-primary);
  }
  .tool-meta p {
    margin: 2px 0 0;
    font-size: 11px;
    color: var(--text-muted);
  }
  .tool-btn,
  .tool-btn.sp-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 14px;
    border-radius: 8px;
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    border: 1px solid transparent;
    cursor: pointer;
    white-space: nowrap;
    box-shadow: none !important;
    transition: all 0.15s ease;
  }
  .tool-btn:hover,
  .tool-btn.sp-btn:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }
  .tool-btn:active,
  .tool-btn.sp-btn:active {
    transform: translateY(0);
  }

  /* Custom Code Creator Form */
  .custom-code-creator {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.1fr) auto;
    gap: 10px;
    align-items: end;
    background: var(--bg-canvas);
    padding: 12px 14px;
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
  }
  .field-col {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .field-col label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .field-col input {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    outline: none;
  }
  .field-col input.mono-input {
    font-family: var(--font-mono);
    text-transform: uppercase;
  }
  .field-col input:focus {
    border-color: var(--accent);
  }
  .save-custom-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    box-shadow: none !important;
    transition: all 0.15s ease;
  }
  .save-custom-btn:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }
  .save-custom-btn:active {
    transform: translateY(0);
  }

  .empty-profiles {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-secondary);
  }
  .saved-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
    gap: 10px;
  }
  .saved-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 11px 13px;
    border-radius: 10px;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
  }
  .saved-meta {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .saved-title-row {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
  }
  .dual-swatch {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--bg-s);
    border: 3px solid var(--ac-s);
    flex: none;
  }
  .saved-title-row small {
    color: var(--text-muted);
    font-size: 10.5px;
  }
  .saved-code-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .code-badge {
    font-family: var(--font-mono);
    font-size: 10.5px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 2px 7px;
    border-radius: 5px;
  }
  .saved-actions {
    display: flex;
    align-items: center;
    gap: 5px;
    flex: none;
  }
  .s-act {
    height: 26px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .s-act:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
  .s-act.primary {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
  }
  .s-act.danger:hover {
    border-color: var(--danger);
    color: var(--danger);
  }

  /* 3. Two-Column Deep Controls */
  .scheme-toggle-box {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    padding: 12px 14px;
    margin-bottom: 14px;
    box-shadow: 0 2px 8px oklch(0 0 0 / 0.15);
  }
  .scheme-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .scheme-title-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .scheme-tag {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent);
    text-transform: uppercase;
  }
  .scheme-label {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
  }
  .scheme-hint {
    font-size: 11px;
    color: var(--text-muted);
  }
  .scheme-seg {
    display: inline-flex;
    align-items: center;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 3px;
    gap: 4px;
  }
  .scheme-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 13px;
    border-radius: 7px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .scheme-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-3);
  }
  .scheme-btn.active {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-weight: 700;
    box-shadow: 0 2px 8px var(--accent-glow);
  }
  .scheme-btn :global(svg) {
    flex: none;
  }

  .tab-single {
    min-width: 0;
    width: 100%;
  }

  .color-group-box {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    padding: 12px 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    box-sizing: border-box;
  }
  .cg-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-wrap: wrap;
  }
  .cg-head strong {
    display: block;
    font-size: 13px;
  }
  .cg-head small {
    font-size: 11px;
    color: var(--text-muted);
  }

  .color-picker-pill {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    transition: border-color 0.15s ease;
  }
  .color-picker-pill:hover {
    border-color: var(--accent);
  }
  .color-picker-pill input[type="color"] {
    position: absolute;
    inset: 0;
    opacity: 0;
    width: 100%;
    height: 100%;
    cursor: pointer;
  }
  .cp-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 1px solid oklch(1 0 0 / 0.35);
  }

  .control-block {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .ctrl-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }
  .slider-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .ctrl-val {
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--accent);
    font-weight: 700;
  }

  .swatch-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .color-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--bg-surface-2);
    padding: 3px;
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }
  .color-swatch.bg-sw span {
    border: 1px solid oklch(1 0 0 / 0.2);
  }
  .color-swatch span {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: var(--sw-color);
  }
  .color-swatch.active {
    border-color: var(--accent);
    transform: scale(1.1);
  }

  .hue-slider,
  .std-slider {
    width: 100%;
    height: 6px;
    border-radius: 999px;
    appearance: none;
    outline: none !important;
    border: none !important;
    cursor: pointer;
  }
  .hue-slider {
    background: linear-gradient(
      to right,
      oklch(0.7 0.18 0),
      oklch(0.7 0.18 60),
      oklch(0.7 0.18 120),
      oklch(0.7 0.18 180),
      oklch(0.7 0.18 240),
      oklch(0.7 0.18 300),
      oklch(0.7 0.18 360)
    );
  }
  .std-slider {
    background: var(--bg-surface-3);
    accent-color: var(--accent);
  }
  .std-slider::-webkit-slider-thumb,
  .hue-slider::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent);
    border: none !important;
    outline: none !important;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4) !important;
    cursor: pointer;
    transition: transform 0.15s ease;
  }
  .hue-slider::-webkit-slider-thumb {
    background: #ffffff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.6) !important;
  }
  .std-slider::-webkit-slider-thumb:hover,
  .hue-slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }
  .std-slider:focus,
  .std-slider:focus-visible,
  .hue-slider:focus,
  .hue-slider:focus-visible {
    outline: none !important;
  }

  .wf-grid-options {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 8px;
    min-width: 0;
  }
  .wf-opt-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 4px;
    padding: 9px 10px;
    border-radius: 9px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
    color: var(--text-primary);
    font-family: var(--font-sans);
    text-align: left;
    cursor: pointer;
    min-width: 0;
    box-sizing: border-box;
    overflow: hidden;
    transition: all 0.15s ease;
  }
  .wf-opt-card:hover {
    border-color: var(--border-strong);
    background: var(--bg-surface-2);
  }
  .wf-opt-card.active {
    border-color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .wfo-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }
  .wfo-top strong {
    font-size: 12px;
    font-weight: 700;
  }
  .wfo-tag {
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 700;
    padding: 1.5px 5px;
    border-radius: 4px;
    background: var(--bg-surface-2);
    color: var(--accent);
    flex: none;
  }
  .wf-opt-card small {
    font-size: 10.5px;
    color: var(--text-muted);
    line-height: 1.35;
  }

  .seg-group {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: minmax(0, 1fr);
    gap: 4px;
    background: var(--bg-canvas);
    padding: 4px;
    border-radius: 9px;
    border: 1px solid var(--border-subtle);
    min-width: 0;
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
  }
  .seg-group button {
    height: 30px;
    padding: 0 6px;
    min-width: 0;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .seg-group button.active {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-weight: 700;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 8px;
    border-top: 1px solid var(--border-subtle);
  }
  .toggle-row strong {
    display: block;
    font-size: 12.5px;
  }
  .toggle-row small {
    font-size: 11px;
    color: var(--text-muted);
  }
  .switch-btn {
    width: 42px;
    height: 24px;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--bg-canvas);
    padding: 2px;
    cursor: pointer;
    flex: none;
    transition: all 0.18s ease;
  }
  .switch-btn span {
    display: block;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--text-secondary);
    transition: transform 0.18s ease, background 0.18s ease;
  }
  .switch-btn.on {
    background: var(--accent-soft);
    border-color: var(--accent);
  }
  .switch-btn.on span {
    transform: translateX(18px);
    background: var(--accent);
  }

  .wf-preview-box {
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    height: 100px;
    flex: none;
    justify-content: space-between;
    overflow: hidden;
  }
  .wfp-head code {
    max-width: 55%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ctrl-disabled {
    opacity: 0.42;
    pointer-events: none;
  }
  .wfo-preview {
    height: 26px;
    pointer-events: none;
    overflow: hidden;
    border-radius: 5px;
    background: var(--bg-surface-1);
    padding: 0 4px;
  }
  .wf-opt-card {
    min-height: 106px;
  }
  .color-picker-pill.small {
    height: 26px;
    padding: 0 8px;
    font-size: 11px;
  }
  .grad-presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 7px;
  }
  .grad-chip {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 6px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .grad-chip:hover {
    border-color: var(--accent);
    color: var(--text-primary);
  }
  .grad-swatch {
    display: block;
    height: 26px;
    border-radius: 5px;
    background: linear-gradient(135deg, var(--g1), var(--g2));
    border: 1px solid oklch(1 0 0 / 0.12);
  }
  .hue-slider:disabled,
  .std-slider:disabled {
    cursor: default;
  }
  .seg-group button:disabled {
    cursor: default;
  }
  .wfp-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: var(--font-mono);
    font-size: 9.5px;
    color: var(--text-muted);
  }
  .wfp-head code {
    color: var(--accent);
  }

  .hub-brand-preview {
    display: flex;
    align-items: center;
    gap: 18px;
    min-width: 0;
  }

  /* Discord-style Custom Wallpaper Card */
  .discord-bg-card {
    border-color: var(--border-strong);
    box-shadow: 0 10px 28px oklch(0 0 0 / 0.35);
  }
  .bg-header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .bg-upload-primary-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    height: 32px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: none !important;
    transition: transform 0.14s ease, background 0.14s ease;
  }
  .bg-upload-primary-btn:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }
  .bg-upload-primary-btn:disabled {
    opacity: 0.55;
    cursor: default;
  }
  .bg-action-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .bg-action-pill:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
    transform: translateY(-1px);
  }
  .bg-action-pill.danger:hover {
    border-color: var(--danger);
    color: var(--danger);
    background: oklch(0.67 0.19 25 / 0.12);
  }

  .discord-bg-top-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 14px;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
  }

  .bg-preview-dropzone {
    position: relative;
    width: 100%;
    max-width: 100%;
    height: 164px;
    min-height: 164px;
    max-height: 164px;
    border-radius: 12px;
    border: 2px dashed var(--border-strong);
    background: var(--bg-canvas);
    overflow: hidden;
    contain: paint;
    isolation: isolate;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: border-color 0.18s ease, box-shadow 0.18s ease;
  }
  .bg-preview-dropzone.has-img {
    border-style: solid;
    border-color: var(--accent);
  }
  .bg-preview-dropzone.drop-active {
    border-style: dashed;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .bgp-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    display: block;
    pointer-events: none;
  }
  .bgp-dim {
    position: absolute;
    inset: 0;
    background: #070a10;
    pointer-events: none;
  }
  .bgp-mock-ui {
    position: relative;
    z-index: 2;
    width: calc(100% - 24px);
    height: calc(100% - 24px);
    max-height: 140px;
    padding-right: 108px;
    box-sizing: border-box;
    display: grid;
    grid-template-columns: 60px minmax(0, 1fr);
    gap: 10px;
    pointer-events: none;
  }
  .bgp-mock-sidebar {
    background: oklch(0.11 0.015 242);
    backdrop-filter: blur(8px);
    border: 1px solid oklch(1 0 0 / 0.14);
    border-radius: 9px;
    padding: 8px 6px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    overflow: hidden;
  }
  .bgp-lines {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .bgp-lines span {
    display: block;
    height: 5px;
    border-radius: 999px;
    background: oklch(1 0 0 / 0.22);
  }
  .bgp-mock-stage {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    overflow: hidden;
  }
  .bgp-mock-card {
    width: 100%;
    min-width: 0;
    padding: 10px 12px;
    border-radius: 10px;
    background: oklch(0.13 0.018 242);
    backdrop-filter: blur(10px);
    border: 1px solid oklch(1 0 0 / 0.16);
    display: flex;
    flex-direction: column;
    gap: 3px;
    box-sizing: border-box;
    overflow: hidden;
  }
  .bgp-badge {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bgp-mock-card strong {
    font-size: 12.5px;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bgp-mock-card small {
    font-size: 11px;
    color: oklch(0.85 0.01 240);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .bgp-change-btn {
    position: absolute;
    right: 12px;
    bottom: 12px;
    z-index: 3;
    height: 30px;
    padding: 0 12px;
    border-radius: 8px;
    border: 1px solid oklch(1 0 0 / 0.25);
    background: oklch(0.1 0.01 240 / 0.86);
    backdrop-filter: blur(8px);
    color: #fff;
    font-family: var(--font-sans);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .bgp-change-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .bgp-empty {
    width: 100%;
    height: 100%;
    padding: 16px 18px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 6px;
    cursor: pointer;
    overflow: hidden;
  }
  .bgp-empty:hover .bgp-empty-icon {
    transform: scale(1.06);
    border-color: var(--accent);
    color: var(--accent);
  }
  .bgp-empty-icon {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    background: var(--accent-soft);
    border: 1px solid var(--border-strong);
    color: var(--accent);
    display: grid;
    place-items: center;
    flex: none;
    transition: all 0.16s ease;
  }
  .bgp-empty strong {
    font-size: 12.5px;
    color: var(--text-primary);
  }
  .bgp-empty small {
    font-size: 11px;
    color: var(--text-muted);
    max-width: 460px;
    line-height: 1.4;
  }

  .bg-gallery-col {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
  }
  .bg-url-form {
    display: flex;
    gap: 8px;
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
  }
  .bg-url-input {
    flex: 1 1 0%;
    width: 0;
    min-width: 0;
    height: 32px;
    padding: 0 11px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    outline: none;
    box-sizing: border-box;
  }
  .bg-url-input:focus {
    border-color: var(--accent);
  }
  .bg-url-btn {
    height: 32px;
    padding: 0 13px;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    flex: none;
    transition: all 0.15s ease;
  }
  .bg-url-btn:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
    transform: translateY(-1px);
  }
  .bg-url-btn:disabled {
    opacity: 0.45;
    cursor: default;
    transform: none;
  }

  .wp-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    width: 100%;
  }
  .wp-sec-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .wp-thumbs-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(128px, 1fr));
    gap: 8px;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
  }
  .wp-thumb-card {
    position: relative;
    width: 100%;
    min-width: 0;
    height: 72px;
    min-height: 72px;
    max-height: 72px;
    border-radius: 9px;
    overflow: hidden;
    contain: paint;
    border: 2px solid var(--border-subtle);
    background: var(--bg-canvas);
    padding: 0;
    box-sizing: border-box;
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }
  .wp-thumb-card:hover {
    transform: translateY(-1px);
    border-color: var(--border-strong);
  }
  .wp-thumb-card.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent), 0 4px 12px var(--accent-glow);
  }
  .wp-thumb-card img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    object-fit: cover;
    display: block;
    pointer-events: none;
  }
  .wp-thumb-meta {
    position: absolute;
    inset: auto 0 0 0;
    z-index: 1;
    padding: 4px 7px;
    background: linear-gradient(to top, oklch(0 0 0 / 0.9), transparent);
    display: flex;
    flex-direction: column;
    text-align: left;
    pointer-events: none;
  }
  .wp-thumb-meta strong {
    font-size: 10px;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .wp-thumb-meta span {
    font-family: var(--font-mono);
    font-size: 8px;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .wp-thumb-card.user-card {
    cursor: default;
  }
  .wp-user-select {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    display: block;
    overflow: hidden;
  }
  .wp-user-del {
    position: absolute;
    top: 4px;
    right: 4px;
    z-index: 2;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: none;
    background: oklch(0.1 0.01 240 / 0.85);
    color: #fff;
    font-size: 10px;
    display: grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s ease, background 0.15s ease;
  }
  .wp-thumb-card.user-card:hover .wp-user-del {
    opacity: 1;
  }
  .wp-user-del:hover {
    background: var(--danger);
  }

  .discord-bg-controls-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 12px 16px;
    padding: 12px 14px;
    border-radius: 11px;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    min-width: 0;
    width: 100%;
    max-width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }
  .ctrl-sub-hint {
    font-size: 10.5px;
    color: var(--text-muted);
  }
  .dual-sliders {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 8px;
    min-width: 0;
  }

  @container settings (max-width: 560px) {
    .code-bars-grid,
    .custom-code-creator,
    .discord-bg-top-grid,
    .discord-bg-controls-grid {
      grid-template-columns: minmax(0, 1fr);
    }
    .wp-thumbs-grid {
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    }
  }

  /* ===== Browser-style settings layout (sidebar tabs + single page) ===== */
  .settings-shell {
    display: grid;
    grid-template-columns: 192px minmax(0, 1fr);
    gap: 18px;
    align-items: start;
    min-width: 0;
    width: 100%;
    max-width: 1180px;
    margin: 0 auto;
    padding: 4px 0 24px;
    box-sizing: border-box;
  }
  .settings-nav {
    position: sticky;
    top: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
  }
  .settings-nav-title {
    margin: 6px 10px 8px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
  }
  .settings-tab {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 12px;
    border: none;
    border-left: 3px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 13.5px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s ease, color 0.15s ease;
  }
  .settings-tab:hover {
    background: var(--bg-surface-2);
    color: var(--text-primary);
  }
  .settings-tab.active {
    background: var(--accent-soft);
    color: var(--accent);
    border-left-color: var(--accent);
  }
  .settings-tab .tab-ico {
    width: 18px;
    text-align: center;
    font-size: 15px;
    line-height: 1;
  }
  .tab-single {
    display: block;
  }
  .settings-stage {
    container: settings / inline-size;
    padding: 0;
  }

  /* Corner Roundness Customizer & Live Preview */
  .radius-live-badge {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    padding: 4px 12px;
    border-radius: var(--ui-radius-bar, 999px);
  }
  .radius-customizer-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(240px, 1fr);
    gap: 16px;
    align-items: stretch;
  }
  .radius-controls-col {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 14px;
    min-width: 0;
  }
  .radius-live-preview {
    background: var(--bg-canvas);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 16px);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    justify-content: center;
  }
  .rp-tag-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 8px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius-bar, 999px);
  }
  .rp-chip {
    font-size: 10.5px;
    font-weight: 600;
    padding: 3px 9px;
    border-radius: var(--ui-radius-bar, 999px);
    background: var(--bg-surface-1);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }
  .rp-chip.active {
    background: var(--accent);
    color: var(--accent-fg, #090d16);
    border-color: var(--accent);
    font-weight: 700;
  }
  .rp-cards-row {
    display: grid;
    grid-template-columns: 92px 1fr;
    gap: 10px;
    align-items: stretch;
  }
  .rp-song-card {
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 16px);
    padding: 7px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .rp-thumb {
    width: 100%;
    aspect-ratio: 1 / 1;
    border-radius: var(--ui-radius-sm, 11px);
    background: linear-gradient(135deg, var(--accent), var(--accent-soft));
  }
  .rp-lines {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .rp-l1 {
    height: 6px;
    width: 85%;
    border-radius: 99px;
    background: var(--text-primary);
    opacity: 0.75;
  }
  .rp-l2 {
    height: 5px;
    width: 55%;
    border-radius: 99px;
    background: var(--text-secondary);
    opacity: 0.5;
  }
  .rp-info-box {
    background: linear-gradient(145deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 65%, var(--accent-soft) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 16px);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 3px;
  }
  .rp-kicker {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent);
  }
  .rp-info-box strong {
    font-size: 13px;
    color: var(--text-primary);
  }
  .rp-info-box small {
    font-size: 11px;
    color: var(--text-secondary);
  }
  @container settings (max-width: 620px) {
    .radius-customizer-grid {
      grid-template-columns: 1fr;
    }
  }
  @container stage (max-width: 780px) {
    .settings-shell {
      grid-template-columns: minmax(0, 1fr);
      gap: 14px;
    }
    .settings-nav {
      position: static;
      flex-direction: row;
      min-width: 0;
      overflow-x: auto;
      scrollbar-width: thin;
    }
    .settings-nav-title {
      display: none;
    }
    .settings-tab {
      width: auto;
      flex-shrink: 0;
      white-space: nowrap;
      border-left: none;
      border-bottom: 3px solid transparent;
    }
    .settings-tab.active {
      border-bottom-color: var(--accent);
    }
  }
</style>
