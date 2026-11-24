<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    X, Search, Plus, ListMusic, ArrowUpToLine, SkipForward, SkipBack, Pause, Play,
    Maximize, Minimize, Music2, Disc3, ChevronLeft, Eye, EyeOff, TextAlignStart,
    RotateCw, Settings
  } from '@lucide/svelte';
  import { api, fmtTime, getTrackCover, coverSrc, onlineToTrack, type Track, type LyricsResult } from '$lib/api';
  import { player } from '$lib/player.svelte';
  import { lyricsState } from '$lib/lyrics.svelte';
  import { parseLrc, activeIndex, wordFillPercent, activeWordIndex as getActiveWordIdx } from '$lib/lrc';
  import { theme } from '$lib/theme.svelte';
  import { lyricsUi, type LyricsLayoutMode, type VinylSkin, type LyricsFontSize, type LyricsDimInactive, type LyricsBgStyle } from '$lib/lyricsUiConfig.svelte';
  import LyricsPanel from '$lib/LyricsPanel.svelte';

  let {
    onclose,
    onsettings,
    embedded = false,
    ontogglemaximize,
  }: {
    onclose: () => void;
    onsettings?: () => void;
    embedded?: boolean;
    ontogglemaximize?: () => void;
  } = $props();
  let dialog = $state<HTMLElement>();
  let query = $state('');
  let source = $state<'library' | 'beat' | 'vocal'>('library');
  let panel = $state<'search' | 'queue' | 'lyrics'>('search');
  let deskOpen = $state(false);
  let controlsVisible = $state(true);
  let lyricBox = $state<HTMLDivElement>();
  let results = $state<Track[]>([]);
  let searching = $state(false);
  let searched = $state(false);
  let searchError = $state('');
  let notice = $state('');
  let replacementPath = $state<string | null>(null);
  let lyricResult = $state<LyricsResult | null>(null);
  let lyricLoading = $state(false);
  let lyricError = $state('');
  let lyricRetry = $state(0);
  let position = $state(0);
  let fullscreen = $state(false);
  let ownsFullscreen = false;
  let searchToken = 0;

  const current = $derived(player.current);
  const lyrics = $derived(lyricResult ? parseLrc(lyricResult.content) : null);
  const active = $derived(lyrics?.synced ? activeIndex(lyrics.lines, position) : -1);
  const line = $derived(lyrics?.synced && active >= 0 ? lyrics.lines[active] : null);
  const nextLine = $derived(lyrics?.synced ? lyrics.lines[active + 1] : null);
  const prevLine = $derived(lyrics?.synced && active > 0 ? lyrics.lines[active - 1] : null);
  const upcoming = $derived(player.queue.map((track, index) => ({ track, index })).filter(item => item.index > player.index));
  const cover = $derived(current ? getTrackCover(current.cover_path, current.url ?? current.path, current.title, current.artist) : '');
  const backdrop = $derived(theme.config.bgImage ? coverSrc(theme.config.bgImage) : cover);
  const duration = $derived(Number.isFinite(player.duration) ? Math.max(0, player.duration) : 0);
  const progress = $derived(duration ? Math.min(100, Math.max(0, position / duration * 100)) : 0);

  // Tonearm rotation angle based on playback progress
  let isDraggingArm = $state(false);
  let dragArmAngle = $state<number | null>(null);
  let tonearmBaseEl = $state<HTMLElement>();

  // Tốc độ quay đĩa than tính theo giây từ RPM
  const spinDuration = $derived.by(() => {
    const rpm = lyricsUi.config.vinylSpeedRpm || (lyricsUi.config.vinylSpeed === '45' ? 45 : 33);
    return `${(60 / Math.max(15, rpm)).toFixed(2)}s`;
  });

  // Góc xoay của tay cần (kết hợp cả kéo chuột và góc tự động theo tiến độ nhạc)
  const currentTonearmAngle = $derived.by(() => {
    if (dragArmAngle !== null) return dragArmAngle;
    if (!player.playing) return -10; // Vị trí nghỉ (rest) ngoài đĩa
    // Quét từ 14 độ vào 26 độ theo tiến độ bài hát
    return 14 + (progress / 100) * 12;
  });

  let hasMovedArm = false;
  let cleanupArmDrag: (() => void) | null = null;

  function handleArmMouseDown(e: MouseEvent | TouchEvent) {
    if (!lyricsUi.config.showTonearm) return;
    e.preventDefault();
    isDraggingArm = true;
    hasMovedArm = false;

    const startX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const startY = 'touches' in e ? e.touches[0].clientY : e.clientY;

    const baseRect = tonearmBaseEl?.getBoundingClientRect();
    const pivotX = baseRect ? baseRect.left + baseRect.width * 0.5385 : startX;
    const pivotY = baseRect ? baseRect.top + baseRect.height * 0.125 : startY;

    const initialAngle = currentTonearmAngle;
    dragArmAngle = initialAngle;

    function onMouseMove(ev: MouseEvent | TouchEvent) {
      const curX = 'touches' in ev ? ev.touches[0].clientX : ev.clientX;
      const curY = 'touches' in ev ? ev.touches[0].clientY : ev.clientY;
      const dist = Math.hypot(curX - startX, curY - startY);

      if (dist > 5) {
        hasMovedArm = true;
      }

      // Hướng vector từ trục xoay (pivot) đến con trỏ chuột
      // Khi gạt sang bên trái (vào đĩa than), pivotX - curX > 0 => góc dương tăng theo hướng lòng đĩa
      const targetAngle = Math.atan2(pivotX - curX, curY - pivotY) * (180 / Math.PI);

      // Giới hạn góc từ -14 độ (bệ nghỉ) đến 32 độ (vào lòng đĩa)
      dragArmAngle = Math.min(32, Math.max(-14, targetAngle));
    }

    function onMouseUp() {
      cleanup();

      const finalAngle = dragArmAngle ?? initialAngle;
      isDraggingArm = false;
      dragArmAngle = null;

      if (!hasMovedArm) {
        // Nhấp chuột vào tay cần: Bật/Tắt (Play/Pause)
        player.toggle();
        return;
      }

      // Kéo gạt tay cần:
      // Góc >= 5 độ: Đã đẩy vào phía đĩa than -> Bật nhạc (Play)
      // Góc < 5 độ: Đã kéo ra ngoài bệ đỡ -> Dừng nhạc (Pause)
      if (finalAngle >= 5) {
        player.play();
      } else {
        player.pause();
      }
    }

    function cleanup() {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
      window.removeEventListener('touchmove', onMouseMove);
      window.removeEventListener('touchend', onMouseUp);
      cleanupArmDrag = null;
    }

    cleanupArmDrag = cleanup;
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    window.addEventListener('touchmove', onMouseMove, { passive: false });
    window.addEventListener('touchend', onMouseUp);
  }

  // Xoá bỏ nhãn đoạn (intro, verse, hook...) và lọc các từ nhãn
  function cleanLineText(text: string): string {
    return text
      .replace(/^\s*[\[(]?(verse|chorus|hook|drop|outro|intro|bridge|pre-chorus)[\s\d]*[\]):-]*\s*/i, '')
      .trim();
  }

  function getDisplayWords(words: any[]): any[] {
    if (!words) return [];
    return words.filter(w => !/^[\[(]?(verse|chorus|hook|drop|outro|intro|bridge|pre-chorus)[\s\d]*[\]):-]*$/i.test(w.text.trim()));
  }

  // Danh sách từ hiển thị sau khi đã lọc sạch các thẻ [intro], [verse]...
  const currentDisplayWords = $derived(getDisplayWords(line?.words ?? []));

  // Calculate active word index in current line dựa trên chính xác danh sách từ hiển thị
  const activeWordIndex = $derived.by(() => {
    if (!currentDisplayWords || currentDisplayWords.length === 0) return -1;
    return getActiveWordIdx(currentDisplayWords, position);
  });

  onMount(() => {
    const previous = document.activeElement as HTMLElement | null;
    if (!embedded && dialog && (dialog as HTMLDialogElement).showModal) {
      (dialog as HTMLDialogElement).showModal();
    }
    fullscreen = Boolean(document.fullscreenElement);
    player.setKaraoke(true);
    const unsubscribe = lyricsState.subscribe();
    return () => {
      searchToken++;
      unsubscribe();
      player.setKaraoke(false);
      // Dọn dẹp sự kiện kéo cần đĩa nếu đang kéo dở khi đóng
      cleanupArmDrag?.();
      if (ownsFullscreen && document.fullscreenElement) void document.exitFullscreen().catch(() => {});
      if (!embedded && dialog && (dialog as HTMLDialogElement).close) {
        (dialog as HTMLDialogElement).close();
      }
      previous?.focus();
    };
  });

  $effect(() => {
    const selected = current;
    void lyricsState.revision;
    void lyricRetry;
    let disposed = false;
    lyricResult = null; lyricError = ''; lyricLoading = Boolean(selected);
    if (selected) api.readLyrics(selected).then(value => {
      if (!disposed) lyricResult = value;
    }).catch(error => {
      if (!disposed) lyricError = String(error);
    }).finally(() => { if (!disposed) lyricLoading = false; });
    return () => { disposed = true; };
  });

  $effect(() => {
    position = player.position;
    if (!player.playing) return;
    let frame = 0;
    const update = () => {
      position = player.audio?.currentTime ?? player.position;
      frame = requestAnimationFrame(update);
    };
    frame = requestAnimationFrame(update);
    return () => cancelAnimationFrame(frame);
  });

  $effect(() => {
    void controlsVisible;
    if (lyricsUi.config.layout !== 'full' || active < 0 || !lyricBox || deskOpen) return;
    lyricBox.querySelector<HTMLElement>(`[data-stage-line="${active}"]`)?.scrollIntoView({
      block: 'center', behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth',
    });
  });

  async function toggleControls() {
    controlsVisible = !controlsVisible;
    deskOpen = false;
    await tick();
    dialog?.querySelector<HTMLButtonElement>(controlsVisible ? '.hide-controls' : '.show-controls')?.focus();
  }

  function openLyrics() { panel = 'lyrics'; deskOpen = true; }

  async function search() {
    const mine = ++searchToken;
    const text = query.trim();
    searchError = ''; searched = false; results = []; searching = true;
    try {
      const found = source === 'library'
        ? await api.listTracks(text || undefined)
        : text ? (await api.searchOnline(`${text} ${source === 'beat' ? 'karaoke beat không lời' : 'official audio'}`, 'youtube', 18)).map(onlineToTrack) : [];
      if (mine === searchToken) { results = found; searched = true; }
    } catch (error) { if (mine === searchToken) searchError = `Không tìm được bài: ${String(error)}. Hãy thử lại.`; }
    finally { if (mine === searchToken) searching = false; }
  }

  function changeSource(value: typeof source) {
    searchToken++; searching = false; results = []; searched = false; searchError = '';
    source = value;
  }

  async function chooseVersion(value: 'beat' | 'vocal') {
    if (!current) return;
    query = `${current.title} ${current.artist}`;
    replacementPath = current.path;
    changeSource(value); panel = 'search'; deskOpen = true;
    await tick();
    await search();
  }

  function add(track: Track) {
    player.enqueue(track);
    notice = `Đã gọi bài: ${track.title}`;
  }

  function replace(track: Track) {
    if (!current || current.path !== replacementPath) return;
    player.replaceCurrent(track);
    replacementPath = null;
    notice = 'Đã đổi nguồn. Bản mới phát từ đầu vì mốc lời và bản thu có thể khác.';
    deskOpen = false;
  }

  async function toggleFullscreen() {
    try {
      if (document.fullscreenElement) { await document.exitFullscreen(); ownsFullscreen = false; }
      else { await document.documentElement.requestFullscreen(); ownsFullscreen = true; }
    } catch { notice = 'Thiết bị không hỗ trợ toàn màn hình. Phòng hát vẫn dùng toàn bộ cửa sổ ứng dụng.'; }
  }
</script>

<svelte:document onfullscreenchange={() => fullscreen = Boolean(document.fullscreenElement)} />

<svelte:element
  this={embedded ? 'section' : 'dialog'}
  class="karaoke-room"
  class:embedded={embedded}
  class:focused={!controlsVisible}
  class:is-fullscreen={fullscreen}
  class:bg-apple-glass={lyricsUi.config.bgStyle === 'apple-glass'}
  class:bg-glow={lyricsUi.config.bgStyle === 'glow'}
  class:bg-oled={lyricsUi.config.bgStyle === 'oled'}
  class:bg-wallpaper={lyricsUi.config.bgStyle === 'wallpaper'}
  class:font-compact={lyricsUi.config.fontSize === 'compact'}
  class:font-large={lyricsUi.config.fontSize === 'large'}
  class:dim-soft={lyricsUi.config.dimInactive === 'soft'}
  class:dim-hidden={lyricsUi.config.dimInactive === 'hidden'}
  bind:this={dialog}
  aria-label="Studio Lời bài hát & Karaoke"
  oncancel={(event: Event) => { event.preventDefault(); onclose(); }}
>
  <div class="room-background" aria-hidden="true">
    {#if theme.config.bgImage}
      <!-- Nền tùy biến khớp 100% với cài đặt giao diện đã điều chỉnh -->
      <img
        class="stage-backdrop custom-wallpaper"
        class:is-blurred={(theme.config.bgImageBlur ?? 0) > 0}
        src={coverSrc(theme.config.bgImage)}
        alt=""
      />
      <div class="backdrop-tint custom-tint"></div>
    {:else if lyricsUi.config.bgStyle === 'glow'}
      <div class="glow-orb glow-1" style:background={cover ? `url(${cover})` : 'var(--accent)'}></div>
      <div class="glow-orb glow-2" style:background={cover ? `url(${cover})` : 'var(--accent)'}></div>
      <div class="backdrop-tint"></div>
    {:else if backdrop}
      <img class="stage-backdrop album-backdrop" src={backdrop} alt="" />
      <div class="backdrop-tint" class:glass-tint={lyricsUi.config.bgStyle === 'apple-glass'}></div>
    {:else}
      <div class="backdrop-tint"></div>
    {/if}
  </div>

  {#if controlsVisible}
  <header class="room-header">
    <div class="room-name">
      <Music2 size={22} />
      <h1 id="karaoke-title">Studio Lời bài hát</h1>
    </div>

    <div class="room-actions">
      <button class="desk-toggle" onclick={() => deskOpen = !deskOpen} aria-expanded={deskOpen} aria-controls="karaoke-desk">
        <ListMusic size={16} />
        <span>{deskOpen ? 'Sân khấu' : `Gọi bài · ${upcoming.length}`}</span>
      </button>

      <button class="icon-button hide-controls" onclick={toggleControls} aria-label="Ẩn điều khiển" title="Chỉ hiện đĩa than và lời"><EyeOff size={18} /></button>
      {#if ontogglemaximize}
        <button
          type="button"
          class="icon-button"
          onclick={ontogglemaximize}
          aria-label={embedded ? 'Phóng to toàn bộ cửa sổ' : 'Thu nhỏ về giao diện bên phải'}
          title={embedded ? 'Phóng to toàn bộ cửa sổ' : 'Thu nhỏ về giao diện bên phải'}
        >
          {#if embedded}<Maximize size={18} />{:else}<Minimize size={18} />{/if}
        </button>
      {:else}
        <button class="icon-button" onclick={toggleFullscreen} aria-label={fullscreen ? 'Thoát toàn màn hình' : 'Toàn màn hình'}>
          {#if fullscreen}<Minimize size={18} />{:else}<Maximize size={18} />{/if}
        </button>
      {/if}
      {#if onsettings}
        <button class="icon-button" onclick={onsettings} aria-label="Cài đặt lời bài hát" title="Cài đặt"><Settings size={18} /></button>
      {/if}
      <button class="icon-button" onclick={onclose} aria-label="Đóng phòng lời bài hát" title="Đóng · Esc"><X size={20} /></button>
    </div>
  </header>
  {:else}
    <button class="show-controls" onclick={toggleControls} aria-label="Hiện điều khiển" title="Hiện điều khiển"><Eye size={18} /><span>Điều khiển</span></button>
  {/if}

  <div class="room-body" class:show-desk={deskOpen && controlsVisible}>
    <main class="singing-stage" aria-label="Sân khấu lời bài hát">
      {#if lyricsUi.config.layout === 'vinyl'}
        <!-- ======================= Chế độ ĐĨA THAN PHÒNG THU (như video 616.mp4) ======================= -->
        <div class="vinyl-studio-view">
          <!-- Left side: Vinyl Turntable & Tonearm -->
          <div class="turntable-dock">
            <div
              class="vinyl-assembly"
              class:spinning={player.playing}
              style:--spin-duration={spinDuration}
            >
              <!-- Vinyl Record Platter -->
              <div
                class="vinyl-record"
                class:skin-marble={lyricsUi.config.vinylSkin === 'marble'}
                class:skin-album={lyricsUi.config.vinylSkin === 'album'}
                class:skin-black={lyricsUi.config.vinylSkin === 'black'}
                class:skin-cyberpunk={lyricsUi.config.vinylSkin === 'cyberpunk'}
                class:skin-gold={lyricsUi.config.vinylSkin === 'gold'}
                class:skin-custom={lyricsUi.config.vinylSkin === 'custom'}
                style:--vinyl-custom-color={lyricsUi.config.vinylColorHex || `hsl(${lyricsUi.config.vinylHue ?? 240}, 75%, 32%)`}
              >
                <!-- 1. Full Cover nếu chọn 'full' hoặc skin-album -->
                {#if cover && (lyricsUi.config.vinylCoverPos === 'full' || (lyricsUi.config.vinylSkin === 'album' && lyricsUi.config.vinylCoverPos !== 'rim'))}
                  <img class="vinyl-album-art" src={cover} alt="" />
                {/if}

                <!-- 2. Rim Cover nếu chọn 'rim' -->
                {#if cover && lyricsUi.config.vinylCoverPos === 'rim'}
                  <div class="vinyl-rim-art" style:background-image={`url(${cover})`}></div>
                {/if}

                <div class="groove-reflection"></div>

                <!-- Center Spindle & Label (Tâm đĩa chuẩn nét như ảnh mẫu với vòng tròn nét đứt và chốt sáng) -->
                <div class="center-label">
                  {#if cover && (lyricsUi.config.vinylCoverPos === 'center' || (!lyricsUi.config.vinylCoverPos && lyricsUi.config.vinylSkin !== 'album'))}
                    <img class="center-label-art" src={cover} alt="" />
                    <div class="center-label-tint"></div>
                  {:else}
                    <div class="center-label-fallback">
                      <div class="center-vinyl-icon">
                        <Disc3 size={36} />
                      </div>
                    </div>
                  {/if}

                  <!-- Vòng tròn nét đứt tinh tế chuẩn như ảnh mẫu -->
                  <div class="center-dashed-ring"></div>

                  <!-- Chốt tâm trục phát sáng 2 tầng như ảnh -->
                  <div class="spindle-hub">
                    <div class="spindle-accent-ring"></div>
                    <div class="spindle-hole"></div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Tonearm Assembly with Mouse Drag & Click to Play/Pause (Thiết kế chuẩn theo ảnh mẫu) -->
            {#if lyricsUi.config.showTonearm}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="tonearm-assembly"
                class:is-dragging={isDraggingArm}
                class:arm-playing={player.playing}
                style:transform={`rotate(${currentTonearmAngle}deg)`}
                bind:this={tonearmBaseEl}
                onmousedown={handleArmMouseDown}
                ontouchstart={handleArmMouseDown}
                onkeydown={(e) => {
                  if (e.key === ' ' || e.key === 'Enter') {
                    e.preventDefault();
                    player.toggle();
                  }
                }}
                title={player.playing ? 'Bấm hoặc gạt cần ra để Tạm dừng' : 'Bấm hoặc gạt cần vào để Phát nhạc'}
                role="button"
                tabindex="0"
                aria-label="Tay cần đĩa than — Bấm hoặc gạt để Bật/Tắt nhạc"
              >
                <svg viewBox="0 0 130 440" class="tonearm-svg" width="130" height="440">
                  <defs>
                    <linearGradient id="arm-chrome" x1="0%" y1="0%" x2="100%" y2="0%">
                      <stop offset="0%" stop-color="#94a3b8" />
                      <stop offset="35%" stop-color="#ffffff" />
                      <stop offset="65%" stop-color="#cbd5e1" />
                      <stop offset="100%" stop-color="#475569" />
                    </linearGradient>
                    <radialGradient id="base-metal" cx="35%" cy="30%" r="70%">
                      <stop offset="0%" stop-color="#f1f5f9" />
                      <stop offset="50%" stop-color="#cbd5e1" />
                      <stop offset="100%" stop-color="#64748b" />
                    </radialGradient>
                    <radialGradient id="pin-shine" cx="35%" cy="30%" r="65%">
                      <stop offset="0%" stop-color="#ffffff" />
                      <stop offset="55%" stop-color="#cbd5e1" />
                      <stop offset="100%" stop-color="#94a3b8" />
                    </radialGradient>
                    <filter id="arm-drop-shadow" x="-30%" y="-10%" width="180%" height="130%">
                      <feDropShadow dx="3" dy="8" stdDeviation="5" flood-color="#000000" flood-opacity="0.45" />
                    </filter>
                  </defs>

                  <!-- Tube & Cartridge with shadow -->
                  <g filter="url(#arm-drop-shadow)">
                    <!-- J-shaped Chrome Tube cong chữ J tuyệt đẹp như ảnh -->
                    <path
                      d="M 70 55 L 70 275 C 70 325 56 350 36 370"
                      fill="none"
                      stroke="url(#arm-chrome)"
                      stroke-width="7"
                      stroke-linecap="round"
                    />

                    <!-- Collar & Cartridge Head (Đầu kim đĩa than màu xám đen bo góc như ảnh) -->
                    <g transform="translate(36, 370) rotate(-35)">
                      <!-- Metallic connector collar -->
                      <rect x="-5" y="-2" width="10" height="11" rx="2" fill="url(#base-metal)" stroke="#475569" stroke-width="0.8" />
                      <!-- Dark cartridge body -->
                      <rect x="-8.5" y="9" width="17" height="36" rx="4" fill="#1e2430" stroke="#334155" stroke-width="1.2" />
                      <!-- Stylus dot on bottom-left -->
                      <circle cx="-3.5" cy="40" r="2" fill="#94a3b8" />
                      <!-- Finger-lift lever on the right side -->
                      <path d="M 8.5 17 Q 17 17 17 26" fill="none" stroke="#cbd5e1" stroke-width="2" stroke-linecap="round" />
                    </g>
                  </g>

                  <!-- Interactive wider hit-area for easy mouse grabbing -->
                  <path
                    d="M 70 55 L 70 275 C 70 325 56 350 36 370"
                    fill="none"
                    stroke="transparent"
                    stroke-width="48"
                  />
                  <rect x="10" y="340" width="55" height="80" fill="transparent" />

                  <!-- Pivot Base (Bệ trục xoay kim loại ở góc trên bên phải) -->
                  <g>
                    <!-- Outer metallic rim -->
                    <circle cx="70" cy="55" r="30" fill="url(#base-metal)" stroke="rgba(255,255,255,0.8)" stroke-width="1.5" />
                    <!-- Recessed dark ring -->
                    <circle cx="70" cy="55" r="19" fill="#1b202c" stroke="#334155" stroke-width="1.5" />
                    <!-- Center pivot dome/pin -->
                    <circle cx="70" cy="55" r="10" fill="url(#pin-shine)" stroke="rgba(255,255,255,0.9)" stroke-width="1" />
                  </g>
                </svg>
              </div>
            {/if}
          </div>

          <!-- Right side: Lyrics Flow & Track Heading -->
          <div class="vinyl-lyrics-pane" bind:this={lyricBox}>
            <!-- Top Right Track Meta with status dot -->
            <div class="song-header-right">
              <div class="meta-texts">
                <h2 class="vh-title">{current ? current.title : 'Chưa chọn bài hát'}</h2>
                <p class="vh-artist">{current ? current.artist : 'Tran Tra My'}</p>
              </div>
              <div class="neon-status-dot" class:playing={player.playing} title={player.playing ? 'Đang phát' : 'Tạm dừng'}></div>
            </div>

            <!-- Lyric Stream (Curved & Synchronized) -->
            <div class="stream-content">
              {#if !current}
                <div class="empty-state">
                  <Music2 size={44} />
                  <h3>Sân khấu đang chờ bạn</h3>
                  <p>Hãy phát một bản nhạc hoặc mở Gọi bài để bắt đầu.</p>
                </div>
              {:else if lyricLoading}
                <div class="empty-state">
                  <RotateCw class="spinning-icon" size={28} />
                  <p>Đang đồng bộ lời bài hát…</p>
                </div>
              {:else if lyricError}
                <div class="empty-state">
                  <p class="error-msg">{lyricError}</p>
                  <button class="retry-btn" onclick={() => lyricRetry++}>Tải lại lời</button>
                </div>
              {:else if !lyrics?.lines.length}
                <div class="empty-state">
                  <h3>Hát theo giai điệu của bạn</h3>
                  <p>Bản này chưa có lời đồng bộ. Mở Lời bài hát để thêm hoặc tạo tự động.</p>
                  <button class="open-lyrics-btn" onclick={openLyrics}>Mở trình tạo lời</button>
                </div>
              {:else if !lyrics.synced}
                <div class="plain-scroll-box">
                  <pre class="plain-pre">{lyrics.lines.map(r => r.text).join('\n\n')}</pre>
                </div>
              {:else}
                <!-- Curved Studio Lyrics Stream -->
                <div class="curved-lyric-flow">
                  <!-- Previous Line (Dimmed, curving away) -->
                  {#if prevLine}
                    <p class="flow-line line-prev" aria-hidden="true">
                      <span class="flow-text">{cleanLineText(prevLine.text) || '\u00a0'}</span>
                    </p>
                  {:else}
                    <p class="flow-line line-prev placeholder" aria-hidden="true">&nbsp;</p>
                  {/if}

                  <!-- Current Active Line -->
                  <div class="flow-line line-current" aria-label={line?.text || 'Nhạc dạo'}>
                    <div class="words-container">
                      {#if line?.text}
                        {#each currentDisplayWords as word, wIdx}
                          {@const isWordActive = wIdx === activeWordIndex}
                          <span
                            class="sung-word"
                            class:active-word={isWordActive}
                            class:done={wordFillPercent(word, position) >= 100}
                            style:--fill={`${wordFillPercent(word, position)}%`}
                          >
                            <span class="word-chars">{word.text}</span>
                          </span>{#if wIdx < currentDisplayWords.length - 1}{' '}{/if}
                        {/each}
                      {:else}
                        <span class="instrumental-pill">♪ Nhạc dạo ♪</span>
                      {/if}
                    </div>
                  </div>

                  <!-- Next Line (Dimmed, anticipating) -->
                  {#if nextLine}
                    <p class="flow-line line-next">
                      <span class="flow-text">{cleanLineText(nextLine.text) || (active >= lyrics.lines.length - 1 ? 'Cảm ơn bạn đã nghe cùng Trantramy' : '')}</span>
                    </p>
                  {:else}
                    <p class="flow-line line-next placeholder">&nbsp;</p>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- Mini Player Bar under lyrics -->
            {#if controlsVisible}
            <div class="vinyl-mini-controls">
              <div class="mini-timeline">
                <span class="time-num">{fmtTime(position)}</span>
                <div class="seek-track" style:--progress={`${progress}%`}>
                  <input
                    type="range"
                    min="0"
                    max={duration || 1}
                    step="0.1"
                    value={position}
                    disabled={!current || !duration}
                    aria-label="Vị trí bài hát"
                    oninput={e => player.seek(Number(e.currentTarget.value))}
                  />
                </div>
                <span class="time-num">{fmtTime(duration)}</span>
              </div>

              <div class="mini-transport-bar">
                <div class="mini-actions-left">
                  <button class="text-pill-btn" disabled={!current} onclick={() => chooseVersion('beat')}>Tìm bản beat</button>
                  <button class="text-pill-btn" disabled={!current} onclick={() => chooseVersion('vocal')}>Bản có giọng</button>
                  <button class="text-pill-btn" disabled={!current} onclick={openLyrics}><TextAlignStart size={13} /> Sửa lời</button>
                </div>
                <div class="mini-transport-buttons">
                  <button class="play-btn-circle" disabled={!current || player.loadingStream} onclick={() => player.toggle()} aria-label={player.playing ? 'Tạm dừng' : 'Phát'}>
                    {#if player.playing}<Pause size={20} />{:else}<Play size={20} />{/if}
                  </button>
                  <button class="next-btn-circle" disabled={!current} onclick={() => player.next()} aria-label="Bỏ qua"><SkipForward size={18} /></button>
                </div>
              </div>
            </div>
            {/if}
          </div>
        </div>

      {:else if lyricsUi.config.layout === 'stage'}
        <!-- ======================= Chế độ SÂN KHẤU KARAOKE (Center Stage) ======================= -->
        {#if controlsVisible}
        <div class="song-heading">
          {#if current}
            <img class="cover" class:spinning={player.playing} src={cover} alt="" />
            <div class="song-meta"><h2>{current.title}</h2><p>{current.artist}</p></div>
          {:else}
            <Disc3 size={36} /><div class="song-meta"><h2>Sân khấu đang chờ bạn</h2><p>Chọn bài để bắt đầu.</p></div>
          {/if}
        </div>
        {/if}

        <div class="lyric-stage" bind:this={lyricBox}>
          {#if !current}
            <Music2 class="idle-icon" size={44} /><h3>Bài đầu tiên, bạn chọn nhé.</h3><p>Mở Gọi bài, tìm bài yêu thích và thêm vào lượt hát.</p>
          {:else if lyricLoading}
            <p role="status">Đang tải lời bài hát…</p>
          {:else if lyricError}
            <p role="alert">Không tải được lời bài hát.</p><p class="lyric-note">{lyricError}</p><button onclick={() => lyricRetry++}>Tải lại lời</button>
          {:else if !lyrics?.lines.length}
            <h3>Hát theo giai điệu của bạn</h3><p>Bản này chưa có lời. Bạn vẫn có thể phát nhạc hoặc chọn nguồn khác.</p>
            {#if controlsVisible}<button onclick={openLyrics}>Tìm hoặc thêm lời bài hát</button>{/if}
          {:else if !lyrics.synced}
            <textarea class="plain-lyrics" readonly aria-label="Lời chưa có mốc thời gian" value={lyrics.lines.map(row => row.text).join('\n\n')}></textarea>
          {:else}
            <p class="previous-line" aria-hidden="true">{prevLine ? prevLine.text : '\u00a0'}</p>
            <div class="current-line" aria-label={line?.text || 'Nhạc dạo'}>
              {#if line?.text}
                {#each currentDisplayWords as word, wIdx}
                  {@const isWordActive = wIdx === activeWordIndex}
                  {@const fill = wordFillPercent(word, position)}
                  <span class="sung-word" class:active-word={isWordActive} class:done={fill >= 100} style:--fill={`${fill}%`}>
                    <span class="word-chars">{word.text}</span>
                  </span>{#if wIdx < currentDisplayWords.length - 1}{' '}{/if}
                {/each}
              {:else}<span class="instrumental">♪ Nhạc dạo ♪</span>{/if}
            </div>
            <p class="next-line">{nextLine?.text || (active >= lyrics.lines.length - 1 ? 'Cảm ơn bạn đã hát cùng Trantramy.' : '')}</p>
          {/if}
        </div>

        {#if controlsVisible}
        <footer class="stage-controls">
          <div class="timeline"><span>{fmtTime(position)}</span><div class="seek-track" style:--progress={`${progress}%`}><input type="range" min="0" max={duration || 1} step="0.1" value={position} disabled={!current || !duration} aria-label="Vị trí bài karaoke" oninput={e => player.seek(Number(e.currentTarget.value))} /></div><span>{fmtTime(duration)}</span></div>
          <div class="transport-row">
            <div class="version-controls"><button disabled={!current} onclick={() => chooseVersion('beat')}>Tìm bản beat</button><button disabled={!current} onclick={() => chooseVersion('vocal')}>Tìm bản có giọng</button><button disabled={!current} onclick={openLyrics}><TextAlignStart size={15} /> Sửa lời</button></div>
            <div class="transport"><button class="play-button" disabled={!current || player.loadingStream} onclick={() => player.toggle()} aria-label={player.playing ? 'Tạm dừng' : 'Phát'}>{#if player.playing}<Pause size={24} />{:else}<Play size={24} />{/if}</button><button class="icon-button" disabled={!current} onclick={() => player.next()} aria-label="Bỏ qua"><SkipForward size={22} /></button></div>
          </div>
        </footer>
        {/if}

      {:else}
        <!-- ======================= Chế độ CUỘN TOÀN BỘ LỜI (Full Flowing) ======================= -->
        <div class="full-lyrics-container" bind:this={lyricBox}>
          {#if lyrics?.lines.length}
            <div class="full-lines-scroll">
              {#each lyrics.lines as row, index}
                <button
                  type="button"
                  class="full-lyric-line"
                  class:active={index === active}
                  data-stage-line={index}
                  onclick={() => player.seek(row.time)}
                >
                  <span class="full-text">{cleanLineText(row.text) || '♪ Nhạc dạo ♪'}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </main>

    <!-- Side Songbook / Lyrics Editor Drawer -->
    {#if deskOpen && controlsVisible}
    <aside id="karaoke-desk" class="songbook" aria-label="Gọi bài và lời bài hát">
      <div class="book-tabs">
        <button class:chosen={panel === 'search'} aria-pressed={panel === 'search'} onclick={() => panel = 'search'}><Search size={16} />Gọi bài</button>
        <button class:chosen={panel === 'queue'} aria-pressed={panel === 'queue'} onclick={() => panel = 'queue'}><ListMusic size={16} />Chờ hát <span class="count">{upcoming.length}</span></button>
        <button class:chosen={panel === 'lyrics'} aria-pressed={panel === 'lyrics'} onclick={() => panel = 'lyrics'}><TextAlignStart size={16} />Sửa lời</button>
      </div>

      {#if panel === 'search'}
        <form class="song-search" onsubmit={e => { e.preventDefault(); replacementPath = null; void search(); }}>
          <label for="karaoke-search">Bạn muốn hát bài gì?</label>
          <div class="search-field"><input id="karaoke-search" bind:value={query} aria-label="Tên bài karaoke" placeholder="Tên bài hát, ca sĩ…" /><button type="submit" class="icon-button" aria-label="Tìm bài"><Search size={18} /></button></div>
          <div class="source-choice" role="group" aria-label="Nguồn bài karaoke">{#each [{ id: 'library', label: 'Thư viện' }, { id: 'beat', label: 'Beat karaoke' }, { id: 'vocal', label: 'Có giọng' }] as option}<button type="button" class:chosen={source === option.id} aria-pressed={source === option.id} onclick={() => changeSource(option.id as typeof source)}>{option.label}</button>{/each}</div>
          <p class="search-note">{source === 'library' ? 'Tìm trong nhạc đã thêm vào thư viện.' : 'Tìm trên YouTube trực tuyến.'}</p>
        </form>
        {#if replacementPath && current?.path === replacementPath}<div class="replacement-note"><p>Chọn nguồn thay cho bài đang hát. Bản mới sẽ phát từ đầu.</p><button onclick={() => replacementPath = null}>Hủy đổi bản</button></div>{/if}
        <div class="book-list" aria-busy={searching}>
          {#if searching}<p class="book-empty" role="status">Đang tìm bài…</p>
          {:else if searchError}<p class="book-empty" role="alert">{searchError}</p><button class="retry" onclick={search}>Thử lại</button>
          {:else if results.length}
            {#each results as track, index (index)}
              <div class="song-row"><div class="result-meta"><strong>{track.title}</strong><span>{track.artist} · {fmtTime(track.duration)}</span></div><div class="result-actions">{#if replacementPath && current?.path === replacementPath}<button onclick={() => replace(track)} aria-label={`Dùng bản ${track.title}`}>Dùng bản này</button>{:else}<button class="icon-button" onclick={() => add(track)} aria-label={`Gọi bài ${track.title}`} title="Thêm vào lượt hát"><Plus size={20} /></button>{/if}</div></div>
            {/each}
          {:else}<div class="book-empty"><Music2 size={28} /><p>{searched ? 'Không tìm thấy bài phù hợp.' : 'Nhập tên bài để tìm và gọi bài.'}</p>{#if !searched && source === 'library'}<button onclick={search}>Xem thư viện</button>{/if}</div>{/if}
        </div>
      {:else if panel === 'lyrics'}
        <div class="room-lyrics">
          <LyricsPanel track={current ?? undefined} embedded onsettings={onsettings ? () => { onclose(); onsettings?.(); } : undefined} />
        </div>
      {:else}
        <div class="queue-heading"><h2>Lượt hát tiếp theo</h2><p>Hát theo thứ tự danh sách chờ.</p></div>
        <div class="book-list">
          {#each upcoming as { track, index }, order (index)}
            <div class="song-row"><span class="queue-number">{String(order + 1).padStart(2, '0')}</span><div class="result-meta"><strong>{track.title}</strong><span>{track.artist}</span></div><div class="result-actions"><button class="icon-button" disabled={order === 0} onclick={() => player.moveNext(index)} aria-label={`Ưu tiên ${track.title}`} title="Hát tiếp theo"><ArrowUpToLine size={17} /></button><button class="icon-button" onclick={() => player.removeFromQueue(index)} aria-label={`Bỏ bài ${track.title}`} title="Bỏ khỏi lượt hát"><X size={17} /></button></div></div>
          {:else}<div class="book-empty"><ListMusic size={30} /><p>Chưa có bài nào trong hàng chờ.</p><button onclick={() => panel = 'search'}>Gọi thêm bài</button></div>{/each}
        </div>
      {/if}
      <button class="back-stage" onclick={() => deskOpen = false}><ChevronLeft size={16} />Về sân khấu</button>
    </aside>
    {/if}
  </div>
</svelte:element>

<style>
  .karaoke-room {
    position: fixed;
    inset: clamp(10px, 1.8vw, 24px);
    width: auto;
    height: auto;
    max-width: none;
    max-height: none;
    margin: 0;
    padding: 0;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: clamp(20px, 2.6vw, 34px);
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.75), inset 0 1px 1px rgba(255, 255, 255, 0.25);
    color: var(--text-primary);
    background: transparent;
    font: 14px var(--font-sans);
    overflow: hidden;
    transition: inset 0.3s cubic-bezier(0.16, 1, 0.3, 1), border-radius 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .karaoke-room.is-fullscreen {
    inset: 0;
    border-radius: 0;
    border: none;
  }
  .karaoke-room.embedded {
    position: relative;
    inset: auto;
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    border-radius: 0;
    border: none;
    box-shadow: none;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    isolation: isolate;
  }
  .karaoke-room::backdrop {
    background: transparent;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }
  .karaoke-room.focused { grid-template-rows: minmax(0, 1fr); }

  /* Background Modes - Khớp 100% với cài đặt nền đã điều chỉnh */
  .room-background { position: absolute; inset: 0; z-index: -1; overflow: hidden; pointer-events: none; }
  .stage-backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    max-width: 100%;
    max-height: 100%;
    display: block;
    object-fit: var(--bg-image-fit, cover);
    object-position: var(--bg-image-pos, center);
    filter: blur(var(--bg-image-blur, 0px))
      brightness(var(--bg-image-brightness, 1))
      saturate(var(--bg-image-saturation, 1.08));
    transition: filter 0.2s ease, transform 0.2s ease;
  }
  .stage-backdrop.is-blurred {
    transform: scale(1.06);
  }
  .stage-backdrop.album-backdrop {
    filter: blur(48px) brightness(0.55) saturate(1.4);
    transform: scale(1.15);
  }
  .backdrop-tint { position: absolute; inset: 0; background: var(--bg-overlay-tint, var(--bg-canvas)); opacity: 0.88; }
  .backdrop-tint.custom-tint {
    /* Khớp độ mờ lớp phủ theo cài đặt người dùng */
    opacity: var(--bg-image-dim, 0.48);
    transition: opacity 0.18s ease, background 0.2s ease;
  }
  .backdrop-tint.glass-tint {
    background: color-mix(in srgb, var(--bg-canvas) 78%, transparent);
    backdrop-filter: blur(36px) saturate(160%);
    -webkit-backdrop-filter: blur(36px) saturate(160%);
    opacity: 0.85;
  }
  .bg-oled .backdrop-tint { background: #000000; opacity: 0.98; }
  .glow-orb { position: absolute; width: 600px; height: 600px; border-radius: 50%; filter: blur(140px); opacity: 0.35; background-size: cover; }
  .glow-1 { top: -100px; left: -100px; animation: float-glow 14s ease-in-out infinite alternate; }
  .glow-2 { bottom: -100px; right: -100px; animation: float-glow 18s ease-in-out infinite alternate-reverse; }
  @keyframes float-glow { 0% { transform: scale(1) translate(0, 0); } 100% { transform: scale(1.3) translate(60px, 40px); } }

  .show-controls { position: absolute; right: 20px; bottom: 16px; z-index: 2; background: var(--bg-surface-1); border-radius: 9999px; }
  .show-controls:not(:hover):not(:focus-visible) { opacity: .65; }

  /* Header - Trong suốt, liền mạch không tạo thanh đen ở trên đầu */
  .room-header {
    display: flex; justify-content: space-between; align-items: center; gap: 12px;
    padding: 16px 28px;
    background: transparent;
    border-bottom: none;
    z-index: 5;
  }
  .room-name, .room-actions { display: flex; align-items: center; gap: 10px; }
  .room-name { color: var(--accent); }
  h1 { font-size: 16px; font-weight: 700; letter-spacing: -.02em; margin: 0; color: var(--text-primary); }

  .desk-toggle {
    display: inline-flex; align-items: center; gap: 7px; height: 34px; padding: 0 16px;
    border-radius: 9999px; border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.06); color: var(--text-primary); font-size: 12px; font-weight: 600;
    cursor: pointer; backdrop-filter: blur(16px); -webkit-backdrop-filter: blur(16px);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .desk-toggle:hover {
    background: rgba(255, 255, 255, 0.12); border-color: rgba(255, 255, 255, 0.24);
  }

  .icon-button {
    display: inline-flex; align-items: center; justify-content: center;
    width: 34px; height: 34px; padding: 0; border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.14); background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary); cursor: pointer; backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px); transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .icon-button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14); border-color: rgba(255, 255, 255, 0.28);
    transform: scale(1.05);
  }

  /* Body & Stages */
  .room-body { display: grid; grid-template-columns: minmax(0, 1fr); min-height: 0; }
  .room-body.show-desk { grid-template-columns: minmax(0, 1fr) 390px; }
  .singing-stage { position: relative; isolation: isolate; width: 100%; height: 100%; overflow: hidden; }

  /* ========================================================================= */
  /* VINYL TURNTABLE STUDIO VIEW (Chế độ Đĩa Than như video 616.mp4)           */
  /* ========================================================================= */
  .vinyl-studio-view {
    display: grid; grid-template-columns: minmax(320px, 46%) 1fr;
    width: 100%; height: 100%; overflow: hidden;
  }
  .turntable-dock {
    position: relative; display: flex; align-items: center; justify-content: center;
    overflow: hidden; padding: clamp(16px, 2.5vw, 36px); width: 100%; height: 100%;
    min-width: 0; min-height: 0; box-sizing: border-box;
  }
  .vinyl-assembly {
    position: relative; aspect-ratio: 1 / 1; width: min(90%, clamp(280px, 36vw, 540px));
    height: auto; max-height: 90%;
    border-radius: 50%;
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.22), 0 0 0 1px rgba(255, 255, 255, 0.12);
    transform: translateZ(0); flex-shrink: 0;
  }
  .vinyl-assembly.spinning .vinyl-record {
    animation: spin-disc var(--spin-duration, 1.8s) linear infinite;
  }
  @keyframes spin-disc { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }

  .vinyl-record {
    position: relative; width: 100%; height: 100%; aspect-ratio: 1 / 1; border-radius: 50%;
    background: var(--bg-canvas, #0d0d12); overflow: hidden; display: flex; align-items: center; justify-content: center;
    border: 2.5px solid color-mix(in srgb, var(--accent, #a855f7) 50%, rgba(255, 255, 255, 0.45));
    box-shadow: 0 0 24px color-mix(in srgb, var(--accent, #a855f7) 30%, transparent), 0 16px 48px rgba(0, 0, 0, 0.45);
    box-sizing: border-box;
  }
  /* Marble Skin: Các dải màu tím nhung, hồng phấn, xanh tím pastel mượt mà xoay theo đĩa như ảnh mẫu */
  .vinyl-record.skin-marble {
    background: conic-gradient(
      from 0deg,
      #7c3aed 0deg,
      #c084fc 60deg,
      #6366f1 120deg,
      #ec4899 180deg,
      #8b5cf6 240deg,
      #a855f7 300deg,
      #7c3aed 360deg
    );
    box-shadow: 0 0 28px rgba(168, 85, 247, 0.35);
  }
  .vinyl-record.skin-black {
    background: conic-gradient(
      from 0deg,
      #252733 0deg,
      #161720 60deg,
      #313444 120deg,
      #161720 180deg,
      #2b2e3d 240deg,
      #161720 300deg,
      #252733 360deg
    );
  }
  .vinyl-record.skin-cyberpunk {
    background: conic-gradient(
      from 0deg,
      #06b6d4 0deg,
      #3b82f6 90deg,
      #8b5cf6 180deg,
      #06b6d4 270deg,
      #06b6d4 360deg
    );
    box-shadow: 0 0 24px rgba(6, 182, 212, 0.3);
  }
  .vinyl-record.skin-gold {
    background: conic-gradient(
      from 0deg,
      #fbbf24 0deg,
      #f59e0b 60deg,
      #fef08a 120deg,
      #d97706 180deg,
      #fbbf24 240deg,
      #fef08a 300deg,
      #fbbf24 360deg
    );
    box-shadow: 0 0 24px rgba(245, 158, 11, 0.25);
  }
  /* Custom Skin: Màu người dùng chọn xoay tròn với các dải sáng lấp lánh không quầng đen */
  .vinyl-record.skin-custom {
    background: conic-gradient(
      from 0deg,
      var(--vinyl-custom-color, #6366f1) 0deg,
      color-mix(in srgb, var(--vinyl-custom-color, #6366f1) 75%, #ffffff) 60deg,
      var(--vinyl-custom-color, #6366f1) 120deg,
      color-mix(in srgb, var(--vinyl-custom-color, #6366f1) 60%, var(--accent, #8b5cf6)) 180deg,
      color-mix(in srgb, var(--vinyl-custom-color, #6366f1) 85%, #ffffff) 240deg,
      color-mix(in srgb, var(--vinyl-custom-color, #6366f1) 70%, var(--accent, #8b5cf6)) 300deg,
      var(--vinyl-custom-color, #6366f1) 360deg
    );
    box-shadow: 0 0 24px color-mix(in srgb, var(--vinyl-custom-color, #6366f1) 30%, transparent);
  }
  .vinyl-album-art { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; opacity: 0.88; }
  .vinyl-rim-art {
    position: absolute; inset: 0; border-radius: 50%; background-size: cover; background-position: center;
    -webkit-mask-image: radial-gradient(circle, transparent 58%, black 60%, black 98%, transparent 100%);
    mask-image: radial-gradient(circle, transparent 58%, black 60%, black 98%, transparent 100%);
    opacity: 0.9;
  }

  /* Grooves specular reflection - Chùm sáng phản chiếu đĩa than quay cực kỳ rõ nét */
  .groove-reflection {
    position: absolute; inset: 0; border-radius: 50%; pointer-events: none;
    background: conic-gradient(
      from 45deg,
      transparent 0deg,
      rgba(255, 255, 255, 0.28) 25deg,
      rgba(255, 255, 255, 0.45) 35deg,
      rgba(255, 255, 255, 0.28) 45deg,
      transparent 70deg,
      transparent 180deg,
      rgba(255, 255, 255, 0.28) 205deg,
      rgba(255, 255, 255, 0.45) 215deg,
      rgba(255, 255, 255, 0.28) 225deg,
      transparent 250deg
    );
    mix-blend-mode: screen;
    opacity: 0.85;
  }

  /* Center Label — Thiết kế chuẩn nét theo ảnh mẫu với vòng tròn nét đứt và chốt sáng */
  .center-label {
    position: relative; aspect-ratio: 1 / 1; width: 34%; height: 34%; border-radius: 50%;
    background: radial-gradient(circle, color-mix(in srgb, var(--accent, #8b5cf6) 28%, #1f1b2e) 0%, color-mix(in srgb, var(--accent, #8b5cf6) 38%, #161324) 60%, #0d0c18 100%);
    border: 2px solid rgba(255, 255, 255, 0.35);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4), inset 0 0 12px rgba(0, 0, 0, 0.6);
    display: flex; align-items: center; justify-content: center;
    z-index: 2; overflow: hidden;
  }
  .center-label-fallback {
    position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
    background: radial-gradient(circle, color-mix(in srgb, var(--accent) 35%, #1e1b4b) 0%, color-mix(in srgb, var(--bg-canvas) 85%, #0f172a) 100%);
    border-radius: 50%;
  }
  .center-vinyl-icon {
    color: var(--accent);
    display: flex; align-items: center; justify-content: center;
    filter: drop-shadow(0 0 10px var(--accent));
    transform: rotate(45deg);
  }
  .center-label-art {
    position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; border-radius: 50%;
    filter: brightness(0.92) contrast(1.05);
  }
  .center-label-tint {
    position: absolute; inset: 0;
    background: radial-gradient(circle, transparent 60%, rgba(0, 0, 0, 0.25) 100%);
    border-radius: 50%; z-index: 1;
  }
  /* Vòng tròn nét đứt màu trắng mờ chuẩn theo ảnh mẫu */
  .center-dashed-ring {
    position: absolute;
    inset: 12%;
    border-radius: 50%;
    border: 1.5px dashed rgba(255, 255, 255, 0.55);
    pointer-events: none;
    z-index: 2;
  }
  /* Cụm chốt tâm spindle 2 tầng phát sáng sang trọng */
  .spindle-hub {
    position: relative;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3;
  }
  .spindle-accent-ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: radial-gradient(circle, #f3e8ff 0%, #d8b4fe 40%, color-mix(in srgb, var(--accent, #a855f7) 80%, #ffffff) 75%, #6b21a8 100%);
    box-shadow: 0 0 8px rgba(168, 85, 247, 0.65), inset 0 1px 2px rgba(255, 255, 255, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.5);
  }
  .spindle-hole {
    position: relative;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #090a0f;
    border: 1.2px solid rgba(255, 255, 255, 0.35);
    box-shadow: inset 0 1.5px 3px rgba(0, 0, 0, 0.95);
    z-index: 4;
  }

  /* Tonearm Assembly — Thiết kế chuẩn theo ảnh mẫu với ống cần chữ J và đầu kim bo góc */
  .tonearm-assembly {
    position: absolute; top: 4%; right: 3%; width: 130px; height: 440px;
    transform-origin: 70px 55px;
    cursor: grab; user-select: none; z-index: 10; touch-action: none;
    outline: none; pointer-events: auto;
  }
  .tonearm-assembly:hover {
    filter: drop-shadow(0 0 12px rgba(255, 255, 255, 0.45));
  }
  .tonearm-assembly.is-dragging, .tonearm-assembly:active {
    cursor: grabbing;
    transition: none !important;
  }
  .tonearm-assembly:not(.is-dragging) {
    transition: transform 0.65s cubic-bezier(0.18, 0.89, 0.32, 1.15);
  }
  .tonearm-svg {
    display: block; width: 100%; height: 100%; overflow: visible;
  }

  /* Right Pane: Lyrics & Meta */
  .vinyl-lyrics-pane {
    position: relative; display: flex; flex-direction: column; justify-content: space-between;
    height: 100%; padding: clamp(24px, 3.5vw, 44px) clamp(24px, 4vw, 56px); overflow: hidden;
  }
  .song-header-right {
    display: flex; justify-content: space-between; align-items: flex-start; gap: 14px;
  }
  .meta-texts { display: flex; flex-direction: column; gap: 4px; }
  .vh-title { font-size: clamp(20px, 2.4vw, 34px); font-weight: 750; margin: 0; letter-spacing: -0.025em; color: var(--text-primary, #ffffff); }
  .vh-artist { font-size: clamp(12px, 1.2vw, 15px); color: var(--text-secondary); margin: 0; font-weight: 500; }
  .neon-status-dot {
    width: 11px; height: 11px; border-radius: 50%; background: color-mix(in srgb, var(--accent) 50%, #ef4444); flex-shrink: 0; margin-top: 6px;
    transition: all 0.3s ease;
  }
  .neon-status-dot.playing {
    background: var(--accent); box-shadow: 0 0 12px var(--accent);
    animation: pulse-dot 1.4s ease-in-out infinite alternate;
  }
  @keyframes pulse-dot { 0% { opacity: 0.6; transform: scale(0.9); } 100% { opacity: 1; transform: scale(1.15); } }

  /* Curved Lyric Flow */
  .stream-content {
    flex: 1; display: flex; flex-direction: column; justify-content: center; min-height: 0;
  }
  .curved-lyric-flow {
    display: flex; flex-direction: column; gap: clamp(14px, 2.5vh, 28px);
  }
  .flow-line {
    display: flex; align-items: baseline; gap: 14px; margin: 0; transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .flow-text { font-weight: 600; line-height: 1.5; color: var(--text-secondary); }

  .line-prev {
    font-size: clamp(16px, 1.8vw, 24px); opacity: 0.35; transform: translateX(18px);
  }
  .dim-soft .line-prev, .dim-soft .line-next { opacity: 0.55; }
  .dim-hidden .line-prev, .dim-hidden .line-next { opacity: 0; }

  .line-current {
    font-size: clamp(24px, 3.2vw, 44px); font-weight: 750; color: var(--text-primary, #ffffff);
    transform: translateX(0);
  }
  .font-compact .line-current { font-size: clamp(20px, 2.6vw, 34px); }
  .font-large .line-current { font-size: clamp(30px, 4.2vw, 56px); }

  .line-next {
    font-size: clamp(18px, 2.1vw, 28px); opacity: 0.35; transform: translateX(24px);
  }

  /* Karaoke Word-by-Word Highlight chạy chuẩn từng chữ theo nhip */
  .words-container { position: relative; display: inline-flex; flex-wrap: wrap; gap: 8px; align-items: baseline; }
  .sung-word {
    position: relative;
    display: inline-block;
    transition: transform 0.12s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .sung-word.active-word {
    transform: scale(1.06);
  }
  .sung-word .word-chars {
    display: inline-block;
    background: linear-gradient(
      90deg,
      #ffffff 0%,
      #ffffff var(--fill),
      rgba(255, 255, 255, 0.38) var(--fill),
      rgba(255, 255, 255, 0.38) 100%
    );
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
    transition: filter 0.15s ease;
  }
  .sung-word.active-word .word-chars {
    filter: drop-shadow(0 0 14px color-mix(in srgb, var(--accent, #a855f7) 75%, #ffffff));
  }
  .sung-word.done .word-chars {
    background: none;
    color: #ffffff;
    filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.35));
  }

  .instrumental-pill { font-style: italic; color: var(--text-secondary, #94a3b8); font-size: 0.7em; }

  /* Vinyl Mini Controls under lyrics */
  .vinyl-mini-controls {
    display: flex; flex-direction: column; gap: 10px; padding-top: 18px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }
  .mini-timeline { display: flex; align-items: center; gap: 10px; }
  .time-num { font: 11px var(--font-mono, monospace); color: var(--text-secondary); font-variant-numeric: tabular-nums; }
  .seek-track { position: relative; flex: 1; height: 18px; }
  .seek-track::before {
    content: ''; position: absolute; inset: 7px 0; border-radius: 9999px;
    background: linear-gradient(to right, var(--accent) var(--progress), rgba(255, 255, 255, 0.12) var(--progress));
    pointer-events: none;
  }
  .seek-track input { position: absolute; inset: 0; width: 100%; margin: 0; opacity: 0; cursor: pointer; }

  .mini-transport-bar { display: flex; justify-content: space-between; align-items: center; }
  .mini-actions-left { display: flex; gap: 6px; }
  .text-pill-btn {
    display: inline-flex; align-items: center; gap: 5px; height: 30px; padding: 0 14px;
    border-radius: 9999px; border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.06); color: var(--text-secondary); font-size: 11px; cursor: pointer;
    backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .text-pill-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12); color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.24);
  }
  .mini-transport-buttons { display: flex; align-items: center; gap: 8px; }
  .play-btn-circle {
    width: 40px; height: 40px; border-radius: 50%; background: var(--accent);
    color: var(--accent-fg, #ffffff); border: none; display: flex; align-items: center; justify-content: center; cursor: pointer;
    box-shadow: 0 4px 14px var(--accent-glow, rgba(0, 0, 0, 0.35)); transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .play-btn-circle:hover:not(:disabled) { filter: brightness(1.15); transform: scale(1.06); }
  .next-btn-circle {
    width: 32px; height: 32px; border-radius: 50%; background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.14); color: var(--text-primary, #ffffff); display: flex; align-items: center; justify-content: center; cursor: pointer;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .next-btn-circle:hover:not(:disabled) { background: rgba(255, 255, 255, 0.14); transform: scale(1.05); }

  /* Center Stage Style */
  .song-heading { display: flex; align-items: center; gap: 14px; padding: 20px 32px 0; }
  .cover { width: 52px; height: 52px; border-radius: 12px; object-fit: cover; transition: border-radius 0.3s ease; }
  .cover.spinning { border-radius: 50%; animation: spin-disc 3s linear infinite; }
  .song-meta h2 { font-size: 16px; margin: 0; }
  .song-meta p { font-size: 12px; color: var(--text-secondary); margin: 3px 0 0; }

  .lyric-stage {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 22px; height: 100%; text-align: center; padding: 24px; overflow-y: auto;
  }
  .lyric-stage .previous-line { font-size: clamp(15px, 1.6vw, 22px); color: var(--text-secondary); opacity: 0.45; }
  .lyric-stage .current-line {
    font-size: clamp(28px, 4.5vw, 68px); font-weight: 750; line-height: 1.4; max-width: 26ch;
  }
  .lyric-stage .next-line { font-size: clamp(18px, 2.2vw, 30px); color: var(--text-primary); opacity: 0.65; }

  .stage-controls { padding: 16px 32px; border-top: 1px solid rgba(255, 255, 255, 0.08); }
  .timeline { display: flex; align-items: center; gap: 12px; }
  .timeline > span { font: 11px var(--font-mono); color: var(--text-secondary); }
  .transport-row { display: flex; justify-content: space-between; align-items: center; margin-top: 10px; }
  .play-button { width: 48px; height: 48px; border-radius: 50%; background: var(--accent); color: var(--accent-fg, #ffffff); border: none; cursor: pointer; }

  /* Full lyrics layout */
  .full-lyrics-container { height: 100%; overflow-y: auto; padding: 10vh 32px; }
  .full-lines-scroll { display: flex; flex-direction: column; align-items: center; gap: 20px; }
  .full-lyric-line {
    background: transparent; border: none; font-size: clamp(18px, 2.5vw, 32px);
    font-weight: 600; color: var(--text-secondary); cursor: pointer; display: flex; align-items: baseline; gap: 10px;
    border-radius: 12px; padding: 6px 14px; transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .full-lyric-line:hover { background: rgba(255, 255, 255, 0.05); }
  .full-lyric-line.active { color: var(--accent); font-weight: 750; transform: scale(1.06); }

  /* Side Drawer for songbook & lyrics editor (Bo cong kính nổi theo phong cách Cài đặt) */
  .songbook {
    margin: 14px 16px 16px 0;
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: color-mix(in srgb, var(--bg-canvas) 75%, transparent);
    backdrop-filter: blur(30px) saturate(190%);
    -webkit-backdrop-filter: blur(30px) saturate(190%);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .book-tabs {
    display: flex;
    padding: 5px;
    gap: 6px;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 9999px;
    margin: 14px 14px 8px;
    border: none;
  }
  .book-tabs button {
    flex: 1;
    border: none;
    border-radius: 9999px;
    background: transparent;
    padding: 6px 10px;
    cursor: pointer;
    color: var(--text-secondary);
    font-size: 11.5px;
    font-weight: 550;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }
  .book-tabs button.chosen {
    background: var(--accent);
    color: #ffffff;
    font-weight: 650;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  }
  .room-lyrics { flex: 1; display: flex; min-height: 0; overflow: hidden; }
  .book-list { flex: 1; overflow-y: auto; padding: 0 14px; }
  .song-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 12px;
    border-bottom: none;
    margin-bottom: 4px;
    transition: background 0.15s ease;
  }
  .song-row:hover { background: rgba(255, 255, 255, 0.06); }
  .song-search { padding: 14px; }
  .search-field { display: flex; gap: 6px; }
  .search-field input {
    flex: 1;
    padding: 9px 12px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-primary);
    font-size: 12px;
  }
  .source-choice { display: flex; gap: 5px; margin-top: 10px; }
  .source-choice button {
    flex: 1;
    padding: 6px 8px;
    font-size: 11px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }
  .source-choice button.chosen {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent);
    font-weight: 600;
  }
  .back-stage {
    margin: 10px 14px 14px;
    padding: 8px 14px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.05);
    cursor: pointer;
    font-size: 12px;
    color: var(--text-secondary);
    transition: all 0.2s ease;
  }
  .back-stage:hover { background: rgba(255, 255, 255, 0.1); color: var(--text-primary); }

  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-secondary); text-align: center; }
  .empty-state h3 { font-size: 20px; color: var(--text-primary); margin: 0; }
  .open-lyrics-btn, .retry-btn { padding: 8px 18px; border-radius: 9999px; background: var(--accent); color: var(--accent-fg, #ffffff); border: none; cursor: pointer; }

  /* Apple Liquid Glass button overrides for transparency & seamlessness */
  :global(html[data-apple-glass="on"]) .karaoke-room button:not(.play-button):not(.play-btn-circle) {
    background: rgba(255, 255, 255, 0.08) !important;
    backdrop-filter: blur(16px) saturate(180%) !important;
    -webkit-backdrop-filter: blur(16px) saturate(180%) !important;
    border-color: rgba(255, 255, 255, 0.18) !important;
  }
  :global(html[data-apple-glass="on"]) .room-header {
    background: transparent !important;
    border-bottom: none !important;
  }

  @media (max-width: 900px) {
    .karaoke-room { inset: 8px; border-radius: 20px; }
    .vinyl-studio-view { grid-template-columns: 1fr; grid-template-rows: auto 1fr; }
    .turntable-dock { padding: 12px; }
    .vinyl-assembly { width: 240px; height: 240px; }
    .tonearm-assembly { display: none; }
  }
</style>
