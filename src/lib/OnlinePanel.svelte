<script lang="ts">
  import { Play, Bookmark, Plus, Download, ChevronDown } from "@lucide/svelte";
  import { api, fmtTime, getTrackCover, handleImgError, type OnlineTrack, type Track } from "$lib/api";
  import { player } from "$lib/player.svelte";
  import Recommendations from "./Recommendations.svelte";
  import WaveformBar from "./WaveformBar.svelte";

  let {
    libraryTracks = [],
    results = [],
    busy = false,
    err = "",
    downloads = {},
    onrefresh,
    ondownload,
    onopen,
    onclear,
    onloadmore,
    loadingMore = false,
  }: {
    libraryTracks: Track[];
    results?: OnlineTrack[];
    busy?: boolean;
    err?: string;
    downloads: Record<string, number>;
    onrefresh: () => Promise<void>;
    ondownload: (t: OnlineTrack) => Promise<void>;
    onopen?: (t: Track) => void;
    onclear?: () => void;
    onloadmore?: () => Promise<void>;
    loadingMore?: boolean;
  } = $props();

  const savedUrls = $derived(
    new Set(libraryTracks.map((t) => t.url || t.path).filter(Boolean)),
  );
  const downloadedUrls = $derived(
    new Set(
      libraryTracks
        .filter((t) => t.url && !t.path.startsWith("http"))
        .map((t) => t.url as string),
    ),
  );

  function toTrack(r: OnlineTrack, idx: number): Track {
    const existing = libraryTracks.find((t) => t.url === r.url || t.path === r.url);
    if (existing) return existing;
    return {
      id: -1 - idx,
      path: r.url,
      title: r.title,
      artist: r.artist,
      album: r.album,
      duration: r.duration,
      cover_path: r.thumbnail,
      source: r.source,
      url: r.url,
    };
  }

  function playAndInspectResult(r: OnlineTrack, i: number) {
    player.playOnlineList(results, i);
    onopen?.(toTrack(r, i));
  }

  async function saveOne(t: OnlineTrack) {
    try {
      await api.addOnlineToLibrary(t);
      await onrefresh();
    } catch {
      // handled upstream
    }
  }

  async function saveAll() {
    for (const r of results) {
      await api.addOnlineToLibrary(r);
    }
    await onrefresh();
  }
</script>

<div class="online-wrap">
  {#if busy}
    <div class="searching-banner">
      <span class="mini-spin"></span>
      <span>Đang tìm kiếm trực tuyến từ YouTube & SoundCloud…</span>
    </div>
  {/if}

  {#if err}
    <div class="err-banner">{err}</div>
  {/if}

  {#if results.length > 0}
    <section class="results-panel">
      <header class="results-head">
        <div class="res-meta">
          <span class="kicker">Kết quả tìm kiếm</span>
          <strong>{results.length} bản nhạc tìm thấy</strong>
        </div>
        <div class="res-actions">
          <button class="act-pill primary" onclick={() => playAndInspectResult(results[0], 0)}>
            <Play size={13} />
            <span>Phát tất cả</span>
          </button>
          <button class="act-pill" onclick={saveAll}>
            <Bookmark size={13} />
            <span>Lưu tất cả</span>
          </button>
          {#if onclear}
            <button class="act-pill ghost" onclick={onclear}>Đóng kết quả</button>
          {/if}
        </div>
      </header>

      <div class="results-list">
        {#each results as r, i (r.url + ":" + i)}
          {@const pct = downloads[r.url]}
          {@const isDownloaded = downloadedUrls.has(r.url)}
          {@const isSaved = savedUrls.has(r.url) || isDownloaded}
          {@const isCurrent = player.current?.url === r.url}

          <div
            class="res-row"
            class:cur={isCurrent}
            ondblclick={() => playAndInspectResult(r, i)}
            role="row"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && playAndInspectResult(r, i)}
          >
            <button
              class="idx-trigger"
              onclick={() => playAndInspectResult(r, i)}
              title="Phát & Mở sóng nhạc {r.title}"
            >
              <span class="idx-no">{String(i + 1).padStart(2, "0")}</span>
              <span class="idx-icon">
                <Play size={14} />
              </span>
            </button>

            <button class="info-btn" onclick={() => playAndInspectResult(r, i)} title="Mở khung chi tiết bài hát">
              <div class="art-cell">
                <img
                  src={getTrackCover(r.thumbnail, r.url, r.title, r.artist)}
                  alt=""
                  loading="lazy"
                  onerror={(e) => handleImgError(e, r.title, r.artist, r.url)}
                />
              </div>

              <div class="info-cell">
                <div class="title-line">
                  <span class="r-title">{r.title}</span>
                  <span
                    class="src-badge"
                    class:sc={r.source === "soundcloud"}
                    class:sp={r.source === "spotify"}
                  >
                    {r.source === "soundcloud" ? "SC" : r.source === "spotify" ? "SP" : "YT"}
                  </span>
                </div>
                <div class="r-artist">{r.artist}</div>
              </div>
            </button>

            <div class="wave-cell">
              <WaveformBar
                seed="{r.url}:{r.title}:{r.duration}"
                position={isCurrent ? player.position : 0}
                duration={isCurrent && player.duration > 0 ? player.duration : r.duration}
                playing={isCurrent && player.playing}
                barsCount={40}
                height={26}
                onseek={(sec) => {
                  if (isCurrent) {
                    player.seek(sec);
                  } else {
                    playAndInspectResult(r, i);
                    setTimeout(() => player.seek(sec), 250);
                  }
                }}
              />
            </div>

            <div class="dur-cell">{fmtTime(r.duration)}</div>

            <div class="row-actions">
              <button
                class="row-chip"
                onclick={() => player.enqueueOnline(r)}
                title="Thêm vào hàng chờ"
              >
                <Plus size={13} />
                <span>Hàng chờ</span>
              </button>

              <button
                class="row-chip"
                class:saved={isSaved}
                onclick={() => saveOne(r)}
                disabled={isSaved}
                title="Lưu vào thư viện"
              >
                {isSaved ? "Đã lưu" : "Lưu"}
              </button>

              {#if isDownloaded}
                <span class="dl-tag offline">OFFLINE</span>
              {:else if pct !== undefined}
                <span class="dl-tag active">{Math.round(pct)}%</span>
              {:else}
                <button
                  class="row-chip"
                  onclick={() => ondownload(r)}
                  title="Tải về máy nghe Offline"
                >
                  <Download size={13} />
                  <span>Tải</span>
                </button>
              {/if}
            </div>
          </div>
        {/each}
        {#if onloadmore}
          <div class="load-more-results-row">
            {#if loadingMore}
              <div class="searching-more-pill">
                <span class="mini-spin"></span>
                <span>Đang tiếp tục tìm thêm kết quả cho bạn…</span>
              </div>
            {:else}
              <button type="button" class="load-more-search-btn" onclick={onloadmore}>
                <ChevronDown size={14} />
                <span>Tải thêm kết quả tìm kiếm ({results.length} bài đã tìm thấy)</span>
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <Recommendations {downloads} {savedUrls} {downloadedUrls} onsave={saveOne} {ondownload} {onopen} {onclear} />
</div>

<style>
  .online-wrap {
    padding: 4px 6px 28px;
  }

  .searching-banner {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    border-radius: 10px;
    background: var(--accent-soft);
    border: 1px solid oklch(0.7 0.17 238 / 0.3);
    color: var(--accent);
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 16px;
  }

  .mini-spin {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid oklch(0.7 0.17 238 / 0.25);
    border-top-color: var(--accent);
    animation: spin 0.75s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .err-banner {
    padding: 12px 16px;
    border-radius: 10px;
    background: oklch(0.67 0.19 25 / 0.12);
    border: 1px solid oklch(0.67 0.19 25 / 0.35);
    color: var(--danger);
    font-size: 13px;
    margin-bottom: 16px;
  }

  /* Results Section */
  
  .load-more-results-row {
    display: flex;
    justify-content: center;
    padding: 18px 0 8px;
  }
  .searching-more-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 18px;
    border-radius: 999px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    color: var(--accent);
    font-size: 12.5px;
    font-weight: 600;
  }
  .load-more-search-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 20px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .load-more-search-btn:hover {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    border-color: var(--accent);
    transform: translateY(-1px);
    box-shadow: 0 4px 14px var(--accent-glow);
  }

  .results-panel {
    margin-bottom: 24px;
    border-radius: var(--ui-radius, 16px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    padding: 16px;
  }
  .results-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 12px;
    margin-bottom: 10px;
    border-bottom: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }
  .res-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .kicker {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--accent);
  }
  .res-meta strong {
    font-size: 15px;
    letter-spacing: -0.015em;
  }
  .res-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .act-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .act-pill.primary {
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
  }
  .act-pill.ghost {
    color: var(--text-secondary);
  }
  .act-pill:hover {
    filter: brightness(1.08);
  }

  .results-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .res-row {
    display: grid;
    grid-template-columns: 30px minmax(0, 1.8fr) minmax(100px, 1.2fr) 48px auto;
    gap: 10px;
    align-items: center;
    padding: 7px 10px;
    border-radius: var(--ui-radius-sm, 10px);
    transition: background 0.15s ease;
    outline: none;
  }
  .res-row:hover,
  .res-row:focus-visible {
    background: var(--bg-surface-2);
  }
  .res-row.cur {
    background: var(--accent-soft);
  }

  @container stage (max-width: 760px) {
    .res-row {
      grid-template-columns: 28px minmax(0, 1fr) 48px auto;
      gap: 8px;
      padding-inline: 6px;
    }
    .wave-cell {
      display: none;
    }
  }
  @container stage (max-width: 560px) {
    .results-panel {
      padding: 12px;
    }
    .res-row {
      grid-template-columns: 28px minmax(0, 1fr) 48px;
    }
    .row-actions {
      grid-column: 2 / -1;
      justify-content: flex-start;
      flex-wrap: wrap;
    }
  }

  .idx-trigger {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }
  .idx-no {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }
  .idx-icon {
    display: none;
    color: var(--accent);
  }
  .res-row:hover .idx-no {
    display: none;
  }
  .res-row:hover .idx-icon {
    display: inline-grid;
    place-items: center;
  }

  .info-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 0;
    color: inherit;
    font-family: var(--font-sans);
    text-align: left;
    cursor: pointer;
  }
  .art-cell {
    width: 42px;
    height: 42px;
    border-radius: 8px;
    overflow: hidden;
    flex: none;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
  }
  .art-cell img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .info-cell {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .title-line {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .r-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .info-btn:hover .r-title,
  .res-row.cur .r-title {
    color: var(--accent);
  }
  .src-badge {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.05em;
    padding: 1.5px 5px;
    border-radius: 4px;
    background: var(--yt-soft);
    color: var(--yt-tone);
    flex: none;
  }
  .src-badge.sc {
    background: var(--sc-soft);
    color: var(--sc-tone);
  }
  .src-badge.sp {
    background: rgba(30, 215, 96, 0.14);
    color: #1ed760;
  }
  .r-artist {
    font-size: 11.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .wave-cell {
    min-width: 0;
  }

  .dur-cell {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11.5px;
    color: var(--text-muted);
    text-align: right;
  }

  .row-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
  }
  .row-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .row-chip:hover:not(:disabled) {
    background: var(--bg-surface-3);
    color: var(--text-primary);
  }
  .row-chip.saved {
    background: var(--signal-online-soft);
    color: var(--signal-online);
    border-color: oklch(0.75 0.15 175 / 0.35);
  }
  .row-chip:disabled {
    cursor: default;
  }

  .dl-tag {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 6px;
  }
  .dl-tag.offline {
    background: var(--signal-online-soft);
    color: var(--signal-online);
  }
  .dl-tag.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
