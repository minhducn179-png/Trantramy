<script lang="ts">
  import { Sparkles, Pause, Play } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { lyricsState } from "./lyrics.svelte";
  onMount(() => lyricsState.subscribe());
  let confirmStart = $state(false);
  const queue = $derived(lyricsState.queue);
  const finished = $derived(queue.jobs.filter(j => ["done", "skipped", "failed", "needs_review"].includes(j.state)).length);
  const pending = $derived(queue.jobs.some(j => j.state === "pending" || j.state === "running"));
  const current = $derived(queue.jobs.find(j => j.state === "running"));
</script>

<section class="batch" aria-label="Tạo lời cho thư viện">
  <div class="actions">
    {#if queue.running}
      <button onclick={() => lyricsState.pause()}><Pause size={14} /> Tạm dừng</button>
    {:else if pending}
      <button disabled={lyricsState.busy} onclick={() => lyricsState.start(true)}><Play size={14} /> Tiếp tục tạo lời</button>
    {:else}
      <button disabled={lyricsState.busy} onclick={() => confirmStart = !confirmStart}><Sparkles size={14} /> Tạo lời tự động cho cả thư viện</button>
    {/if}
    {#if queue.jobs.length}
      <span class="counts">{finished}/{queue.jobs.length} bài · {queue.running ? "Đang chạy" : pending ? "Đã tạm dừng" : "Đã kết thúc lượt"}</span>
    {/if}
  </div>
  {#if confirmStart}
    <div class="confirmation">
      <p>Chỉ tạo lời còn thiếu, không thay lời có sẵn. Tác vụ dùng cấu hình tại <strong>Cài đặt → Lời bài hát</strong>.</p>
      <p>Nếu bạn đã cho phép cloud, âm thanh có thể được gửi đến nhà cung cấp và tính phí. Mỗi lượt dừng theo giới hạn số bài đã đặt.</p>
      <div class="actions">
        <button onclick={() => { confirmStart = false; lyricsState.start(); }}>Bắt đầu hàng đợi</button>
        <button onclick={() => confirmStart = false}>Hủy</button>
      </div>
    </div>
  {/if}
  {#if current}<p class="status" aria-live="polite">{current.track.title} · {current.message}</p>{/if}
  {#if !queue.running && pending}<p class="status">Tiến độ đã lưu. Một yêu cầu cloud đang gửi có thể cần kết thúc trước khi dừng.</p>{/if}
  {#if lyricsState.error || queue.error}<p role="alert" class="error">{lyricsState.error || queue.error}</p>{/if}
  {#if queue.jobs.length}
    <details>
      <summary>Xem kết quả và lỗi từng bài</summary>
      <ul>
        {#each queue.jobs.filter(j => j.state !== "pending" || j.message).slice(-30) as job}
          <li><strong>{job.track.title}</strong><span class:error={job.state === "failed"}>{job.message || job.state}</span></li>
        {/each}
      </ul>
    </details>
  {/if}
</section>

<style>
  .batch { padding: 14px 18px; border-bottom: 1px solid var(--border-subtle); color: var(--text-secondary); font-size: 12px; }
  .actions { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  button { display: inline-flex; align-items: center; justify-content: center; gap: 7px; padding: 8px 12px; border: 1px solid var(--border-subtle); border-radius: 7px; background: var(--bg-surface-2); color: var(--text-primary); font: inherit; cursor: pointer; }
  button:hover:not(:disabled) { border-color: var(--accent); }
  button:disabled { opacity: .5; cursor: wait; }
  button:focus-visible, summary:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
  .counts { font-variant-numeric: tabular-nums; }
  .status { margin: 8px 0 0; overflow-wrap: anywhere; }
  .confirmation { margin-top: 10px; max-width: 75ch; }
  .confirmation p { line-height: 1.6; }
  .error { color: var(--danger, var(--text-primary)); overflow-wrap: anywhere; }
  details { margin-top: 10px; }
  summary { cursor: pointer; }
  ul { list-style: none; padding: 0; max-height: 240px; overflow-y: auto; }
  li { padding: 7px 0; border-bottom: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 3px; overflow-wrap: anywhere; }
</style>
