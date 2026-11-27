<script lang="ts">
  import { X, Mic, PenLine, Sparkles, Save, WandSparkles, Search, Settings2, Check, ArrowRight, AudioLines, Trash2, RotateCcw } from "@lucide/svelte";
  import { onMount, tick } from "svelte";
  import { api, fmtTime, type Track, type LyricsResult } from "$lib/api";
  import { player } from "$lib/player/player.svelte";
  import { lyricsState, lyricDrafts, lyricFreshDrafts } from "./lyrics.svelte";
  import { parseLrc, activeIndex, wordFillPercent, autoGenerateTimedLyrics, correctLyricsText, getLrcOffset, setLrcOffset, alignLyricsToTranscript, checkLyricsTiming } from "./lrc";

  let { onclose, onsettings, track, embedded = false }: { onclose?: () => void; onsettings?: () => void; track?: Track; embedded?: boolean } = $props();
  const current = $derived(track ?? player.current);
  const isCurrent = $derived(Boolean(current && player.current?.path === current.path));
  let result = $state<LyricsResult | null>(null);
  let loading = $state(false);
  let generating = $state(false);
  let saving = $state(false);
  let message = $state("");
  let error = $state("");
  let readError = $state("");
  let showEditor = $state(false);
  let editorContent = $state("");
  let box = $state<HTMLDivElement>();
  let currentPos = $state(0);
  let reviewing = $state(false);
  let alignAudio = $state(false);
  let preview = $state<{ before: string; content: string; changed: number; aligned: boolean } | null>(null);
  let stage = $state("");
  let selectionVersion = 0;
  let seenDeletion = lyricsState.deletion?.version ?? 0;
  let deleting = $state(false);
  let confirmation = $state<"delete" | "rewrite" | null>(null);
  let replacement = $state<LyricsResult | null>(null);
  const busy = $derived(generating || reviewing || saving || deleting);
  const hasUnclear = $derived(editorContent.includes('⟦không nghe rõ⟧'));
  const pipelineProgress = $derived(lyricsState.progress?.path === current?.path && (loading || generating) ? lyricsState.progress : null);
  const timingIssues = $derived(checkLyricsTiming(showEditor ? editorContent : result?.content ?? "", current?.duration ?? 0));
  const changes = $derived.by(() => {
    if (!preview) return [];
    const before = parseLrc(preview.before).lines;
    return parseLrc(preview.content).lines.map((line, index) => ({ before: before[index]?.text ?? "", after: line.text })).filter(line => line.before !== line.after);
  });
  let token = 0;
  let loadedPath = "";
  let karaokeMode = $state(typeof localStorage === "undefined" || localStorage.getItem("dlc.lyrics.karaoke") !== "0");
  const lyrics = $derived(result ? parseLrc(result.content) : null);
  const active = $derived(isCurrent && lyrics?.synced ? activeIndex(lyrics.lines, currentPos) : -1);
  const sourceLabel: Record<string,string> = { lrc:"File LRC",manual:"Bạn chỉnh sửa",lrclib:"LRCLIB",lyricsplus:"LyricsPlus","youtube-cc":"Phụ đề YouTube","youtube-auto":"Phụ đề tự động",groq:"Groq Whisper",openai:"OpenAI Whisper",gemini:"Gemini","whisper.cpp":"Whisper offline","legacy-cache":"Cache cũ" };
  onMount(() => lyricsState.subscribe());

  function draft(value: string) {
    editorContent = value;
    preview = null;
    if (current) lyricDrafts.set(current.path, value);
  }
  function openEditor() {
    if (!current) return;
    editorContent = lyricDrafts.get(current.path) ?? result?.content ?? "";
    showEditor = true;
    preview = null;
  }
  async function removeSaved() {
    const selected = current;
    if (!selected || busy || confirmation !== "delete") return;
    deleting = true; error = ""; message = "";
    try {
      await api.deleteLyrics(selected);
      lyricsState.forget(selected.path);
    } catch (e) { if (current?.path === selected.path) error = String(e); }
    finally { deleting = false; confirmation = null; }
  }
  async function rewrite() {
    const selected = current;
    const version = selectionVersion;
    if (!selected || busy || confirmation !== "rewrite") return;
    generating = true; error = ""; message = "Đang nhận diện lại từ âm thanh, không dùng lời đã lưu…";
    confirmation = null; replacement = null; preview = null;
    try {
      const found = await api.generateLyrics(selected, false, true);
      if (version !== selectionVersion || current?.path !== selected.path) return;
      replacement = found;
      message = "Đây là bản viết lại để xem trước. Bản đã lưu và bản nháp cũ chưa bị thay đổi.";
    } catch (e) { if (version === selectionVersion && current?.path === selected.path) { error = String(e); message = "Chưa viết lại được. Bản đã lưu và bản nháp được giữ nguyên."; } }
    finally { generating = false; }
  }
  async function applyReplacement() {
    if (!current || !replacement || busy) return;
    lyricFreshDrafts.add(current.path);
    draft(replacement.content);
    if (replacement.draft_only) result = replacement;
    message = replacement.draft_only ? "Đã chọn bản nháp chưa rõ để sửa; chưa thay lời đã lưu." : "Đã chọn bản viết lại làm bản nháp. Lưu lời mới thay bản đang lưu.";
    replacement = null; showEditor = true;
    await tick();
    const editor = box?.querySelector('textarea');
    editor?.focus();
    editor?.scrollIntoView({ block: 'nearest' });
  }
  async function review() {
    const selected = current;
    const version = selectionVersion;
    const submitted = editorContent;
    if (!selected || busy || !submitted.trim() || hasUnclear) return;
    reviewing = true; error = ""; message = ""; preview = null;
    let reviewed: { content: string; changed_lines: number } | null = null;
    try {
      const content = !lyricFreshDrafts.has(selected.path) && !alignAudio && !parseLrc(submitted).synced && result && parseLrc(result.content).synced
        ? correctLyricsText(result.content, submitted) : submitted;
      stage = "Đang rà chính tả và dấu câu…";
      reviewed = await api.reviewLyrics(content);
      if (version !== selectionVersion || current?.path !== selected.path) return;
      if (editorContent !== submitted) { message = "Bạn vừa sửa lời. Kết quả AI không thay bản nháp; hãy rà soát lại khi sửa xong."; return; }
      if (alignAudio) {
        stage = "Đang nhận diện giọng hát để đối chiếu mốc…";
        const transcript = await api.transcribeLyrics(selected);
        if (version !== selectionVersion || current?.path !== selected.path) return;
        if (editorContent !== submitted) { message = "Bản nháp vừa thay đổi. Không áp dụng mốc từ tác vụ cũ."; return; }
        if (transcript.timing !== "word") throw new Error("Nguồn nhận diện chưa có mốc từng chữ. Chọn Whisper offline, Groq hoặc OpenAI trong thiết lập.");
        preview = { before: content, content: alignLyricsToTranscript(reviewed.content, transcript.content, selected.duration), changed: reviewed.changed_lines, aligned: true };
      } else preview = { before: content, content: reviewed.content, changed: reviewed.changed_lines, aligned: false };
      message = "Rà soát xong. Xem thay đổi rồi áp dụng vào bản nháp; chưa ghi đè lời đã lưu.";
    } catch (e) {
      if (version === selectionVersion && current?.path === selected.path) {
        error = String(e);
        if (reviewed && editorContent === submitted) {
          preview = { before: submitted, content: reviewed.content, changed: reviewed.changed_lines, aligned: false };
          message = "Đã rà soát chữ nhưng chưa căn được theo âm thanh. Bạn vẫn có thể xem và áp dụng phần sửa chữ.";
        }
      }
    } finally { reviewing = false; stage = ""; }
  }
  function applyPreview() {
    if (!preview) return;
    const aligned = preview.aligned;
    draft(preview.content);
    message = aligned ? "Đã đưa mốc nhận diện vào bản nháp. Nghe kiểm tra rồi Lưu lời." : "Đã áp dụng sửa chữ, giữ nguyên mốc cũ. Nghe kiểm tra rồi Lưu lời.";
  }
  function toggleKaraoke() {
    karaokeMode = !karaokeMode;
    localStorage.setItem("dlc.lyrics.karaoke",karaokeMode ? "1" : "0");
  }
  async function generate() {
    const selected = current;
    if (!selected || busy) return;
    const version = selectionVersion;
    const original = editorContent;
    generating = true; error = ""; message = "Đang tạo lời theo cấu hình đã lưu; quy trình nhiều bước sẽ hiện tiến độ bên dưới…";
    try {
      const found = await api.generateLyrics(selected);
      if (version !== selectionVersion || current?.path !== selected.path) return;
      result = found;
      if (editorContent === original) { draft(found.content); showEditor = true; }
      message = found.draft_only ? "Đã giữ bản nháp có chỗ chưa nghe rõ. Nghe và sửa trước khi dùng." : found.notice ?? "Đã có lời và mốc từ nguồn đã chọn. Bật tự rà soát trong Thiết lập AI hoặc dùng AI rà soát bên dưới.";
    } catch (e) { if (version === selectionVersion && current?.path === selected.path) { error = String(e); message = ""; } }
    finally { generating = false; }
  }
  async function lookup() {
    const selected = current;
    if (!selected || busy) return;
    const version = selectionVersion;
    if (lyricDrafts.has(selected.path)) {
      error = "Bài này có bản nháp. Hãy lưu bản nháp trước khi tìm lời khác.";
      return;
    }
    const original = editorContent;
    generating = true; error = ""; message = "Đang tìm lời có sẵn, không gửi âm thanh…";
    try {
      const found = await api.generateLyrics(selected, true);
      if (version !== selectionVersion || current?.path !== selected.path) return;
      if (editorContent !== original || lyricDrafts.has(selected.path)) {
        message = "Bạn vừa sửa bản nháp nên kết quả tra cứu không được áp dụng. Lưu bản nháp rồi tìm lại nếu cần.";
        return;
      }
      result = found;
      lyricFreshDrafts.add(selected.path);
      draft(found.content); showEditor = true;
      message = "Đây là bản xem trước. Bấm Lưu lời để chọn bản này. Bản đang lưu chưa bị thay thế.";
    } catch (e) { if (version === selectionVersion && current?.path === selected.path) { error = String(e); message = ""; } }
    finally { generating = false; }
  }
  async function save() {
    const selected = current;
    const submitted = editorContent;
    if (!selected || !submitted.trim() || busy || preview || replacement || hasUnclear) return;
    const version = selectionVersion;
    saving = true; error = "";
    try {
      const content = !lyricFreshDrafts.has(selected.path) && !parseLrc(submitted).synced && result && parseLrc(result.content).synced
        ? correctLyricsText(result.content, submitted) : submitted;
      await api.saveLyrics(selected.path,selected.title,selected.artist,selected.album,selected.duration,content);
      if (version !== selectionVersion || current?.path !== selected.path) return;
      const parsed = parseLrc(content);
      result = { content, source:"manual",timing:parsed.hasWordSync ? "word" : parsed.synced ? "line" : "plain",review:false };
      if (editorContent === submitted) { editorContent = content; lyricDrafts.delete(selected.path); lyricFreshDrafts.delete(selected.path); showEditor = false; }
      message = "Đã lưu bản chỉnh sửa trong app. File .lrc gốc được giữ nguyên.";
    } catch(e) { if (version === selectionVersion && current?.path === selected.path) error = String(e); }
    finally { saving = false; }
  }
  function estimate() {
    if (!current || !editorContent.trim() || hasUnclear) return;
    const plain = parseLrc(editorContent).lines.map(l => l.text).join("\n");
    const duration = isCurrent && player.duration > 0 ? player.duration : current.duration;
    if (!Number.isFinite(duration) || duration <= 0) { error = "Chưa biết thời lượng. Phát hoặc tải bài về máy trước."; return; }
    draft(autoGenerateTimedLyrics(plain,duration,current.title,current.artist));
    message = "Đây chỉ là mốc thời gian ước lượng theo văn bản, không phải căn chỉnh theo giọng hát.";
  }
  function changeOffset(value: number) {
    try {
      draft(setLrcOffset(editorContent, value));
      error = "";
      message = "Lưu lời để áp dụng độ bù. Số dương chạy lời sớm hơn, số âm chạy muộn hơn.";
    } catch (e) { error = String(e); }
  }
  function seek(time: number) {
    if (!current) return;
    if (!isCurrent) { message = "Phát bài này trước khi chuyển đến câu hát."; return; }
    player.seek(time);
  }

  $effect(() => {
    const selected = current;
    void lyricsState.revision;
    const deletion = lyricsState.deletion;
    if (deletion && deletion.version !== seenDeletion) {
      seenDeletion = deletion.version;
      if (deletion.path === selected?.path) {
        selectionVersion++;
        result = null; editorContent = ""; showEditor = false;
        preview = null; replacement = null; confirmation = null;
        error = ""; readError = "";
        message = "Đã xóa lời trong app. File .lrc gốc được giữ nguyên. Chọn Viết lại bằng AI hoặc nhập lời mới khi cần.";
      }
    }
    const mine = ++token;
    if (loadedPath !== (selected?.path ?? "")) {
      loadedPath = selected?.path ?? "";
      selectionVersion++;
      preview = null; replacement = null; confirmation = null; alignAudio = false;
      result = null; error = ""; readError = ""; message = "";
      editorContent = selected ? lyricDrafts.get(selected.path) ?? "" : "";
      showEditor = Boolean(selected && lyricDrafts.has(selected.path));
    }
    if (!selected) { loading = false; return; }
    loading = true;
    api.readLyrics(selected).then(found => {
      if (mine !== token) return;
      result = found;
      readError = "";
    }).catch(e => { if (mine===token) readError = String(e); })
      .finally(() => { if (mine===token) loading = false; });
    return () => { if (mine===token) token++; };
  });
  $effect(() => {
    if (!isCurrent || !player.playing) { currentPos = isCurrent ? player.position : 0; return; }
    let frame = 0;
    const tick = () => { currentPos = player.audio?.currentTime ?? player.position; frame = requestAnimationFrame(tick); };
    tick(); return () => cancelAnimationFrame(frame);
  });
  $effect(() => {
    if (active < 0 || !box) return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    box.querySelector<HTMLElement>(`[data-line="${active}"]`)?.scrollIntoView({block:"nearest",behavior:reduced ? "instant" : "smooth"});
  });
</script>

<aside class="lyrics-panel" class:embedded aria-label="Lời bài hát">
  <header>
    <div class="heading"><strong>Lời bài hát</strong>
      <div class="controls">
        {#if lyrics?.synced}<button class:enabled={karaokeMode} onclick={toggleKaraoke} aria-pressed={karaokeMode} title="Bật/tắt chạy từng chữ"><Mic size={14} /><span>Chữ chạy</span></button>{/if}
        {#if current}<button onclick={lookup} disabled={busy} title="Tìm lời có sẵn · xem trước, không gửi âm thanh"><Search size={14} /><span>{generating ? "Đang tìm…" : "Tìm lời"}</span></button>{/if}
        {#if current}<button onclick={openEditor} disabled={busy} title="Chỉnh sửa lời bài hát" aria-label="Chỉnh sửa lời bài hát"><PenLine size={14} /></button>{/if}
        {#if onclose}<button onclick={onclose} title="Đóng lời bài hát" aria-label="Đóng lời bài hát"><X size={14} /></button>{/if}
      </div>
    </div>
    {#if current}<p class="track-title">{current.title}</p>{/if}
    {#if onsettings}<button class="settings-link" onclick={onsettings}><Settings2 size={13} />Thiết lập AI</button>{/if}
    {#if result}
      <p class="provenance">{sourceLabel[result.source] ?? result.source}{result.review ? " · Cần kiểm tra" : ""}</p>
      {#if result.timing === "estimated"}<p class="hint">Thời gian do AI ước lượng.</p>
      {:else if lyrics?.synced && !lyrics.hasWordSync}<p class="hint">Có mốc theo câu. Chạy từng chữ là ước lượng.</p>{/if}
    {/if}
  </header>
  {#if message}<p class="notice" role="status">{message}</p>{/if}
  {#if result?.notice && !result.draft_only && !message}<p class="notice" role="status">{result.notice}</p>{/if}
  {#if pipelineProgress}<p class="working" role="status"><AudioLines size={16} />{pipelineProgress.stage > 0 ? `${pipelineProgress.stage}/${pipelineProgress.total ?? 5} · ` : ''}{pipelineProgress.message}</p>
  {:else if reviewing}<p class="working" role="status"><AudioLines size={16} />{stage}</p>{/if}
  {#if error || readError}<p class="error" role="alert">{error || readError}</p>{/if}
  <div class="teleprompter" bind:this={box}>
    {#if result?.draft_only}<p class="hint" role="status">{result.notice || 'Bản nháp còn chỗ chưa nghe rõ · chưa phải lyric hoàn chỉnh. Mở chỉnh sửa hoặc Viết lại bằng AI để xử lý.'}</p>{/if}
    {#if current}
      <div class="manage-actions">
        <button onclick={() => confirmation = "rewrite"} disabled={busy}><RotateCcw size={14} />Viết lại bằng AI</button>
        {#if result || editorContent.trim()}<button class="delete-action" onclick={() => confirmation = "delete"} disabled={busy}><Trash2 size={14} />Xóa lời đã lưu</button>{/if}
      </div>
      {#if confirmation}
        <section class="action-confirm" aria-label={confirmation === "delete" ? "Xác nhận xóa lời" : "Xác nhận viết lại lời"}>
          <h4>{confirmation === "delete" ? "Xóa lời của bài này?" : "Nhận diện lại từ đầu?"}</h4>
          <p class="hint">{confirmation === "delete" ? "Xóa bản lưu và bản nháp trong app. Không xóa file .lrc gốc. App sẽ không tự lấy lại lời khi mở bài hoặc chạy thư viện." : "AI sẽ nghe lại theo cấu hình đã lưu, gồm quy trình nhiều bước nếu đã bật. Âm thanh có thể được gửi nhiều lượt và cloud có thể tính phí. Bản cũ chỉ bị thay khi bạn chọn bản mới và Lưu lời."}</p>
          <div class="editor-actions">
            {#if confirmation === "delete"}<button class="delete-action" onclick={removeSaved} disabled={busy}>{deleting ? "Đang xóa…" : "Xác nhận xóa lời"}</button>
            {:else}<button class="primary" onclick={rewrite} disabled={busy}>Bắt đầu viết lại</button>{/if}
            <button onclick={() => confirmation = null} disabled={busy}>Hủy</button>
          </div>
        </section>
      {/if}
      {#if replacement}
        <section class="replacement-preview" aria-label="Bản viết lại bằng AI">
          <h4>{replacement.draft_only ? 'Bản nháp · còn chỗ chưa nghe rõ' : 'Bản viết lại · chưa lưu'}</h4>
          {#if replacement.notice}<p class="hint">{replacement.notice}</p>{/if}
          <div class="replacement-text">{#each parseLrc(replacement.content).lines as line}<p>{line.text}</p>{/each}</div>
          <p class="hint">AI có thể nhận nhầm. Dùng bản này sẽ thay bản nháp; Lưu lời mới thay bản đã lưu.</p>
          <div class="editor-actions"><button class="primary" onclick={applyReplacement} disabled={busy}>{replacement.draft_only ? 'Chỉnh sửa bản nháp' : 'Dùng bản viết lại'}</button><button onclick={() => replacement = null} disabled={busy}>Giữ bản cũ</button></div>
        </section>
      {/if}
    {/if}
    {#if !current}
      <div class="empty"><p>Chọn một bài hát để xem hoặc tạo lời.</p></div>
    {:else if showEditor}
      <div class="editor">
        <div class="workflow-heading"><h3>Hoàn thiện lời bài hát</h3><p>Rà chữ trước. Chỉ nghe lại âm thanh khi cần căn mốc.</p></div>
        <div class="review-tools">
          <button class="primary" onclick={review} disabled={busy || !editorContent.trim() || hasUnclear}><Sparkles size={15} />{reviewing ? "Đang rà soát…" : "AI rà soát"}</button>
          <label class="check"><input type="checkbox" bind:checked={alignAudio} disabled={busy} /> Căn lại theo giọng hát</label>
          <p class="hint">{alignAudio ? "Sẽ nhận diện lại âm thanh theo AI đã cấu hình. Cloud có thể tính phí. Không tự gán mốc khi thiếu chữ hoặc có chữ lặp không rõ." : "Chỉ gửi chữ đến Gemini. Không tải hay nhận diện lại âm thanh; giữ mốc cũ."}</p>
        </div>
        {#if preview}
          <section class="review-preview" aria-label="Kết quả rà soát">
            <h4><Check size={15} />{preview.changed} dòng được sửa{preview.aligned ? " · Đã đối chiếu mốc" : ""}</h4>
            {#if changes.length}<div class="change-list">{#each changes as change}<div class="change-row"><del>{change.before}</del><ArrowRight size={12} /><ins>{change.after}</ins></div>{/each}</div>
            {:else}<p class="hint">AI không đề xuất thay đổi chữ. Điều này không bảo đảm lời hoàn toàn chính xác.</p>{/if}
            <p class="hint">{preview.aligned ? "Mốc lấy từ giọng hát nhận diện, vẫn có thể lệch. Nghe lại trước khi lưu." : "Chưa kiểm chứng âm thanh. Mốc thời gian được giữ nguyên."}</p>
            <div class="editor-actions"><button class="primary" onclick={applyPreview}>Áp dụng bản nháp</button><button onclick={() => preview = null}>Bỏ kết quả</button></div>
          </section>
        {/if}
        <label for={embedded ? "lyrics-editor-detail" : "lyrics-editor-panel"}>Chỉnh sửa lời · mỗi câu một dòng</label>
        <textarea id={embedded ? "lyrics-editor-detail" : "lyrics-editor-panel"} value={editorContent} oninput={e => draft(e.currentTarget.value)} disabled={busy} lang="vi" spellcheck="true" placeholder="Dán lời đúng chính tả hoặc dùng Tạo lời tự động…"></textarea>
        {#if hasUnclear}<p class="hint" role="status">Còn dấu ⟦không nghe rõ⟧. Nghe lại và điền phần đã xác nhận trước khi rà chữ, tạo mốc hoặc lưu; không dùng ngữ cảnh để đoán.</p>{/if}
        {#if timingIssues.length}<ul class="timing-hints">{#each timingIssues as issue}<li>{issue}</li>{/each}</ul>{:else if parseLrc(editorContent).synced}<p class="hint">Chưa phát hiện lỗi mốc cơ bản, chưa bảo đảm khớp nhạc. Bấm vào câu trong Xem lời để nghe thử.</p>{/if}
        <details><summary>Cách sửa tay và giữ mốc</summary><p class="hint">Giữ nguyên timestamp trong LRC. Nếu dán chữ thuần, giữ số dòng và số chữ mỗi dòng để giữ mốc cũ. Thêm/bớt chữ cần căn lại. Bản nháp được giữ khi đổi bài, nhưng mất khi đóng app.</p></details>
        {#if parseLrc(editorContent).synced}
          <label class="offset-field">Bù thời gian (ms) · dương: sớm, âm: muộn
            <input type="number" min="-10000" max="10000" step="50" value={getLrcOffset(editorContent)} onchange={e => changeOffset(e.currentTarget.valueAsNumber)} />
          </label>
        {/if}
        <div class="editor-actions">
          <button onclick={save} disabled={busy || Boolean(preview) || Boolean(replacement) || !editorContent.trim() || hasUnclear}><Save size={14} />{saving ? "Đang lưu…" : "Lưu lời"}</button>
          <details class="advanced"><summary>Mốc ước lượng</summary><p class="hint">Chia mốc theo độ dài chữ, không nghe nhạc. Không dùng để kiểm chứng khớp giọng hát.</p><button onclick={estimate} disabled={busy || !editorContent.trim() || hasUnclear}><WandSparkles size={14} />Tạo mốc ước lượng</button></details>
          <button onclick={() => showEditor = false}>Xem lời</button>
        </div>
        {#if !result}<button onclick={generate} disabled={busy}><Sparkles size={14} />{generating ? "Đang tạo…" : "Tạo lời tự động"}</button>{/if}
        <p class="hint">Cấu hình cloud và tải model tại Cài đặt → Lời bài hát. Không tự gửi âm thanh nếu chưa cho phép.</p>
      </div>
    {:else if loading && !lyrics}
      <div class="empty" role="status"><AudioLines size={24} /><p>Đang tìm lời khớp bài…</p><p class="hint">Ưu tiên bản có sẵn, chỉ dùng nhận diện khi cần và đã cấu hình.</p></div>
    {:else if !lyrics || lyrics.lines.length === 0}
      <div class="empty">
        <AudioLines size={28} class="empty-icon" />
        <h3>Cho lời hát chạy cùng nhạc</h3>
        <p class="hint">Bắt đầu từ bài đang chọn. Không cần biết định dạng LRC.</p>
        <ol class="workflow-steps"><li>Tạo hoặc dán lời</li><li>AI rà soát</li><li>Nghe và lưu</li></ol>
        <button class="start-choice primary" onclick={generate} disabled={busy}><Sparkles size={18} /><span><strong>{generating ? "Đang xử lý…" : "Tạo lời tự động"}</strong><small>Tìm bản có sẵn trước, nhận diện khi cần</small></span><ArrowRight size={15} /></button>
        <button class="start-choice" onclick={openEditor} disabled={busy}><PenLine size={18} /><span><strong>Tôi đã có lời</strong><small>Dán lời để rà soát và căn theo giọng hát</small></span><ArrowRight size={15} /></button>
        <p class="hint">AI có thể nghe nhầm. Bật tự rà soát và quyền gửi lời trong Thiết lập AI.</p>
      </div>
    {:else}
      <button class="review-entry" onclick={openEditor} disabled={busy}><Sparkles size={16} /><span><strong>Rà soát lời có sẵn</strong><small>Sửa chính tả, dấu câu · giữ mốc cũ</small></span><ArrowRight size={14} /></button>
      <div class="lines-wrap">
        {#each lyrics.lines as line, index}
          {#if lyrics.synced}
            <div class="lyric-line" class:active={index===active} class:past={index<active} data-line={index}>
              {#if index===active && karaokeMode && line.words.length}
                <div class="word-line" aria-label={line.text}>
                  {#each line.words as word}
                    {@const fill = wordFillPercent(word,currentPos)}
                    <button class="word" class:done={fill>=100} class:singing={fill>0 && fill<100} style:--fill={`${fill}%`} onclick={() => seek(word.start)} aria-label={`Phát từ ${word.text}`}><span>{word.text}</span></button>{" "}
                  {/each}
                </div>
                <button class="time" onclick={() => seek(line.time)} aria-label={`Phát câu tại ${fmtTime(line.time)}`}>{fmtTime(line.time)}</button>
              {:else}
                <button class="line-button" onclick={() => seek(line.time)}><span>{line.text || "•••"}</span><small>{fmtTime(line.time)}</small></button>
              {/if}
            </div>
          {:else}<p class="plain">{line.text || "\u00a0"}</p>{/if}
        {/each}
      </div>
    {/if}
  </div>
</aside>

<style>
  .lyrics-panel { width: 350px; max-width: 100vw; min-width: 0; flex-shrink: 0; display: flex; flex-direction: column; min-height: 0; border-left: 1px solid var(--border-subtle); background: var(--bg-surface-1); color: var(--text-primary); }
  .lyrics-panel.embedded { width: 100%; border: 1px solid var(--border-subtle); border-radius: 12px; max-height: 540px; min-height: 320px; }
  header { padding: 16px 18px; border-bottom: 1px solid var(--border-subtle); }
  .heading { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
  .heading strong { font-size: 14px; }
  .heading .controls { margin-left: auto; flex-wrap: wrap; justify-content: flex-end; }
  .controls { display: flex; align-items: center; gap: 5px; }
  button { display: inline-flex; align-items: center; justify-content: center; gap: 6px; padding: 7px 9px; border: 1px solid var(--border-subtle); border-radius: 6px; color: var(--text-primary); background: var(--bg-surface-2); font: inherit; font-size: 12px; cursor: pointer; }
  button:hover:not(:disabled) { border-color: var(--accent); }
  button:disabled { opacity: .5; cursor: wait; }
  button:focus-visible, textarea:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  button.enabled { color: var(--accent); }
  .track-title { margin: 8px 0 0; font-size: 13px; overflow-wrap: anywhere; }
  .provenance { margin: 5px 0 0; font-size: 11px; color: var(--text-secondary); }
  .hint { color: var(--text-secondary); font-size: 12px; line-height: 1.6; margin: 6px 0; overflow-wrap: anywhere; }
  .notice, .error { padding: 8px 18px; margin: 0; font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  .error { color: var(--danger, var(--text-primary)); }
  .teleprompter { flex: 1; min-height: 0; overflow-y: auto; scrollbar-gutter: stable; padding: 16px; }
  .empty { display: flex; flex-direction: column; align-items: flex-start; gap: 8px; padding: 16px 0; font-size: 14px; }
  .editor { display: flex; flex-direction: column; gap: 10px; font-size: 12px; }
  textarea { min-height: 230px; width: 100%; resize: vertical; padding: 10px; box-sizing: border-box; border: 1px solid var(--border-subtle); border-radius: 7px; background: var(--bg-surface-2); color: var(--text-primary); font: 12px/1.7 var(--font-mono,monospace); }
  .offset-field { display: flex; flex-direction: column; gap: 6px; }
  .offset-field input { width: 100%; box-sizing: border-box; padding: 8px 10px; border: 1px solid var(--border-subtle); border-radius: 6px; background: var(--bg-surface-2); color: var(--text-primary); font: inherit; font-variant-numeric: tabular-nums; }
  .offset-field input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .editor-actions { display: flex; flex-wrap: wrap; gap: 6px; }
  .manage-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 14px; }
  .delete-action { color: var(--danger, var(--text-primary)); }
  .action-confirm, .replacement-preview { padding: 14px 0; margin-bottom: 16px; border-block: 1px solid var(--border-subtle); }
  .replacement-text { max-height: 220px; overflow-y: auto; overflow-wrap: anywhere; font-size: 13px; line-height: 1.7; }
  .replacement-text p { margin: 6px 0; }
  .settings-link { margin-top: 10px; background: transparent; color: var(--text-secondary); }
  h3 { margin: 0; font-size: 18px; line-height: 1.45; letter-spacing: -.02em; }
  .workflow-heading p { margin: 6px 0 0; color: var(--text-secondary); line-height: 1.6; }
  .workflow-steps { display: flex; flex-wrap: wrap; gap: 6px 18px; padding-left: 18px; margin: 10px 0 16px; font-size: 11px; color: var(--text-secondary); }
  .primary { background: var(--accent); color: var(--bg-base, var(--bg-surface-1)); border-color: var(--accent); font-weight: 700; }
  .start-choice, .review-entry { width: 100%; justify-content: flex-start; text-align: start; gap: 10px; padding: 13px 12px; }
  .start-choice span, .review-entry span { flex: 1; min-width: 0; }
  .start-choice strong, .review-entry strong { display: block; font-size: 12px; }
  .start-choice small, .review-entry small { display: block; margin-top: 5px; font-size: 11px; line-height: 1.5; }
  .review-entry small { color: var(--text-secondary); }
  .review-tools { display: flex; flex-direction: column; align-items: flex-start; gap: 10px; padding: 14px 0; border-block: 1px solid var(--border-subtle); }
  .check { display: flex; align-items: center; gap: 8px; line-height: 1.6; }
  .check input { accent-color: var(--accent); }
  .review-preview { padding-block: 12px; border-bottom: 1px solid var(--border-subtle); }
  h4 { display: flex; align-items: center; gap: 6px; margin: 0 0 12px; font-size: 13px; }
  .change-list { max-height: 220px; overflow-y: auto; }
  .change-row { display: grid; gap: 5px; padding: 10px 0; border-top: 1px solid var(--border-subtle); line-height: 1.6; overflow-wrap: anywhere; }
  del { color: var(--text-secondary); }
  ins { text-decoration: none; font-weight: 600; }
  .timing-hints { margin: 0; padding-left: 18px; color: var(--text-secondary); line-height: 1.7; }
  summary { cursor: pointer; color: var(--text-secondary); padding: 8px 0; font-size: 12px; }
  .advanced { flex-basis: 100%; }
  .working { display: flex; align-items: center; gap: 8px; padding: 12px 18px; margin: 0; font-size: 12px; color: var(--accent); }
  textarea { caret-color: var(--accent); }
  textarea::selection { background: var(--accent); color: var(--bg-surface-1); }
  summary:focus-visible, input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .lines-wrap { padding-block: 12% 25%; }
  .lyric-line { display: flex; align-items: baseline; gap: 8px; margin-block: 6px; border-radius: 7px; }
  .line-button { width: 100%; justify-content: space-between; text-align: start; background: transparent; border: 0; font-size: 16px; font-weight: 600; line-height: 1.6; padding: 8px; }
  .line-button span { min-width: 0; overflow-wrap: anywhere; }
  .line-button small, .time { font: 10px var(--font-mono,monospace); font-variant-numeric: tabular-nums; color: var(--text-secondary); flex-shrink: 0; }
  .lyric-line.past { color: var(--text-secondary); }
  .lyric-line.past .line-button { color: inherit; }
  .lyric-line.active { background: var(--accent-soft); }
  .lyric-line.active .line-button { color: var(--accent); }
  .word-line { flex: 1; min-width: 0; font-size: 17px; line-height: 1.7; padding: 6px; }
  .word { display: inline; padding: 1px 0; background: transparent; border: 0; border-radius: 2px; font-size: inherit; font-weight: 700; color: var(--text-secondary); }
  .word.done { color: var(--accent); }
  .word.singing span { background: linear-gradient(90deg,var(--accent) var(--fill),var(--text-primary) var(--fill)); background-clip: text; -webkit-background-clip: text; color: transparent; }
  .time { margin-inline-end: 6px; background: transparent; border: 0; }
  .plain { color: var(--text-secondary); font-size: 14px; line-height: 1.7; margin: 6px 0; }
  @container stage (max-width: 500px) {
    .lyrics-panel.embedded { min-height: 280px; }
    header { padding: 14px; }
    .teleprompter { padding: 12px; }
    textarea { min-height: 180px; }
  }
</style>
