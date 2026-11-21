<script lang="ts">
  import { Download, ShieldCheck, Square, AudioLines, Sparkles, FolderOpen, Disc3, Palette, Check, SlidersHorizontal, Maximize2, PanelRight, Mic2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { api, type LyricsSettings } from "$lib/api";
  import { lyricsState } from "$lib/lyrics.svelte";
  import { lyricsUi, type LyricsLayoutMode, type VinylSkin, type LyricsFontSize, type LyricsDimInactive, type LyricsBgStyle, type LyricsWindowMode } from "$lib/lyricsUiConfig.svelte";
  import LyricsBatch from "$lib/LyricsBatch.svelte";
  import lyricStandards from "../../docs/LYRIC-STANDARDS.md?raw";
  const standardRules = lyricStandards.split(/\r?\n/).filter(line => /^\d+\.\s/.test(line)).map(line => line.replace(/^\d+\.\s+/, '').replaceAll('`', ''));

  // --- Tab state ---
  let activeTab = $state<'ui' | 'ai'>('ui');

  const VINYL_SWATCHES = [
    { label: "Đen tuyền phay xước cổ điển", color: "#111215", skin: "black" as const },
    { label: "Vàng ánh kim hoàng gia (Retro Gold)", color: "#eab308", skin: "gold" as const },
    { label: "Cẩm thạch loang màu (Liquid Marble)", color: "#a855f7", skin: "marble" as const },
    { label: "Neon Cyberpunk phát sáng", color: "#06b6d4", skin: "cyberpunk" as const },
    { label: "Đỏ Ruby vang Bordeaux", color: "#e11d48", skin: "custom" as const },
    { label: "Xanh Emerald ngọc lục bảo", color: "#10b981", skin: "custom" as const },
    { label: "Xanh Sapphire đại dương sâu", color: "#2563eb", skin: "custom" as const },
    { label: "Tím thạch anh Amethyst", color: "#7c3aed", skin: "custom" as const },
    { label: "Cam hoàng hôn Sunset Glow", color: "#f97316", skin: "custom" as const },
    { label: "Bạc Titanium Metal sang trọng", color: "#94a3b8", skin: "custom" as const },
  ];

  let settings = $state<LyricsSettings | null>(null);
  let apiKey = $state("");
  const audioReady = $derived(
    Boolean(
      settings &&
      (settings.config.provider === "offline"
        ? settings.models.some(m => m.name === settings?.config.model && m.installed)
        : settings.keys.includes(settings.config.provider))
    )
  );
  let busy = $state(false);
  let downloading = $state(false);
  let message = $state("");
  let error = $state("");
  let legacyProvider = $state("");

  onMount(() => {
    refresh();
    const provider = localStorage.getItem("dlc.ai.stt_provider") || "groq";
    if (localStorage.getItem("dlc.ai.stt_key")) legacyProvider = provider;
    return lyricsState.subscribe();
  });

  async function refresh() {
    try {
      settings = await api.lyricsSettings();
      downloading = settings.downloading;
    } catch (e) {
      error = String(e);
    }
  }

  async function save() {
    if (!settings) return;
    busy = true; message = ""; error = "";
    try {
      // Cac tieu chuan lyric va quy trinh kiem duyet nhieu buoc luon luon tu dong duoc ap dung
      const isOffline = settings.config.provider === "offline";
      settings.config.audio_pipeline = !isOffline;
      settings.config.context_rewrite = !isOffline;
      settings.config.auto_review = !isOffline;
      settings.config.cloud_consent = !isOffline;
      settings.config.text_consent = !isOffline;

      if (apiKey.trim()) {
        await api.storeLyricsKey(settings.config.provider, apiKey.trim());
      }
      await api.saveLyricsSettings(settings.config, apiKey.trim() || null);
      apiKey = "";
      message = "Đã lưu cấu hình. Các tiêu chuẩn và quy trình đối chiếu luôn được tự động áp dụng.";
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function removeKey() {
    if (!settings || settings.config.provider === "offline") return;
    busy = true; error = "";
    try {
      await api.storeLyricsKey(settings.config.provider, "");
      await refresh();
      message = "Đã xóa API key của nhà cung cấp này.";
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function migrateKey() {
    const old = localStorage.getItem("dlc.ai.stt_key");
    if (!old || !["groq", "gemini", "openai"].includes(legacyProvider)) {
      error = "Khóa cũ thuộc endpoint tùy chỉnh. Hãy lưu lại khóa trước khi gỡ cấu hình cũ.";
      return;
    }
    busy = true; error = "";
    try {
      await api.storeLyricsKey(legacyProvider, old);
      localStorage.removeItem("dlc.ai.stt_key");
      legacyProvider = "";
      message = "Đã chuyển khóa cũ sang kho hệ điều hành và xóa bản rõ khỏi localStorage.";
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function openFolder() {
    try {
      await api.openLyricsFolder();
    } catch (e) {
      error = String(e);
    }
  }

  async function download(model: string) {
    downloading = true; error = ""; message = "Đang tải model và whisper.cpp từ nguồn đã ghim phiên bản. Chỉ cài khi checksum khớp.";
    lyricsState.download = null;
    try {
      await api.downloadLyricsModel(model);
      message = "Model và công cụ offline đã cài đặt thành công.";
    } catch (e) {
      error = String(e);
    } finally {
      downloading = false;
      await refresh();
    }
  }
</script>

<section class="lyrics-settings" aria-label="Cài đặt lời bài hát">

  <!-- Tab navigation -->
  <div class="settings-tabs" role="tablist" aria-label="Phân mục cài đặt">
    <button
      role="tab"
      type="button"
      class="settings-tab"
      class:active={activeTab === 'ui'}
      aria-selected={activeTab === 'ui'}
      onclick={() => activeTab = 'ui'}
    >
      <Palette size={15} />
      <span>Giao diện lời bài hát</span>
    </button>
    <button
      role="tab"
      type="button"
      class="settings-tab"
      class:active={activeTab === 'ai'}
      aria-selected={activeTab === 'ai'}
      onclick={() => activeTab = 'ai'}
    >
      <Sparkles size={15} />
      <span>AI & Lời bài hát</span>
    </button>
  </div>

  <!-- ===== TAB: GIAO DIỆN ===== -->
  {#if activeTab === 'ui'}

  <!-- Card 1: Chế độ cửa sổ hiển thị -->
  <div class="studio-ui-card">
    <div class="studio-card-head">
      <div class="card-icon-box"><Maximize2 size={20} /></div>
      <div>
        <h3>Chế độ cửa sổ lời bài hát</h3>
        <p class="card-subtext">Chọn cách trang lời bài hát xuất hiện khi bạn bấm nút Lời bài hát.</p>
      </div>
    </div>

    <div class="window-mode-grid">
      <button
        type="button"
        class="window-mode-card"
        class:chosen={lyricsUi.config.windowMode === 'fullscreen'}
        onclick={() => lyricsUi.setWindowMode('fullscreen')}
      >
        <div class="wm-preview wm-preview--full">
          <div class="wm-screen">
            <div class="wm-lyrics-full">
              <span></span><span></span><span class="wm-active"></span><span></span>
            </div>
          </div>
        </div>
        <div class="wm-label">
          <Maximize2 size={14} />
          <strong>Chèn lên toàn bộ cửa sổ</strong>
        </div>
        <p class="wm-desc">Overlay phủ kín toàn bộ cửa sổ ứng dụng (che cả thanh điều hướng và thanh phát nhạc).</p>
        {#if lyricsUi.config.windowMode === 'fullscreen'}
          <span class="wm-chosen-badge"><Check size={11} /> Đang dùng</span>
        {/if}
      </button>

      <button
        type="button"
        class="window-mode-card"
        class:chosen={lyricsUi.config.windowMode === 'mini'}
        onclick={() => lyricsUi.setWindowMode('mini')}
      >
        <div class="wm-preview wm-preview--mini">
          <div class="wm-screen-layout">
            <div class="wm-side-nav">
              <span class="wm-nav-dot"></span>
              <span class="wm-nav-dot"></span>
              <span class="wm-nav-dot"></span>
            </div>
            <div class="wm-stage-lyrics-area">
              <div class="wm-vinyl-circle"></div>
              <div class="wm-lyrics-lines">
                <span></span><span class="wm-active"></span><span></span>
              </div>
            </div>
          </div>
        </div>
        <div class="wm-label">
          <PanelRight size={14} />
          <strong>Tràn toàn bộ giao diện bên phải</strong>
        </div>
        <p class="wm-desc">Lời bài hát tràn ra toàn bộ khu vực bên phải — thanh điều hướng bên trái và thanh phát nhạc bên dưới vẫn giữ nguyên.</p>
        {#if lyricsUi.config.windowMode === 'mini'}
          <span class="wm-chosen-badge"><Check size={11} /> Đang dùng</span>
        {/if}
      </button>
    </div>
  </div>

  <!-- Card 2: Studio Vinyl UI -->
  <div class="studio-ui-card">
    <div class="studio-card-head">
      <div class="card-icon-box"><Disc3 size={20} /></div>
      <div>
        <h3>Giao diện Lời bài hát & Đĩa than (Vinyl Studio)</h3>
        <p class="card-subtext">Tùy biến phong cách hiển thị trang lời bài hát, đĩa than xoay tròn nghệ thuật và hiệu ứng chạy chữ karaoke.</p>
      </div>
    </div>

    <div class="studio-options-grid">
      <!-- 1. Bố cục hiển thị -->
      <div class="opt-group">
        <span class="opt-label">Chế độ hiển thị chính</span>
        <div class="pill-selector">
          <button
            type="button"
            class="pill-choice"
            class:chosen={lyricsUi.config.layout === 'vinyl'}
            onclick={() => lyricsUi.setLayout('vinyl')}
          >
            <Disc3 size={14} />
            <span>Đĩa than phòng thu</span>
          </button>
          <button
            type="button"
            class="pill-choice"
            class:chosen={lyricsUi.config.layout === 'stage'}
            onclick={() => lyricsUi.setLayout('stage')}
          >
            <Sparkles size={14} />
            <span>Sân khấu karaoke</span>
          </button>
          <button
            type="button"
            class="pill-choice"
            class:chosen={lyricsUi.config.layout === 'full'}
            onclick={() => lyricsUi.setLayout('full')}
          >
            <AudioLines size={14} />
            <span>Cuộn toàn bộ lời</span>
          </button>
        </div>
      </div>

      <!-- 2. Vị trí hiển thị logo & ảnh bìa bài hát -->
      <div class="opt-group">
        <span class="opt-label">Vị trí hiển thị logo / ảnh bìa bài hát (xoay tròn cùng đĩa khi phát)</span>
        <div class="pill-selector">
          {#each [
            { id: 'center', label: 'Tâm đĩa (Trong lòng)' },
            { id: 'rim', label: 'Vành viền ngoài' },
            { id: 'full', label: 'Toàn bộ mặt đĩa' }
          ] as opt}
            <button
              type="button"
              class="pill-choice"
              class:chosen={lyricsUi.config.vinylCoverPos === opt.id}
              onclick={() => lyricsUi.setVinylCoverPos(opt.id as any)}
            >
              {#if lyricsUi.config.vinylCoverPos === opt.id}<Check size={13} />{/if}
              <span>{opt.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- 3. Màu sắc đĩa than thiết kế giống như cài đặt màu nền -->
      <div class="color-group-box">
        <div class="cg-head">
          <div class="cg-title-group">
            <strong>Màu sắc & Phong cách đĩa than (Vinyl Color & Appearance)</strong>
            <small>Hiển thị khi bài hát không có logo hoặc khi sử dụng đĩa màu nghệ thuật</small>
          </div>
          <label class="color-picker-pill" title="Bấm để mở bảng chọn màu đĩa than tự do">
            <input
              type="color"
              value={lyricsUi.config.vinylColorHex || '#6366f1'}
              oninput={(e) => lyricsUi.setVinylColorHex(e.currentTarget.value)}
            />
            <span class="cp-dot" style="background: {lyricsUi.config.vinylColorHex || '#6366f1'}"></span>
            <span>Chọn màu đĩa</span>
          </label>
        </div>

        <!-- Dãy nút chọn kiểu dáng đĩa than -->
        <div class="skin-preset-pills">
          {#each [
            { id: 'marble', label: 'Cẩm thạch (Marble)' },
            { id: 'black', label: 'Đen phay xước' },
            { id: 'cyberpunk', label: 'Neon Cyberpunk' },
            { id: 'gold', label: 'Retro Gold' },
            { id: 'custom', label: 'Màu tùy chỉnh' }
          ] as opt}
            <button
              type="button"
              class="pill-choice"
              class:chosen={lyricsUi.config.vinylSkin === opt.id}
              onclick={() => lyricsUi.setVinylSkin(opt.id as any)}
            >
              {#if lyricsUi.config.vinylSkin === opt.id}<Check size={12} />{/if}
              <span>{opt.label}</span>
            </button>
          {/each}
        </div>

        <!-- Dải các ô tròn swatch màu sắc mẫu -->
        <div class="swatch-row">
          {#each VINYL_SWATCHES as sw}
            <button
              type="button"
              class="color-swatch"
              class:active={lyricsUi.config.vinylSkin === sw.skin && (sw.skin !== 'custom' || lyricsUi.config.vinylColorHex?.toLowerCase() === sw.color.toLowerCase())}
              style="--sw-color: {sw.color}"
              title={sw.label}
              onclick={() => {
                if (sw.skin === 'custom') {
                  lyricsUi.setVinylColorHex(sw.color);
                } else {
                  lyricsUi.setVinylSkin(sw.skin);
                  lyricsUi.setVinylColorHex(sw.color);
                }
              }}
            >
              <span></span>
            </button>
          {/each}
        </div>

        <!-- Thanh trượt dải quang phổ (Hue Slider) -->
        <div class="control-block">
          <div class="slider-head">
            <span class="ctrl-label">Tông màu quang phổ đĩa than (Vinyl Hue)</span>
            <span class="ctrl-val">{Math.round(lyricsUi.config.vinylHue ?? 240)}°</span>
          </div>
          <input
            type="range"
            class="hue-slider"
            min="0"
            max="360"
            step="1"
            value={lyricsUi.config.vinylHue ?? 240}
            oninput={(e) => {
              lyricsUi.setVinylHue(+e.currentTarget.value);
            }}
            aria-label="Tông màu quang phổ đĩa than"
          />
        </div>
      </div>

      <!-- 4. Hiệu ứng -->
      <div class="opt-group">
        <span class="opt-label">Hiệu ứng chữ Karaoke & Cơ học</span>
        <div class="check-grid">
          <label class="studio-check">
            <input type="checkbox" checked={lyricsUi.config.bouncingCursor} onchange={e => lyricsUi.setBouncingCursor(e.currentTarget.checked)} />
            <span>Con trỏ nhảy chữ (Bouncing Block Cursor) theo nhịp từng từ</span>
          </label>
          <label class="studio-check">
            <input type="checkbox" checked={lyricsUi.config.showSectionTags} onchange={e => lyricsUi.setSectionTags(e.currentTarget.checked)} />
            <span>Hiển thị nhãn đoạn nhạc (verse, hook, drop, outro...)</span>
          </label>
          <label class="studio-check">
            <input type="checkbox" checked={lyricsUi.config.showTonearm} onchange={e => lyricsUi.setTonearm(e.currentTarget.checked)} />
            <span>Cần kim đĩa than (Tonearm cơ học — kéo gạt bằng chuột để Bật/Tắt nhạc)</span>
          </label>
        </div>
      </div>

      <!-- 5. Cỡ chữ, nền, thanh trượt tốc độ quay -->
      <div class="opt-columns">
        <div class="opt-group">
          <span class="opt-label">Cỡ chữ lời bài hát</span>
          <div class="pill-selector">
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.fontSize === 'compact'} onclick={() => lyricsUi.setFontSize('compact')}>Nhỏ (Gọn)</button>
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.fontSize === 'normal'} onclick={() => lyricsUi.setFontSize('normal')}>Tiêu chuẩn</button>
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.fontSize === 'large'} onclick={() => lyricsUi.setFontSize('large')}>Lớn</button>
          </div>
        </div>
        <div class="opt-group">
          <span class="opt-label">Phong cách nền trang lời</span>
          <div class="pill-selector">
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.bgStyle === 'apple-glass'} onclick={() => lyricsUi.setBgStyle('apple-glass')}>Liquid Glass</button>
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.bgStyle === 'glow'} onclick={() => lyricsUi.setBgStyle('glow')}>Hào quang</button>
            <button type="button" class="pill-choice" class:chosen={lyricsUi.config.bgStyle === 'oled'} onclick={() => lyricsUi.setBgStyle('oled')}>Đen OLED</button>
          </div>
        </div>
        <div class="opt-group">
          <div class="opt-header-row">
            <span class="opt-label">Tốc độ quay đĩa than</span>
            <span class="speed-val-pill">{lyricsUi.config.vinylSpeedRpm || 33} RPM</span>
          </div>
          <div class="speed-slider-box">
            <span class="slider-side-label">Chậm</span>
            <input
              type="range"
              min="16"
              max="58"
              step="1"
              value={lyricsUi.config.vinylSpeedRpm || 33}
              oninput={e => lyricsUi.setVinylSpeedRpm(Number(e.currentTarget.value))}
              class="speed-slider-control"
              aria-label="Tốc độ đĩa quay"
            />
            <span class="slider-side-label">Nhanh</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  {/if}
  <!-- ===== END TAB UI ===== -->

  <!-- ===== TAB: AI & LỜI BÀI HÁT ===== -->
  {#if activeTab === 'ai'}

  <header>
    <h2>Cài đặt AI viết lời bài hát</h2>
    <p>Chọn model AI tạo lời bài hát. Dù sử dụng bất kỳ AI nào, hệ thống luôn tự động tuân thủ tiêu chuẩn ngữ cảnh và thực hiện đầy đủ các bước đối chiếu âm thanh, phân câu nhịp điệu, căn mốc từng chữ và kiểm tra toàn vẹn trước khi xuất lyric.</p>
  </header>

  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if message}<p class="notice" role="status">{message}</p>{/if}

  {#if legacyProvider}
    <div class="migration">
      <p>Phát hiện API key của bản cũ đang lưu dạng rõ trong localStorage.</p>
      <button onclick={migrateKey} disabled={busy}>Chuyển khóa cũ sang kho Windows</button>
    </div>
  {/if}

  {#if settings}
    <form onsubmit={(event) => { event.preventDefault(); save(); }}>
      <fieldset disabled={busy}>
        <legend>Model AI viết lời</legend>
        <p class="readiness">
          <AudioLines size={16} />
          <span>{audioReady ? "AI đã sẵn sàng hoạt động với cấu hình hiện tại." : (settings.config.provider === "offline" ? "Cần tải model offline bên dưới để sử dụng." : `Cần nhập API key cho ${settings.config.provider.toUpperCase()} để hoạt động.`)}</span>
        </p>
        <label class="field">Nhà cung cấp AI
          <select bind:value={settings.config.provider} onchange={() => { apiKey = ""; }}>
            <option value="gemini">Google · Gemini (Khuyên dùng · phân câu và ngữ cảnh tốt nhất)</option>
            <option value="openai">OpenAI · Whisper-1</option>
            <option value="groq">Groq · Whisper Large v3 Turbo (Siêu nhanh)</option>
            <option value="offline">Offline · whisper.cpp trên máy (Bảo mật, không cần mạng)</option>
          </select>
        </label>
        {#if settings.config.provider === "gemini"}
          <label class="field">Phiên bản Model Gemini
            <select bind:value={settings.config.gemini_model}>
              <option value="gemini-2.5-flash">gemini-2.5-flash (Cân bằng tốc độ & chất lượng cao)</option>
              <option value="gemini-2.5-flash-lite">gemini-2.5-flash-lite (Siêu nhẹ & tiết kiệm)</option>
              <option value="gemini-3-flash-preview">gemini-3-flash-preview (Thế hệ mới nhất)</option>
            </select>
          </label>
        {/if}
        {#if settings.config.provider !== "offline"}
          <label class="field">
            <span>API key {settings.config.provider.toUpperCase()} · để trống để giữ khóa đã lưu</span>
            <input type="password" bind:value={apiKey} autocomplete="off" spellcheck="false"
              placeholder={settings.keys.includes(settings.config.provider) ? "Đã lưu khóa trong Windows Credential Manager (nhập mới để đổi)" : "Nhập API key để lưu an toàn vào Windows"} />
          </label>
          <div class="key-status-row">
            <p class="hint">
              <ShieldCheck size={14} />
              <span>{settings.keys.includes(settings.config.provider) ? `Đã lưu API key cho ${settings.config.provider.toUpperCase()} trong kho hệ điều hành.` : `Chưa lưu API key cho ${settings.config.provider.toUpperCase()}.`}</span>
            </p>
            {#if settings.keys.includes(settings.config.provider)}
              <button type="button" class="btn-subtle" onclick={removeKey}>Xóa khóa đã lưu</button>
            {/if}
          </div>
        {:else}
          <label class="field">Model sử dụng trên máy
            <select bind:value={settings.config.model}>
              <option value="base">Base · 142 MiB (Nhẹ, đa ngôn ngữ)</option>
              <option value="small">Small · 466 MiB (Chính xác hơn, đa ngôn ngữ)</option>
            </select>
          </label>
          <div class="model-list">
            {#each settings.models as model}
              <div class="model-row">
                <span><strong>{model.name}</strong> · {(model.bytes / 1024 / 1024).toFixed(1)} MiB · {model.installed ? "Đã cài" : "Chưa tải"}</span>
                <button type="button" disabled={downloading} onclick={() => download(model.name)}>
                  <Download size={14} /> {model.installed ? "Kiểm tra / cài lại công cụ" : "Tải model"}
                </button>
              </div>
            {/each}
          </div>
          {#if downloading}
            <div class="download-state" aria-live="polite">
              <p>{lyricsState.download ? `${(lyricsState.download.done / 1024 / 1024).toFixed(1)} / ${(lyricsState.download.total / 1024 / 1024).toFixed(1)} MiB` : "Đang kết nối hoặc cài công cụ…"}</p>
              {#if lyricsState.download}<progress value={lyricsState.download.done} max={lyricsState.download.total} aria-label="Tiến độ tải model"></progress>{/if}
              <button type="button" onclick={() => api.cancelLyricsDownload().catch(e => error = String(e))}><Square size={12} /> Hủy tải</button>
            </div>
          {/if}
        {/if}
      </fieldset>

      <fieldset disabled={busy}>
        <legend>Tính năng liên quan</legend>
        <label class="field">Ngôn ngữ bài hát ưu tiên
          <select bind:value={settings.config.language}>
            <option value="auto">Tự nhận diện ngôn ngữ</option>
            <option value="vi">Tiếng Việt</option>
            <option value="en">Tiếng Anh</option>
          </select>
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={settings.config.automatic} />
          <span>Tự động tạo lời bằng AI khi mở bài hát chưa có lời</span>
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={settings.config.network_lookup} />
          <span>Cho phép tìm kiếm lời bài hát có sẵn qua mạng (LRCLIB, phụ đề YouTube)</span>
        </label>
        {#if settings.config.provider !== "offline"}
          <label class="check">
            <input type="checkbox" bind:checked={settings.config.offline_fallback} />
            <span>Tự động dự phòng bằng model offline nếu dịch vụ cloud gặp sự cố kết nối</span>
          </label>
        {/if}
      </fieldset>

      <fieldset disabled={busy}>
        <legend>Thư mục lưu trữ lời bài hát</legend>
        {#if settings.lyrics_dir}
          <p class="lyrics-dir-path"><code>{settings.lyrics_dir}</code></p>
        {/if}
        <div class="dir-actions-row">
          <p class="hint">
            <span>Đang lưu trữ <strong>{settings.lyrics_count ?? 0}</strong> bản lời bài hát đã đồng bộ và bản nháp.</span>
          </p>
          <button type="button" class="btn-open-dir" onclick={openFolder}>
            <FolderOpen size={15} />
            <span>Mở thư mục lưu trữ lyric</span>
          </button>
        </div>
      </fieldset>

      <div class="standards-banner">
        <div class="banner-head">
          <Sparkles size={18} />
          <strong>Tiêu chuẩn & quy trình luôn tự động thực hiện cho mọi AI</strong>
        </div>
        <p class="banner-desc">Hệ thống luôn tự động tuân thủ 16 quy chuẩn lời bài hát (giữ đúng phát âm, chính tả tiếng Việt, viết hoa, dấu câu, giữ nguyên từ lặp, không bịa lời) và luôn trải qua đầy đủ các bước:</p>
        <ol class="pipeline-steps">
          <li><strong>Chép phụ đề:</strong> AI nhận diện lời hát ban đầu từ âm thanh bài hát.</li>
          <li><strong>Nghe đối chiếu:</strong> Đối chiếu phát âm, chính tả, ngữ cảnh và khoảng nghỉ.</li>
          <li><strong>Rà soát & viết lại:</strong> Chuẩn hóa câu theo ngữ cảnh, xử lý dấu câu và viết hoa.</li>
          <li><strong>Phân câu nhịp điệu:</strong> Ghép câu theo cấu trúc âm nhạc, giữ nguyên từ lặp.</li>
          <li><strong>Căn mốc từng chữ:</strong> Lấy mốc thời gian bắt đầu/kết thúc chuẩn xác cho từng từ.</li>
          <li><strong>Kiểm tra toàn vẹn:</strong> Đối chiếu thứ tự, loại trừ mốc chồng chéo trước khi xuất lyric.</li>
        </ol>
        <details>
          <summary>Xem chi tiết 16 tiêu chuẩn AI tự đối chiếu</summary>
          <ol class="pipeline-steps">
            {#each standardRules as rule}
              <li>{rule}</li>
            {/each}
          </ol>
        </details>
      </div>

      <button class="save" type="submit" disabled={busy}>{busy ? "Đang lưu…" : "Lưu cài đặt lời bài hát"}</button>
    </form>
    <LyricsBatch />
  {:else if !error}<p role="status">Đang tải cài đặt…</p>{/if}

  {/if}
  <!-- ===== END TAB AI ===== -->

</section>

<style>
  .lyrics-settings { color: var(--text-primary); max-width: 960px; padding: 24px; }

  /* --- Tabs --- */
  .settings-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 24px;
    padding: 4px;
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    width: fit-content;
  }
  .settings-tab {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 18px;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-secondary);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    align-self: auto;
  }
  .settings-tab:hover:not(.active) {
    color: var(--text-primary);
    background: var(--bg-surface-2);
    border-color: var(--border-subtle);
  }
  .settings-tab.active {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
    font-weight: 650;
  }

  /* --- Window mode cards --- */
  .window-mode-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }
  .window-mode-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font: inherit;
    text-align: left;
    cursor: pointer;
    align-self: auto;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    position: relative;
  }
  .window-mode-card:hover { border-color: var(--accent); }
  .window-mode-card.chosen { border-color: var(--accent); background: var(--accent-soft, color-mix(in srgb, var(--accent) 12%, transparent)); }
  .wm-preview {
    width: 100%;
    aspect-ratio: 16/9;
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-surface-3, var(--bg-surface-1));
    border: 1px solid var(--border-subtle);
  }
  .wm-screen { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; padding: 10%; box-sizing: border-box; }
  /* Full: lyrics centred across entire screen */
  .wm-lyrics-full { display: flex; flex-direction: column; align-items: center; gap: 6px; width: 100%; }
  .wm-lyrics-full span { height: 4px; border-radius: 2px; background: var(--text-secondary); opacity: 0.3; width: 70%; }
  .wm-lyrics-full span.wm-active { background: var(--accent); opacity: 1; width: 90%; height: 5px; }
  /* Right stage view (Spotify style) preview */
  .wm-screen-layout { width: 100%; height: 100%; display: flex; gap: 4px; padding: 6px; box-sizing: border-box; }
  .wm-side-nav { width: 18%; height: 100%; background: var(--bg-surface-1); border-radius: 4px; opacity: 0.6; display: flex; flex-direction: column; gap: 4px; padding: 4px 3px; align-items: center; }
  .wm-nav-dot { width: 100%; height: 4px; border-radius: 2px; background: var(--text-muted); opacity: 0.4; }
  .wm-stage-lyrics-area { flex: 1; height: 100%; background: var(--bg-surface-3, var(--bg-base)); border-radius: 4px; border: 1.5px solid var(--accent); padding: 6px; display: flex; align-items: center; gap: 8px; box-shadow: inset 0 0 12px var(--accent-soft, rgba(168, 85, 247, 0.15)); }
  .wm-vinyl-circle { width: 34px; height: 34px; border-radius: 50%; background: radial-gradient(circle, var(--accent) 20%, #111 25%, #222 60%, #000 100%); flex-shrink: 0; box-shadow: 0 0 8px rgba(0,0,0,0.5); }
  .wm-lyrics-lines { flex: 1; display: flex; flex-direction: column; gap: 4px; justify-content: center; }
  .wm-lyrics-lines span { height: 3px; border-radius: 2px; background: var(--text-secondary); opacity: 0.4; width: 80%; }
  .wm-lyrics-lines span.wm-active { background: var(--accent); opacity: 1; width: 100%; height: 4.5px; }
  .wm-label { display: flex; align-items: center; gap: 7px; font-size: 13px; font-weight: 700; }
  .wm-desc { margin: 0; font-size: 11.5px; line-height: 1.5; color: var(--text-secondary); }
  .wm-chosen-badge { display: inline-flex; align-items: center; gap: 4px; font-size: 11px; font-weight: 650; color: var(--accent); }

  /* --- AI tab --- */
  header { margin-bottom: 24px; }
  .readiness { display: flex; align-items: flex-start; gap: 8px; font-size: 13px; line-height: 1.6; margin: 0; color: var(--text-secondary); }
  .readiness :global(svg) { flex-shrink: 0; margin-top: 2px; color: var(--accent); }
  h2 { margin: 0 0 8px; font-size: 22px; letter-spacing: -.02em; }
  p { line-height: 1.6; margin: 8px 0; color: var(--text-secondary); }
  form { display: flex; flex-direction: column; gap: 24px; }
  fieldset { margin: 0; padding: 18px 0 0; border: 0; border-top: 1px solid var(--border-subtle); min-width: 0; display: flex; flex-direction: column; gap: 14px; }
  legend { font-size: 15px; font-weight: 700; padding-inline-end: 16px; color: var(--text-primary); }
  .field { display: flex; flex-direction: column; gap: 7px; font-size: 13px; max-width: 580px; }
  input:not([type="checkbox"]), select { min-width: 0; width: 100%; padding: 9px 11px; background: var(--bg-surface-2); border: 1px solid var(--border-subtle); border-radius: 7px; color: var(--text-primary); font: inherit; box-sizing: border-box; }
  .check { display: flex; align-items: flex-start; gap: 9px; font-size: 13px; line-height: 1.6; max-width: 75ch; cursor: pointer; }
  input[type="checkbox"] { accent-color: var(--accent); margin-top: 4px; flex-shrink: 0; cursor: pointer; }
  .hint { margin: 0; font-size: 12px; max-width: 75ch; display: flex; align-items: center; gap: 6px; color: var(--text-secondary); }
  .key-status-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; max-width: 580px; }
  .btn-subtle { padding: 4px 8px; font-size: 12px; background: transparent; border-color: var(--border-subtle); color: var(--text-secondary); }
  .btn-subtle:hover { color: var(--danger, #e57373); border-color: var(--danger, #e57373); }
  .model-list { display: flex; flex-direction: column; max-width: 580px; }
  .lyrics-dir-path { margin: 0; font-size: 12px; color: var(--text-secondary); word-break: break-all; max-width: 580px; }
  .lyrics-dir-path code { background: var(--bg-surface-2); padding: 5px 9px; border-radius: 6px; border: 1px solid var(--border-subtle); display: inline-block; width: 100%; box-sizing: border-box; }
  .dir-actions-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; max-width: 580px; }
  .btn-open-dir { background: var(--bg-surface-2); border: 1px solid var(--border-subtle); color: var(--text-primary); font-weight: 600; padding: 7px 12px; }
  .btn-open-dir:hover { border-color: var(--accent); }
  .model-row { display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; padding: 10px 0; border-bottom: 1px solid var(--border-subtle); font-size: 13px; font-variant-numeric: tabular-nums; }
  .download-state { display: flex; flex-direction: column; gap: 8px; max-width: 580px; padding: 12px 0; }
  .standards-banner { background: var(--bg-surface-2); border: 1px solid var(--border-subtle); border-radius: 9px; padding: 16px 18px; display: flex; flex-direction: column; gap: 10px; font-size: 13px; }
  .banner-head { display: flex; align-items: center; gap: 9px; color: var(--accent); }
  .banner-head strong { font-size: 14px; color: var(--text-primary); }
  .banner-desc { margin: 0; line-height: 1.5; color: var(--text-secondary); }
  .pipeline-steps { margin: 0; padding-left: 20px; font-size: 13px; line-height: 1.8; color: var(--text-secondary); }
  details { margin-top: 4px; }
  details summary { cursor: pointer; color: var(--accent); font-weight: 600; padding: 4px 0; }
  details summary:hover { text-decoration: underline; }
  details .pipeline-steps { margin-top: 8px; }
  button { align-self: flex-start; display: inline-flex; align-items: center; gap: 7px; padding: 8px 14px; border: 1px solid var(--border-subtle); border-radius: 7px; background: var(--bg-surface-2); color: var(--text-primary); font: inherit; font-size: 13px; cursor: pointer; }
  button:hover:not(:disabled) { border-color: var(--accent); }
  button:disabled { opacity: .5; cursor: wait; }
  input:focus-visible, select:focus-visible, button:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
  .save { margin-bottom: 24px; font-weight: 700; background: var(--accent); color: var(--bg-base, #000); border-color: var(--accent); }
  .notice, .migration { padding: 12px 0; border-bottom: 1px solid var(--border-subtle); }
  .error { color: var(--danger, var(--text-primary)); overflow-wrap: anywhere; }
  progress { width: 100%; height: 6px; accent-color: var(--accent); }

  /* --- Studio UI card (tab Giao diện) --- */
  .studio-ui-card { background: var(--bg-surface-1); border: 1px solid var(--border-subtle); border-radius: 14px; padding: 22px; margin-bottom: 24px; display: flex; flex-direction: column; gap: 18px; box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08); }
  .studio-card-head { display: flex; align-items: center; gap: 14px; }
  .card-icon-box { width: 42px; height: 42px; border-radius: 10px; background: var(--accent-soft); color: var(--accent); display: flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .studio-card-head h3 { margin: 0; font-size: 16px; font-weight: 700; color: var(--text-primary); }
  .card-subtext { margin: 3px 0 0; font-size: 12px; color: var(--text-secondary); }
  .studio-options-grid { display: flex; flex-direction: column; gap: 16px; }
  .opt-group { display: flex; flex-direction: column; gap: 8px; }
  .opt-label { font-size: 11.5px; font-weight: 650; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-secondary); }
  .pill-selector { display: flex; flex-wrap: wrap; gap: 6px; }
  .pill-choice { display: inline-flex; align-items: center; gap: 6px; min-height: 32px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--border-subtle); background: var(--bg-surface-2); color: var(--text-primary); font-size: 12px; font-weight: 500; cursor: pointer; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1); align-self: auto; }
  .pill-choice:hover { border-color: var(--accent); background: var(--bg-surface-3); }
  .pill-choice.chosen { background: var(--accent); border-color: var(--accent); color: #ffffff; font-weight: 650; }
  .check-grid { display: flex; flex-direction: column; gap: 8px; }
  .studio-check { display: flex; align-items: center; gap: 9px; font-size: 12.5px; color: var(--text-primary); cursor: pointer; }
  .studio-check input { accent-color: var(--accent); cursor: pointer; }
  .opt-columns { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; }

  /* Speed range slider */
  .opt-header-row { display: flex; align-items: center; justify-content: space-between; margin-bottom: 2px; }
  .speed-val-pill { font-size: 11.5px; font-weight: 700; color: var(--accent); background: var(--accent-soft, rgba(168, 85, 247, 0.15)); border: 1px solid var(--accent); padding: 1px 8px; border-radius: 999px; font-family: var(--font-mono, monospace); }
  .speed-slider-box { display: flex; align-items: center; gap: 12px; background: var(--bg-surface-2, rgba(255, 255, 255, 0.04)); padding: 8px 14px; border-radius: 10px; border: 1px solid var(--border-subtle); }
  .slider-side-label { font-size: 11px; color: var(--text-muted); font-weight: 600; flex-shrink: 0; }
  .speed-slider-control { flex: 1; height: 6px; border-radius: 3px; background: var(--bg-surface-3, rgba(255, 255, 255, 0.15)); accent-color: var(--accent); cursor: pointer; outline: none; }

  /* Color group box matching SettingsPanel (Cài đặt màu nền) */
  .color-group-box {
    background: var(--bg-surface-2, rgba(255, 255, 255, 0.04));
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    padding: 14px 16px;
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
  .cg-title-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cg-title-group strong {
    font-size: 13px;
    font-weight: 650;
    color: var(--text-primary);
  }
  .cg-title-group small {
    font-size: 11px;
    color: var(--text-secondary);
  }
  .color-picker-pill {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--bg-surface-3, rgba(255, 255, 255, 0.08));
    border: 1px solid var(--border-strong, rgba(255, 255, 255, 0.18));
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
    border: 1px solid rgba(255, 255, 255, 0.4);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  }
  .skin-preset-pills {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .swatch-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 2px 0;
  }
  .color-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--bg-surface-3, rgba(255, 255, 255, 0.08));
    padding: 3px;
    cursor: pointer;
    transition: transform 0.15s ease, border-color 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .color-swatch span {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: var(--sw-color);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
  }
  .color-swatch:hover {
    transform: scale(1.15);
  }
  .color-swatch.active {
    border-color: var(--accent);
    transform: scale(1.1);
    box-shadow: 0 0 8px var(--accent-soft, rgba(168, 85, 247, 0.4));
  }
  .control-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .slider-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .ctrl-label {
    font-size: 11.5px;
    color: var(--text-secondary);
    font-weight: 500;
  }
  .ctrl-val {
    font-family: var(--font-mono, monospace);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .hue-slider {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: linear-gradient(
      to right,
      #ef4444 0%,
      #f97316 17%,
      #eab308 33%,
      #22c55e 50%,
      #06b6d4 67%,
      #3b82f6 83%,
      #a855f7 92%,
      #ef4444 100%
    );
    cursor: pointer;
    appearance: none;
    -webkit-appearance: none;
    outline: none;
  }
  .hue-slider::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #ffffff;
    border: 2px solid #000000;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
    cursor: pointer;
    transition: transform 0.15s ease;
  }
  .hue-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  @media (max-width: 620px) {
    .lyrics-settings { padding: 16px; }
    .window-mode-grid { grid-template-columns: 1fr; }
    .settings-tabs { width: 100%; }
    .settings-tab { flex: 1; justify-content: center; }
  }
</style>
