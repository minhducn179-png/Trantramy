<script lang="ts">
  import { ListMusic, X, Plus, Play } from "@lucide/svelte";
  import { api, fmtTime, getTrackCover, handleImgError, type SpotifyCollection, type SpotifyTrackItem, type Track } from "./api";
  import { player } from "./player.svelte";

  let {
    open = $bindable(false),
    onimported,
  }: {
    open: boolean;
    onimported?: (playlistId: number, name: string) => void;
  } = $props();

  let urlInput = $state("");
  let loading = $state(false);
  let errorMsg = $state("");
  let collection = $state<SpotifyCollection | null>(null);

  // Matching state
  let matching = $state(false);
  let matchProgress = $state(0);
  let matchTotal = $state(0);

  async function handleFetch() {
    const trimmed = urlInput.trim();
    if (!trimmed) {
      errorMsg = "Vui lòng dán đường dẫn Spotify";
      return;
    }
    errorMsg = "";
    loading = true;
    collection = null;

    try {
      const res = await api.importSpotify(trimmed);
      collection = res;
      if (res.tracks.length === 0) {
        errorMsg = "Không tìm thấy bài hát nào trong liên kết này.";
      }
    } catch (err) {
      errorMsg = `Lỗi: ${err}`;
    } finally {
      loading = false;
    }
  }

  async function handlePaste() {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        urlInput = text;
        handleFetch();
      }
    } catch {}
  }

  // Create local playlist & save Spotify tracks
  async function saveAsPlaylist(andPlay = false) {
    if (!collection || collection.tracks.length === 0) return;
    matching = true;
    errorMsg = "";
    matchProgress = 0;
    matchTotal = collection.tracks.length;

    try {
      // 1. Create a playlist
      const plName = collection.title || "Spotify Imported";
      const plId = await api.createPlaylist(plName);

      // Set cover if available
      if (collection.thumbnail) {
        await api.setPlaylistCover(plId, collection.thumbnail).catch(() => {});
      }

      // 2. Add tracks into library and playlist in fast batches
      const addedTracks: Track[] = [];
      const batchSize = 6;

      for (let i = 0; i < collection.tracks.length; i += batchSize) {
        const slice = collection.tracks.slice(i, i + batchSize);
        const batchAdded = await Promise.all(
          slice.map(async (item) => {
            try {
              return await api.addOnlineToLibrary({
                title: item.title,
                artist: item.artist,
                album: item.album || collection!.title,
                duration: item.duration,
                thumbnail: item.thumbnail || collection!.thumbnail || "",
                url: item.url,
                source: "spotify",
              });
            } catch {
              return null;
            }
          }),
        );
        for (const track of batchAdded) {
          if (track) {
            await api.addToPlaylist(plId, track.id).catch(() => {});
            addedTracks.push(track);
          }
        }
        matchProgress = Math.min(collection.tracks.length, i + slice.length);
      }

      matching = false;
      onimported?.(plId, plName);

      if (andPlay && addedTracks.length > 0) {
        player.playList(addedTracks, 0);
      }

      close();
    } catch (e) {
      matching = false;
      errorMsg = `Lỗi khi lưu playlist: ${e}`;
    }
  }

  function playNow() {
    saveAsPlaylist(true);
  }

  function close() {
    open = false;
    collection = null;
    errorMsg = "";
    urlInput = "";
    matching = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <div class="modal-backdrop" onclick={close} role="presentation">
    <div
      class="modal-dialog"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => {
        e.stopPropagation();
        if (e.key === "Escape") close();
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Nhập nhạc từ Spotify"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="modal-header">
        <div class="brand-left">
          <div class="sp-icon">
            <ListMusic size={22} />
          </div>
          <div>
            <h2 class="modal-title">Nhập từ Spotify</h2>
            <p class="modal-sub">Dán link Playlist, Album hoặc Bài hát để nghe & tải về máy</p>
          </div>
        </div>

        <button class="close-btn" onclick={close} title="Đóng" aria-label="Đóng">
          <X size={18} />
        </button>
      </div>

      <!-- Input Form -->
      <div class="input-section">
        <div class="input-wrap">
          <input
            type="text"
            placeholder="Dán link Spotify (https://open.spotify.com/playlist/...)"
            bind:value={urlInput}
            onkeydown={(e) => e.key === "Enter" && handleFetch()}
            disabled={loading || matching}
            class="url-input"
            aria-label="Liên kết Spotify"
          />
          <button class="paste-btn" onclick={handlePaste} disabled={loading || matching} title="Dán từ Clipboard">
            Dán
          </button>
          <button class="fetch-btn" onclick={handleFetch} disabled={loading || matching}>
            {#if loading}
              <span class="spinner"></span>
              Đang lấy...
            {:else}
              Trích xuất
            {/if}
          </button>
        </div>

        {#if errorMsg}
          <div class="err-box">{errorMsg}</div>
        {/if}
      </div>

      <!-- Preview Section -->
      {#if collection}
        <div class="preview-area">
          <div class="collection-header">
            <div class="cover-box">
              <img
                src={getTrackCover(collection.thumbnail, collection.tracks[0]?.url, collection.title, collection.author)}
                alt=""
                onerror={(e) => handleImgError(e, collection?.title, collection?.author)}
              />
            </div>
            <div class="meta-box">
              <span class="kind-tag">{collection.kind.toUpperCase()}</span>
              <h3 class="col-title">{collection.title}</h3>
              <p class="col-author">Bởi {collection.author} · {collection.total} bài hát</p>
            </div>
          </div>

          <!-- Matching Progress bar if active -->
          {#if matching}
            <div class="match-box">
              <div class="match-info">
                <span>Đang xử lý lưu bài hát: {matchProgress} / {matchTotal}</span>
                <span class="pct">{Math.round((matchProgress / matchTotal) * 100)}%</span>
              </div>
              <div class="progress-bar">
                <div class="progress-fill" style="width: {(matchProgress / matchTotal) * 100}%"></div>
              </div>
            </div>
          {/if}

          <!-- Tracklist preview -->
          <div class="tracklist-scroll">
            {#each collection.tracks as t, i}
              <div class="track-row">
                <span class="t-idx">{i + 1}</span>
                <div class="t-cover">
                  <img
                    src={getTrackCover(t.thumbnail, t.url, t.title, t.artist)}
                    alt=""
                    onerror={(e) => handleImgError(e, t.title, t.artist, t.url)}
                  />
                </div>
                <div class="t-text">
                  <span class="t-name">{t.title}</span>
                  <span class="t-art">{t.artist}</span>
                </div>
                <span class="t-dur">{fmtTime(t.duration)}</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="modal-footer">
          <button class="action-btn secondary" onclick={() => saveAsPlaylist(false)} disabled={matching}>
            <Plus size={16} />
            Lưu vào Playlist
          </button>

          <button class="action-btn primary" onclick={playNow} disabled={matching}>
            <Play size={16} />
            Lưu & Phát ngay
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: oklch(0 0 0 / 0.62);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-dialog {
    width: 100%;
    max-width: 620px;
    max-height: calc(100dvh - 32px);
    background: var(--bg-surface-1);
    border: 1px solid var(--border-strong);
    border-radius: calc(var(--ui-radius, 12px) + 4px);
    box-shadow: 0 24px 60px oklch(0 0 0 / 0.65);
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    animation: scaleUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .modal-dialog > * {
    flex-shrink: 0;
  }

  @keyframes scaleUp {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 1.15rem 1.4rem 0.9rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
  }

  .brand-left {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.85rem;
  }

  .sp-icon {
    width: 38px;
    height: 38px;
    border-radius: var(--ui-radius, 10px);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .modal-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.015em;
  }

  .modal-sub {
    margin: 0.15rem 0 0;
    font-size: 0.76rem;
    color: var(--text-secondary);
  }

  .close-btn {
    flex-shrink: 0;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.38rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-3);
    border-color: var(--border-strong);
  }

  .input-section {
    padding: 1.1rem 1.4rem 0.9rem;
  }

  .input-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--ui-radius, 10px);
    padding: 0.35rem 0.45rem;
    transition: border-color 0.15s ease;
  }

  .input-wrap:focus-within {
    border-color: var(--accent);
  }

  .url-input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 0.84rem;
    padding: 0.4rem 0.6rem;
    outline: none;
  }

  .url-input::placeholder {
    color: var(--text-muted);
  }

  .paste-btn,
  .fetch-btn {
    border: none;
    border-radius: 7px;
    font-family: var(--font-sans);
    font-size: 0.78rem;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .paste-btn {
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: 0.45rem 0.75rem;
  }

  .paste-btn:hover {
    background: var(--bg-surface-3);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .fetch-btn {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    padding: 0.45rem 1rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
    box-shadow: 0 2px 10px var(--accent-glow);
  }

  .fetch-btn:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }

  .fetch-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid oklch(0.09 0.015 240 / 0.3);
    border-top-color: var(--accent-fg, oklch(0.09 0.015 240));
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .err-box {
    margin-top: 0.75rem;
    padding: 0.6rem 0.85rem;
    border-radius: 8px;
    background: oklch(0.67 0.19 25 / 0.14);
    border: 1px solid oklch(0.67 0.19 25 / 0.35);
    color: var(--danger);
    font-size: 0.8rem;
    font-weight: 600;
  }

  .preview-area {
    flex: 1;
    overflow-y: auto;
    padding: 0 1.4rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .collection-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.8rem;
    background: var(--bg-canvas);
    border-radius: var(--ui-radius, 12px);
    border: 1px solid var(--border-subtle);
  }

  .cover-box {
    width: 64px;
    height: 64px;
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .cover-box img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .meta-box {
    flex: 1;
    min-width: 0;
  }

  .kind-tag {
    font-family: var(--font-mono);
    font-size: 0.65rem;
    font-weight: 800;
    letter-spacing: 0.06em;
    color: var(--accent);
  }

  .col-title {
    margin: 0.15rem 0;
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .col-author {
    margin: 0;
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .match-box {
    padding: 0.75rem 1rem;
    border-radius: 8px;
    background: var(--accent-soft);
    border: 1px solid var(--accent);
  }

  .match-info {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: 0.4rem;
  }

  .progress-bar {
    height: 6px;
    border-radius: 999px;
    background: var(--bg-canvas);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.15s ease;
  }

  .tracklist-scroll {
    max-height: 240px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    background: var(--bg-canvas);
    border: 1px solid var(--border-subtle);
    border-radius: var(--ui-radius, 10px);
    padding: 0.4rem;
  }

  .track-row {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    padding: 0.35rem 0.6rem;
    border-radius: 7px;
    transition: background 0.12s ease;
  }

  .track-row:hover {
    background: var(--bg-surface-2);
  }

  .t-idx {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-muted);
    width: 22px;
    text-align: center;
  }

  .t-cover {
    width: 34px;
    height: 34px;
    border-radius: 6px;
    overflow: hidden;
    flex-shrink: 0;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
  }

  .t-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .t-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .t-name {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .t-art {
    font-size: 0.72rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .t-dur {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 0.9rem 1.4rem 1.1rem;
    border-top: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.55rem 1.1rem;
    border-radius: 8px;
    font-family: var(--font-sans);
    font-size: 0.82rem;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.12s ease;
    border: none;
  }

  .action-btn.secondary {
    background: var(--bg-surface-2);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
  }

  .action-btn.secondary:hover {
    background: var(--bg-surface-3);
    border-color: var(--border-strong);
  }

  .action-btn.primary {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    box-shadow: 0 3px 12px var(--accent-glow);
  }

  .action-btn.primary:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  @media (max-width: 600px) {
    .modal-header,
    .input-section,
    .preview-area,
    .modal-footer {
      padding-inline: 16px;
    }
    .input-wrap {
      flex-wrap: wrap;
    }
    .url-input {
      flex-basis: 100%;
    }
    .modal-footer {
      flex-wrap: wrap;
      gap: 10px;
    }
    .modal-title {
      font-size: 16px;
    }
  }
</style>
