<script lang="ts">
  import { AudioLines, Play, Heart, Info, Download, ListPlus, X, Bookmark, Trash2 } from "@lucide/svelte";
  import { coverSrc, fmtTime, getTrackCover, handleImgError, type Track, type Playlist } from "$lib/api";
  import { player } from "$lib/player.svelte";
  import WaveformBar from "./WaveformBar.svelte";
  import LyricsBatch from "./LyricsBatch.svelte";

  let {
    tracks,
    playlists,
    activePlaylist = null,
    downloads = {},
    onadd,
    onremove,
    ondelete,
    ondownload,
    onopen,
    ontogglelike,
  }: {
    tracks: Track[];
    playlists: Playlist[];
    activePlaylist?: number | null;
    downloads?: Record<string, number>;
    ondelete: (t: Track) => void;
    onadd: (playlistId: number, trackId: number) => void;
    onremove: (trackId: number) => void;
    ondownload?: (t: Track) => void;
    onopen?: (t: Track, index: number) => void;
    ontogglelike?: (t: Track) => void;
  } = $props();

  let menuFor = $state<number | null>(null);

  function activateTrack(t: Track, i: number) {
    player.playList(tracks, i);
    onopen?.(t, i);
  }
</script>

{#if tracks.length === 0}
  <div class="empty-state">
    <div class="empty-icon" aria-hidden="true">
      <AudioLines size={32} />
    </div>
    <h3>Thư viện bài hát đang trống</h3>
    <p>Bấm <strong>Thêm thư mục nhạc</strong> ở thanh bên hoặc vào mục <strong>Khám phá</strong> để lưu bài hát yêu thích.</p>
  </div>
{:else}
  <LyricsBatch />
  <div class="table-wrap">
    <div class="thead" aria-hidden="true">
      <span class="col-idx">#</span>
      <span class="col-title">Bài hát</span>
      <span class="col-wave">Sóng nhạc</span>
      <span class="col-dur">Thời lượng</span>
      <span class="col-act"></span>
    </div>

    <div class="tbody" role="table" aria-label="Danh sách bài hát">
      {#each tracks as t, i (t.id)}
        {@const isRemote = t.path.startsWith("http://") || t.path.startsWith("https://")}
        {@const dlKey = t.url ?? t.path}
        {@const dlPct = downloads[dlKey]}
        {@const isCurrent =
          player.current?.id === t.id ||
          Boolean(player.current?.url && player.current.url === t.url)}

        <div
          class="trow"
          class:cur={isCurrent}
          ondblclick={() => activateTrack(t, i)}
          role="row"
          tabindex="0"
          onkeydown={(e) => e.key === "Enter" && activateTrack(t, i)}
        >
          <!-- Index / Equalizer / Play Trigger -->
          <button
            class="idx-cell"
            onclick={() => (isCurrent ? player.toggle() : activateTrack(t, i))}
            title={isCurrent && player.playing ? "Tạm dừng" : "Phát bài hát"}
            aria-label="Phát {t.title}"
          >
            {#if isCurrent && player.playing}
              <span class="eq" aria-hidden="true">
                <i class="b1"></i>
                <i class="b2"></i>
                <i class="b3"></i>
              </span>
            {:else}
              <span class="idx-num">{String(i + 1).padStart(2, "0")}</span>
              <span class="idx-play" aria-hidden="true">
                <Play size={14} />
              </span>
            {/if}
          </button>

          <!-- Cover + Title + Artist (Click opens full Track Detail Frame) -->
          <button class="title-cell" onclick={() => activateTrack(t, i)} title="Xem chi tiết bài hát">
            <div class="cover-box">
              <img
                src={getTrackCover(t.cover_path, t.url || t.path, t.title, t.artist)}
                alt=""
                loading="lazy"
                onerror={(e) => handleImgError(e, t.title, t.artist, t.url || t.path)}
              />
            </div>

            <div class="text-meta">
              <div class="song-title">
                <span class="t-text">{t.title}</span>
                {#if t.source !== "local"}
                  <span
                    class="source-tag"
                    class:yt={t.source === "youtube"}
                    class:sc={t.source === "soundcloud"}
                    class:sp={t.source === "spotify"}
                    class:offline={!isRemote}
                    title={isRemote ? "Phát trực tuyến (Cloud)" : "Đã tải về máy (Offline)"}
                  >
                    {t.source === "soundcloud" ? "SC" : t.source === "spotify" ? "SP" : "YT"} · {isRemote ? "CLOUD" : "OFFLINE"}
                  </span>
                {/if}
              </div>
              <div class="song-artist">
                <span>{t.artist || "Nghệ sĩ chưa rõ"}</span>
                {#if t.album}
                  <span class="dot-sep">·</span>
                  <span class="album-sub">{t.album}</span>
                {/if}
              </div>
            </div>
          </button>

          <!-- Inline SoundCloud Waveform Column -->
          <div class="wave-cell">
            <WaveformBar
              seed="{t.id}:{t.title}:{t.duration}"
              position={isCurrent ? player.position : 0}
              duration={isCurrent && player.duration > 0 ? player.duration : t.duration}
              playing={isCurrent && player.playing}
              barsCount={44}
              height={28}
              onseek={(sec) => {
                if (isCurrent) {
                  player.seek(sec);
                } else {
                  player.playList(tracks, i);
                  onopen?.(t, i);
                  setTimeout(() => player.seek(sec), 250);
                }
              }}
            />
          </div>

          <!-- Duration -->
          <div class="dur-cell">{fmtTime(t.duration)}</div>

          <!-- Row Actions -->
          <div class="act-cell">
            <button
              class="row-btn like-btn"
              class:liked={t.liked}
              onclick={(e) => {
                e.stopPropagation();
                ontogglelike?.(t);
              }}
              title={t.liked ? "Bỏ thích" : "Yêu thích (Lưu vào Trang chủ)"}
              aria-label="Thích"
            >
              <Heart size={15} fill={t.liked ? "currentColor" : "none"} />
            </button>

            <button
              class="row-btn"
              onclick={() => onopen?.(t, i)}
              title="Mở khung thông tin chi tiết bài hát"
              aria-label="Chi tiết bài hát"
            >
              <Info size={15} />
            </button>

            {#if isRemote && ondownload}
              {#if dlPct !== undefined}
                <span class="dl-pill">{Math.round(dlPct)}%</span>
              {:else}
                <button
                  class="row-btn"
                  onclick={() => ondownload(t)}
                  title="Tải về nghe Offline"
                  aria-label="Tải về nghe Offline"
                >
                  <Download size={15} />
                </button>
              {/if}
            {/if}

            <button
              class="row-btn"
              onclick={() => player.enqueue(t)}
              title="Thêm vào hàng chờ"
              aria-label="Thêm vào hàng chờ"
            >
              <ListPlus size={15} />
            </button>

            {#if activePlaylist}
              <button
                class="row-btn danger"
                onclick={() => onremove(t.id)}
                title="Gỡ khỏi Playlist này"
                aria-label="Gỡ khỏi Playlist"
              >
                <X size={15} />
              </button>
            {:else}
              <div class="menu-anchor">
                <button
                  class="row-btn"
                  class:active={menuFor === t.id}
                  onclick={() => (menuFor = menuFor === t.id ? null : t.id)}
                  title="Thêm vào Playlist"
                  aria-label="Thêm vào Playlist"
                >
                  <Bookmark size={15} />
                </button>

                {#if menuFor === t.id}
                  <div class="popover">
                    <div class="popover-head">Thêm vào Playlist</div>
                    {#each playlists as p}
                      <button
                        class="popover-item"
                        onclick={() => {
                          onadd(p.id, t.id);
                          menuFor = null;
                        }}
                      >
                        <span>{p.name}</span>
                        <small>{p.count}</small>
                      </button>
                    {:else}
                      <span class="popover-empty">Chưa tạo playlist nào</span>
                    {/each}
                  </div>
                {/if}
              </div>

              <button
                class="row-btn danger"
                onclick={() => ondelete(t)}
                title="Xóa khỏi thư viện"
                aria-label="Xóa khỏi thư viện"
              >
                <Trash2 size={15} />
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .empty-state {
    margin: 36px auto;
    max-width: 440px;
    padding: 36px 28px;
    border-radius: var(--ui-radius, 16px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .empty-icon {
    width: 56px;
    height: 56px;
    border-radius: var(--ui-radius-md, 14px);
    background: var(--accent-soft);
    color: var(--accent);
    display: grid;
    place-items: center;
    margin-bottom: 4px;
  }
  .empty-state h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.015em;
  }
  .empty-state p {
    margin: 0;
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-secondary);
  }

  .table-wrap {
    display: flex;
    flex-direction: column;
    padding-bottom: 24px;
  }
  .thead {
    display: grid;
    grid-template-columns: 32px minmax(0, 1.9fr) minmax(100px, 1fr) 64px 152px;
    gap: 12px;
    align-items: center;
    padding: 0 12px 10px;
    border-inline: 1px solid transparent;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-bottom: 6px;
  }
  .col-idx {
    text-align: center;
  }
  .col-dur {
    text-align: right;
    white-space: nowrap;
    font-size: 10px;
    letter-spacing: 0;
  }

  .tbody {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .trow {
    display: grid;
    grid-template-columns: 32px minmax(0, 1.9fr) minmax(100px, 1fr) 64px 152px;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    border-radius: var(--ui-radius-sm, 10px);
    border: 1px solid transparent;
    transition: background 0.16s ease, border-color 0.16s ease;
    outline: none;
  }
  .trow:hover,
  .trow:focus-visible {
    background: var(--bg-surface-1);
    border-color: var(--border-subtle);
  }
  .trow.cur {
    background: oklch(0.7 0.17 238 / 0.09);
    border-color: oklch(0.7 0.17 238 / 0.25);
  }

  /* Index / Equalizer */
  .idx-cell {
    width: 32px;
    height: 32px;
    margin: 0 auto;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
  }
  .idx-num {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }
  .idx-play {
    display: none;
    color: var(--accent);
  }
  .trow:hover .idx-num,
  .trow:focus-visible .idx-num {
    display: none;
  }
  .trow:hover .idx-play,
  .trow:focus-visible .idx-play {
    display: inline-grid;
    place-items: center;
  }
  .trow.cur .idx-num {
    color: var(--accent);
    font-weight: 600;
  }

  .eq {
    display: inline-flex;
    align-items: flex-end;
    gap: 2.5px;
    height: 14px;
  }
  .eq i {
    display: block;
    width: 2.5px;
    border-radius: 2px;
    background: var(--accent);
  }
  .eq .b1 {
    animation: eq-bar-1 0.75s infinite ease-in-out;
  }
  .eq .b2 {
    animation: eq-bar-2 0.65s infinite ease-in-out;
  }
  .eq .b3 {
    animation: eq-bar-3 0.85s infinite ease-in-out;
  }

  /* Title & Cover */
  .title-cell {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 0;
    color: inherit;
    font-family: var(--font-sans);
    text-align: left;
    cursor: pointer;
  }
  .cover-box {
    width: 44px;
    height: 44px;
    border-radius: var(--ui-radius-sm, 9px);
    overflow: hidden;
    flex: none;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
  }
  .cover-box img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .text-meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .song-title {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .t-text {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title-cell:hover .t-text,
  .trow.cur .t-text {
    color: var(--accent);
  }

  .source-tag {
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: 5px;
    flex: none;
    background: var(--yt-soft);
    color: var(--yt-tone);
  }
  .source-tag.sc {
    background: var(--sc-soft);
    color: var(--sc-tone);
  }
  .source-tag.sp {
    background: rgba(30, 215, 96, 0.14);
    color: #1ed760;
  }
  .source-tag.offline {
    background: var(--signal-online-soft);
    color: var(--signal-online);
  }

  .song-artist {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .dot-sep,
  .album-sub {
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .wave-cell {
    min-width: 0;
    padding: 0 4px;
  }

  .dur-cell {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 12px;
    color: var(--text-muted);
    text-align: right;
  }

  /* Row actions */
  .act-cell {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 3px;
    opacity: 0.35;
    transition: opacity 0.16s ease;
  }
  .trow:hover .act-cell,
  .trow:focus-within .act-cell,
  .trow.cur .act-cell {
    opacity: 1;
  }

  .row-btn {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: inline-grid;
    place-items: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .row-btn.like-btn.liked {
    color: #ef4444;
  }
  .row-btn.like-btn:hover {
    color: #ef4444;
  }
  .row-btn:hover,
  .row-btn.active {
    background: var(--bg-surface-3);
    color: var(--accent);
  }
  .row-btn.danger:hover {
    background: oklch(0.67 0.19 25 / 0.16);
    color: var(--danger);
  }

  .dl-pill {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 2px 6px;
    border-radius: 6px;
  }

  .menu-anchor {
    position: relative;
  }
  .popover {
    position: absolute;
    right: 0;
    top: 34px;
    min-width: 180px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    padding: 6px;
    box-shadow: 0 14px 34px oklch(0 0 0 / 0.65);
    z-index: 40;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .popover-head {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    padding: 4px 8px 6px;
    border-bottom: 1px solid var(--border-subtle);
    margin-bottom: 2px;
  }
  .popover-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12.5px;
    padding: 7px 8px;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
  }
  .popover-item:hover {
    background: var(--bg-surface-3);
    color: var(--accent);
  }
  .popover-item small {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    color: var(--text-muted);
  }
  .popover-empty {
    font-size: 12px;
    color: var(--text-muted);
    padding: 8px;
  }

  @container stage (max-width: 760px) {
    .thead,
    .trow {
      grid-template-columns: 32px minmax(0, 1fr) 64px 152px;
      gap: 8px;
      padding-inline: 8px;
    }
    .col-wave,
    .wave-cell {
      display: none;
    }
  }
  @container stage (max-width: 560px) {
    .thead,
    .trow {
      grid-template-columns: 28px minmax(0, 1fr) 64px;
      gap: 8px;
    }
    .col-act {
      display: none;
    }
    .act-cell {
      grid-column: 2 / -1;
      justify-content: flex-start;
      opacity: 1;
    }
    .title-cell {
      gap: 8px;
    }
    .cover-box {
      width: 36px;
      height: 36px;
    }
  }
</style>
