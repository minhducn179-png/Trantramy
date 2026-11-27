import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { transpileModule, ScriptTarget } from 'typescript';
import { connect } from 'node:net';

const port = 5196;
const session = `dlc-layout-${process.pid}`;
const server = spawn(process.execPath, ['node_modules/vite/bin/vite.js', ...(process.argv[2] === 'karaoke' || process.env.DLC_UI_PREVIEW === '1' ? ['preview'] : []), '--host', '127.0.0.1', '--port', String(port), '--strictPort'], { stdio: ['ignore', 'inherit', 'inherit'] });
const cli = (args, input) => {
  assert(args.every(a => /^[\w./:@=\\-]+$/.test(String(a))), 'Unexpected shell argument');
  const options = { input, encoding: 'utf8', timeout: 30000, maxBuffer: 2 ** 22 };
  const output = process.env.AGENT_BROWSER_BIN
    ? execFileSync(process.env.AGENT_BROWSER_BIN, ['--session', session, '--json', ...args], options)
    : execFileSync(process.execPath, ['node_modules/agent-browser/bin/agent-browser.js', '--session', session, '--json', ...args], options);
  const result = JSON.parse(output);
  assert(result.success, result.error);
  return result.data;
};
const evaluate = script => cli(['eval', '--stdin'], `(async () => { await new Promise(resolve => setTimeout(resolve, 0)); return (${script}); })()`).result;
let refs = {};
const snapshot = () => refs = cli(['snapshot', '-i']).refs;
const tap = (name, last = false) => {
  snapshot();
  const matches = Object.entries(refs).filter(([, r]) => r.role === 'button' && r.name?.trim() === name).sort((a, b) => Number(a[0].slice(1)) - Number(b[0].slice(1)));
  assert(matches.length, `Missing button: ${name}`);
  cli(['click', '@' + (last ? matches.at(-1) : matches[0])[0]]);
  snapshot();
};
const escape = () => { cli(['press', 'Escape']); snapshot(); };
const findings = [];
const check = label => {
  const issues = evaluate(`(() => {
    const issues = [];
    const stage = document.querySelector('.stage-content');
    if (stage.scrollWidth > stage.clientWidth + 2) issues.push('stage overflow');
    for (const el of document.querySelectorAll('.thead,.trow,.res-row,.cfg-card,.code-hub-card,.settings-stage,.hero-main,.inspector-grid,.deck,.eq-dialog,.modal-dialog,.karaoke-room,.room-body')) {
      if (el.scrollWidth > el.clientWidth + 2 && !['auto','scroll'].includes(getComputedStyle(el).overflowX)) issues.push(el.className + ' overflow');
    }
    for (const el of document.querySelectorAll('.stage-content button,.stage-content input,.deck button,.deck input,.lyrics-drawer button,.queue-drawer button')) {
      if (!el.getClientRects().length || el.closest('.yt-grid-row,.filter-chips,.settings-nav')) continue;
      const r = el.getBoundingClientRect();
      const area = el.closest('.stage-content')?.getBoundingClientRect();
      const left = area ? area.left : 0;
      const right = area ? area.right : innerWidth;
      if (r.left < left - 2 || r.right > right + 2) issues.push((el.getAttribute('aria-label') || el.title || el.textContent.trim()).slice(0,60) + ' clipped');
    }
    const deck = document.querySelector('.deck').getBoundingClientRect();
    if (Math.abs(deck.bottom - innerHeight) > 1) issues.push('player not anchored');
    if (document.querySelector('.stage').getBoundingClientRect().top < 32) issues.push('title strip overlaps content');
    return [...new Set(issues)];
  })()`);
  if (issues.length) findings.push({ label, issues });
};

try {
  let ready = false;
  for (let i = 0; i < 100 && !ready; i++) {
    if (server.exitCode !== null) throw new Error('Vite did not start');
    ready = await new Promise(resolve => {
      const socket = connect(port, '127.0.0.1');
      socket.once('connect', () => { socket.destroy(); resolve(true); });
      socket.once('error', () => resolve(false));
    });
    if (!ready) await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert(ready, 'Vite startup timeout');
  mkdirSync('.pi/ui-check', { recursive: true });
  mkdirSync('.impeccable/review', { recursive: true });
  const fixture = transpileModule(readFileSync('tests/ui-fixture.ts', 'utf8'), { compilerOptions: { target: ScriptTarget.ES2022 } }).outputText;
  writeFileSync('.pi/ui-check/ui-fixture.js', fixture);
  cli(['--init-script', '.pi/ui-check/ui-fixture.js', 'open', `http://127.0.0.1:${port}`]);
  cli(['wait', '.nav-item']);
  snapshot();
  const karaokeOnly = process.argv[2] === 'karaoke';
  if (!karaokeOnly) {
    tap('Thư viện'); tap('Phát Bài hát 3'); tap('Trang chủ');
  }
  const sizes = [[1440,900], [1100,720], [800,600], [520,400]].filter(([width]) => !process.argv[2] || width === Number(process.argv[2]));
  assert(sizes.length || karaokeOnly, 'Unknown viewport');
  for (const [width, height] of sizes) {
    cli(['set', 'viewport', String(width), String(height)]); snapshot();
    tap('Trang chủ'); check(`${width}: home`);
    if (width === 1440 || width === 520) cli(['screenshot', 'body', `.pi/ui-check/home-${width}.png`]);
    tap('Thư viện'); check(`${width}: library`);
    const columns = evaluate(`(() => {
      const h = document.querySelector('.thead').children;
      const r = document.querySelector('.trow').children;
      return [0,1,3].map(i => Math.abs(h[i].getBoundingClientRect().left - r[i].getBoundingClientRect().left));
    })()`);
    assert(columns.every(n => n < 2), `${width}: table columns align`);
    if (width === 520) {
      assert(evaluate(`(() => {
        const row = document.querySelectorAll('.trow')[1];
        const actions = row.querySelector('.act-cell');
        const bounds = actions.getBoundingClientRect();
        const title = row.querySelector('.title-cell').getBoundingClientRect();
        const buttons = [...actions.querySelectorAll('button')].map(button => button.getBoundingClientRect());
        return getComputedStyle(actions).opacity === '1'
          && Math.abs(bounds.left - title.left) < 2
          && Math.abs(buttons[0].left - bounds.left) < 2
          && buttons.every(button => button.width > 0 && button.left >= bounds.left - 1 && button.right <= bounds.right + 1);
      })()`), 'Narrow row actions stay visible and aligned');
      cli(['screenshot', 'body', '.pi/ui-check/library-520.png']);
    }
    const before = evaluate(`window.__uiCalls.filter(c => c.command === 'toggle_like').length`);
    const like = Object.entries(refs).find(([,r]) => r.role === 'button' && r.name === 'Thích');
    assert(like, 'Library like action exists');
    cli(['click', '@' + like[0]]); snapshot();
    assert.equal(evaluate(`window.__uiCalls.filter(c => c.command === 'toggle_like').length`), before + 1);
    tap('Cài đặt');
    for (const tab of ['Tổng quan','Màu sắc','Hình nền','Trình phát & Sóng nhạc','Dynamic Island','Âm thanh','Lời bài hát & AI','Mã kiểu & Sao lưu']) {
      tap(tab, true); check(`${width}: settings ${tab}`);
      if (width === 1440 && tab === 'Tổng quan') cli(['screenshot', 'body', '.pi/ui-check/settings-1440.png']);
      if (tab === 'Dynamic Island') {
        tap('Trang lời bài hát', true);
        assert(evaluate(`document.querySelector('.karaoke-room').matches(':modal')`), 'Settings shortcut opens karaoke');
        escape();
      }
    }
    tap('Khám phá'); check(`${width}: discovery`);
    const search = Object.entries(refs).find(([,r]) => r.role === 'textbox' && r.name === 'Tìm nhạc, dán link…');
    cli(['fill', '@' + search[0], 'fixture']); cli(['press', 'Enter']); snapshot();
    check(`${width}: search results`);
    tap('Thư viện');
    const library = cli(['snapshot', '-i', '-s', '.tbody']);
    const title = Object.entries(library.refs).find(([ref,r]) => library.snapshot.includes(`[ref=${ref}]`) && r.role === 'button' && r.name.startsWith('Bản nhạc tiếng Việt'));
    assert(title, 'Track title opens details even when this track is already playing');
    cli(['click', '@' + title[0]]); snapshot();
    check(`${width}: detail`);
    tap('Đến lời bài hát trên trang thông tin');
    assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer').length`), 0, 'Detail lyrics action never opens a duplicate drawer');
    assert.equal(evaluate(`document.querySelectorAll('.lyrics-panel').length`), 1, 'Detail contains only the embedded lyrics');
    assert(evaluate(`document.activeElement.id === 'track-detail-lyrics'`), 'Detail lyrics receives keyboard focus');
    assert(await evaluate(`(async () => {
      await new Promise(resolve => setTimeout(resolve, 600));
      const lyrics = document.getElementById('track-detail-lyrics');
      const target = lyrics.querySelector('.lyric-line.active, .plain, textarea, .empty') ?? lyrics.querySelector('.lyric-line');
      const bounds = target.getBoundingClientRect();
      const stage = document.querySelector('.stage-content').getBoundingClientRect();
      return bounds.top >= stage.top - 1 && bounds.bottom <= stage.bottom + 1;
    })()`), 'Actual lyric text, not only panel controls, scrolls into the visible stage');
    tap('Đến lời bài hát trên trang thông tin');
    assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer').length`), 0, 'Repeated detail action stays inline');
    check(`${width}: inline lyrics`);
    if (width === 1440 || width === 520) cli(['screenshot', 'body', `.impeccable/review/detail-lyrics-${width}.png`]);
    tap('Hàng chờ phát');
    tap('Đến lời bài hát trên trang thông tin');
    assert.equal(evaluate(`document.querySelectorAll('.queue-drawer').length`), 0, 'Jump closes queue so it cannot cover lyrics');
    for (const page of ['Trang chủ', 'Thư viện', 'Khám phá', 'Cài đặt', 'Danh sách phát có tên dài để kiểm tra 12']) {
      tap(page); tap('Lời bài hát & Studio Karaoke');
      assert(evaluate(`document.querySelector('.karaoke-room').matches(':modal')`), page + ': karaoke opens');
      assert.equal(evaluate(`document.querySelectorAll('.lyrics-panel').length`), 0, page + ': no duplicate editor');
      escape();
      assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, page + ': karaoke closes');
    }
    tap('Thư viện'); tap('Lời bài hát & Studio Karaoke');
    tap('Sửa lời');
    assert.equal(evaluate(`document.querySelectorAll('.room-lyrics .lyrics-panel').length`), 1, 'Karaoke opens the shared editor');
    escape();
    const deck = cli(['snapshot', '-i', '-s', '.deck']);
    const identity = Object.entries(deck.refs).find(([ref, r]) => deck.snapshot.includes(`[ref=${ref}]`) && r.role === 'button' && r.name.includes('Nghệ sĩ'));
    assert(identity, 'Current track opens inspector from player');
    cli(['click', '@' + identity[0]]); snapshot();
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Entering inspector does not revive karaoke');
    tap('Trang chủ');
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Leaving inspector does not revive karaoke');
    tap('Lời bài hát & Studio Karaoke'); tap('Sửa lời'); check(`${width}: lyrics editor`);
    const editors = evaluate(`(() => {
      const b = document.querySelector('.room-lyrics [aria-label="Chỉnh sửa lời bài hát"]');
      const r = b.getBoundingClientRect();
      return b.contains(document.elementFromPoint(r.x + r.width/2, r.y + r.height/2));
    })()`);
    assert(editors, 'Lyrics editor is not covered by window controls');
    escape();
    tap('Hàng chờ phát');
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Queue does not reopen karaoke');
    check(`${width}: queue`); escape();
    tap('Bộ cân bằng âm thanh (Equalizer)'); check(`${width}: equalizer`);
    assert(evaluate(`(() => {
      const e = document.querySelector('.eq-dialog'); e.scrollTop = e.scrollHeight;
      return e.getBoundingClientRect().height <= innerHeight - 30 && e.querySelector('.eq-footer').getBoundingClientRect().bottom <= innerHeight - 14;
    })()`), 'Equalizer footer remains reachable');
    const preamp = Object.entries(refs).find(([,r]) => r.role === 'slider' && r.name === 'Pre-Amp');
    cli(['click', '@' + preamp[0]]); snapshot();
    escape();
    assert.equal(evaluate(`document.querySelectorAll('.eq-dialog').length`), 0, 'Escape closes EQ from inside its controls');
    assert(!Object.values(refs).some(r => r.name.includes('Visualizer')), 'Old Visualizer entry is removed');
    tap('Lời bài hát & Studio Karaoke');
    assert(evaluate(`document.querySelector('.karaoke-room').matches(':modal')`), 'Player utility opens karaoke instead of visualizer');
    check(`${width}: karaoke`); escape();
    tap('Nhập danh sách phát từ Spotify'); check(`${width}: spotify`);
    const url = Object.entries(refs).find(([,r]) => r.role === 'textbox' && r.name === 'Liên kết Spotify');
    cli(['fill', '@' + url[0], 'https://open.spotify.com/playlist/fixture']);
    tap('Trích xuất'); check(`${width}: spotify preview`);
    assert(evaluate(`(() => {
      const e = document.querySelector('.modal-dialog'); e.scrollTop = e.scrollHeight;
      return e.querySelector('.modal-footer').getBoundingClientRect().bottom <= innerHeight - 14;
    })()`), 'Spotify save actions remain reachable');
    escape();
    assert.equal(evaluate(`document.querySelectorAll('.modal-dialog').length`), 0, 'Escape closes Spotify from its URL field');
    const defects = findings.filter(f => f.label.startsWith(width + ':'));
    console.log(`${defects.length ? 'FAIL' : 'PASS'}: ${width}x${height}, navigation, all settings, library/search/detail, drawers, dialogs and like callback`);
    if (defects.length) console.log(JSON.stringify(defects));
  }
  if (karaokeOnly) {
    mkdirSync('.impeccable/review', { recursive: true });
    cli(['set', 'viewport', '1440', '900']);
    evaluate(`(() => {
      const native = window.__TAURI_INTERNALS__, original = native.invoke.bind(native);
      window.__karaokeLyrics = '[00:02.00]<00:02.00>Xin <00:02.60>chào <00:03.20>Việt <00:03.80>Nam\\n[00:08.00]Cùng hát lên giai điệu của chúng ta';
      native.invoke = async (command, args) => {
        if (command === 'lyrics_read') {
          window.__uiCalls.push({ command, args });
          if (window.__karaokeLyrics === null) return null;
          return { content: window.__karaokeLyrics, source: 'manual', timing: 'word', review: false, draft_only: Boolean(window.__karaokeDraft) };
        }
        if (command === 'lyrics_generate') {
          window.__uiCalls.push({ command, args });
          return { content: '[00:02.00]Lời karaoke mới\\n[00:08.00]Câu tiếp theo', source: 'lrclib', timing: 'line', review: false };
        }
        if (command === 'save_lyrics') {
          window.__uiCalls.push({ command, args });
          window.__karaokeLyrics = args.content;
          window.__uiEmit('lyrics-updated', args.path);
          return null;
        }
        if (command === 'resolve_stream') return original(command, args);
        if (command === 'search_online') {
          window.__uiCalls.push({ command, args });
          if (args.query.includes('không lời')) return [{ title: 'Beat kiểm thử', artist: 'Nguồn thử nghiệm', url: 'https://example.invalid/beat', duration: 60, source: 'youtube', thumbnail: null, album: '' }];
          return [{ title: 'Có giọng kiểm thử', artist: 'Nguồn thử nghiệm', url: 'https://example.invalid/vocal', duration: 60, source: 'youtube', thumbnail: null, album: '' }];
        }
        return original(command, args);
      };
    })()`);
    const setLayout = label => {
      tap('Cài đặt lời bài hát');
      const tabs = cli(['snapshot', '-i']);
      const ui = Object.entries(tabs.refs).find(([, r]) => r.role === 'tab' && r.name.includes('Giao diện'));
      assert(ui, 'Lyrics appearance tab exists');
      cli(['click', '@' + ui[0]]); snapshot();
      tap(label);
      tap('Thư viện'); tap('Lời bài hát & Studio Karaoke');
    };
    snapshot(); tap('Lời bài hát & Studio Karaoke');
    assert(evaluate(`document.querySelector('.karaoke-room').matches(':modal')`), 'Room is modal and traps focus');
    assert.equal(evaluate(`document.querySelectorAll('.songbook').length`), 0, 'Room starts with a full-width stage, not a permanently open desk');
    tap('Gọi bài · 0'); tap('Xem thư viện');
    tap('Gọi bài Bản nhạc tiếng Việt với tên thật dài để kiểm tra bố cục và dấu câu');
    tap('Gọi bài Bài hát 2'); tap('Gọi bài Bài hát 3'); tap('Gọi bài Bài hát 4');
    tap('Chờ hát 3'); tap('Ưu tiên Bài hát 4');
    assert(evaluate(`document.querySelector('.book-list .song-row strong').textContent === 'Bài hát 4'`), 'Priority moves song to next turn');
    tap('Bỏ bài Bài hát 2');
    assert(evaluate(`document.querySelectorAll('.book-list .song-row').length === 2`), 'Remove drops only selected turn');
    tap('Bỏ qua');
    assert(evaluate(`document.querySelector('.vh-title').textContent === 'Bài hát 4'`), 'Skip follows priority');
    tap('Tìm bản beat');
    tap('Dùng bản Beat kiểm thử');
    assert(evaluate(`document.querySelector('.vh-title').textContent === 'Beat kiểm thử'`), 'Explicit source changes current version');
    tap('Bản có giọng'); tap('Dùng bản Có giọng kiểm thử');
    assert(evaluate(`document.querySelector('.vh-title').textContent === 'Có giọng kiểm thử'`), 'Original vocal source can be selected');
    tap('Tạm dừng');
    evaluate(`(() => { const seek = document.querySelector('[aria-label="Vị trí bài hát"]'); seek.value = '3'; seek.dispatchEvent(new Event('input', { bubbles: true })); })()`);
    assert(evaluate(`document.querySelector('.line-current').textContent.includes('Xin chào Việt Nam')`), 'Seek updates active lyric');
    const fills = evaluate(`Array.from(document.querySelectorAll('.sung-word'), word => parseFloat(word.style.getPropertyValue('--fill')))`);
    assert(fills[0] === 100 && fills[1] > 0 && fills[1] < 100 && fills[2] === 0, 'Word fills follow playback clock');
    setLayout('Cuộn toàn bộ lời');
    assert.equal(evaluate(`document.querySelectorAll('.full-lyric-line').length`), 2, 'Full lyric view contains every line');
    assert(evaluate(`Boolean(document.querySelector('.stage-controls .play-button:not(:disabled)'))`), 'Full lyric view keeps playback controls available');
    tap('Xin chào Việt Nam');
    assert(evaluate(`document.querySelector('.full-lyric-line.active').textContent.includes('Xin chào')`), 'Full view highlights current line');
    tap('Cùng hát lên giai điệu của chúng ta');
    assert(evaluate(`document.querySelector('.full-lyric-line.active').textContent.includes('Cùng hát')`), 'Full lyric lines seek playback');
    setLayout('Sân khấu karaoke');
    const sourcePath = evaluate(`window.__uiCalls.filter(c => c.command === 'lyrics_read').at(-1).args.track.path`);
    tap('Sửa lời'); tap('Tìm lời');
    assert(evaluate(`document.querySelector('.room-lyrics textarea').value.includes('Lời karaoke mới')`), 'Karaoke reuses the normal lyric lookup and editor');
    assert(!evaluate(`document.querySelector('.lyric-stage').textContent.includes('Lời karaoke mới')`), 'Lookup is a preview, not a silent replacement');
    tap('Lưu lời');
    assert.equal(evaluate(`window.__uiCalls.filter(c => c.command === 'save_lyrics').at(-1).args.path`), sourcePath, 'Lyrics are saved against the selected karaoke recording');
    tap('Về sân khấu');
    assert(evaluate(`document.querySelector('.lyric-stage').textContent.includes('Lời karaoke mới')`), 'Saved normal lyrics update the karaoke stage through the shared event');
    const beforeHide = evaluate(`document.querySelector('.song-meta h2').textContent`);
    tap('Ẩn điều khiển');
    assert(evaluate(`document.querySelectorAll('.room-header,.song-heading,.stage-controls,.songbook').length === 0`), 'Focus mode removes every auxiliary surface');
    assert(evaluate(`document.activeElement.getAttribute('aria-label') === 'Hiện điều khiển'`), 'Focus moves to a visible restore control');
    assert(evaluate(`document.querySelector('.singing-stage').clientWidth === document.querySelector('.room-body').clientWidth`), 'Focused lyrics use the available room width');
    assert(evaluate(`document.querySelector('.stage-backdrop').getAttribute('src').length > 0`), 'Stage retains its background behind lyrics');
    cli(['screenshot', 'body', '.impeccable/review/karaoke-focused-desktop.png']);
    tap('Hiện điều khiển');
    assert.equal(evaluate(`document.querySelector('.song-meta h2').textContent`), beforeHide, 'Hiding tools does not change the recording');
    assert(evaluate(`document.activeElement.getAttribute('aria-label') === 'Ẩn điều khiển'`), 'Restoring tools preserves keyboard navigation');
    tap('Thu nhỏ về giao diện bên phải');
    assert(evaluate(`document.querySelector('.karaoke-room').classList.contains('embedded')`), 'Shrink moves the room into the main content');
    tap('Phóng to toàn bộ cửa sổ');
    assert(evaluate(`document.querySelector('.karaoke-room').matches(':modal')`), 'Expand restores the modal room');
    tap('Gọi bài · 1'); tap('Chờ hát 1');
    const capture = (width, height, file) => {
      cli(['set', 'viewport', String(width), String(height)]); snapshot();
      const defects = evaluate(`(() => {
        const errors = [], room = document.querySelector('.karaoke-room');
        for (const el of room.querySelectorAll('header,main,aside,footer,.room-body,.transport-row,.book-tabs')) {
          if (el.getClientRects().length && el.scrollWidth > el.clientWidth + 2) errors.push(el.className + ' overflow');
        }
        for (const el of room.querySelectorAll('button,input')) {
          if (!el.getClientRects().length) continue;
          const r = el.getBoundingClientRect();
          if (r.left < -1 || r.right > innerWidth + 1 || r.top < -1 || r.bottom > innerHeight + 1) errors.push((el.getAttribute('aria-label') || el.textContent) + ' clipped');
        }
        return errors;
      })()`);
      assert.deepEqual(defects, [], width + ': room fits viewport');
      cli(['screenshot', 'body', file]);
    };
    capture(1440, 900, '.impeccable/review/desktop.png');
    tap('Về sân khấu');
    capture(390, 844, '.impeccable/review/mobile.png');
    tap('Ẩn điều khiển');
    capture(390, 844, '.impeccable/review/karaoke-focused-mobile.png');
    tap('Hiện điều khiển');
    tap('Gọi bài · 1');
    capture(390, 844, '.impeccable/review/mobile-songbook.png');
    tap('Về sân khấu');
    capture(520, 400, '.impeccable/review/karaoke-compact.png');
    const beforeClose = evaluate(`document.querySelector('.deck .title').textContent`);
    cli(['press', 'Escape']); snapshot();
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Escape closes room from inside controls');
    assert.equal(evaluate(`document.querySelector('.deck .title').textContent`), beforeClose, 'Closing preserves current playback');
    assert(evaluate(`document.activeElement.getAttribute('aria-label') === 'Lời bài hát & Studio Karaoke'`), 'Closing restores focus to entry button');
    for (const content of ['Lời chưa có mốc\nCâu thứ hai', '', null]) {
      evaluate(`window.__karaokeLyrics = ${JSON.stringify(content)}`);
      tap('Lời bài hát & Studio Karaoke');
      if (content) assert.equal(evaluate(`document.querySelector('.plain-lyrics').value`), content.replaceAll('\n', '\n\n'), 'Plain lyrics remain readable without invented timing');
      else assert(evaluate(`document.querySelector('.lyric-stage').textContent.includes('Bản này chưa có lời')`), 'Missing lyrics do not masquerade as synced words');
      tap('Đóng phòng lời bài hát');
    }
    evaluate(`(window.__karaokeLyrics = '[00:02.00]Xin ⟦không nghe rõ⟧.', window.__karaokeDraft = true)`);
    tap('Lời bài hát & Studio Karaoke');
    assert(evaluate(`document.querySelector('.plain-lyrics').value.includes('⟦không nghe rõ⟧')`), 'Karaoke never presents unresolved audio as timed lyrics');
    assert(evaluate(`document.querySelector('.lyric-stage').textContent.includes('Bản nháp có chỗ chưa nghe rõ')`), 'Karaoke identifies incomplete drafts');
    assert.equal(evaluate(`document.querySelectorAll('.sung-word').length`), 0, 'Draft timestamps never produce confirmed word highlighting');
    assert(evaluate(`Boolean(document.querySelector('.stage-controls .play-button:not(:disabled)'))`), 'Draft lyrics keep playback controls available');
    tap('Ẩn điều khiển');
    assert(evaluate(`document.querySelector('.lyric-stage').textContent.includes('chưa căn mốc')`), 'Focused karaoke preserves the uncertainty warning');
    escape();
    evaluate(`(window.__karaokeLyrics = null, window.__karaokeDraft = false)`);
    tap('Lời bài hát & Studio Karaoke'); tap('Sửa lời');
    assert(evaluate(`document.querySelector('.room-lyrics').textContent.includes('Tạo lời tự động')`), 'Missing karaoke lyrics expose the same generation flow as normal tracks');
    assert(evaluate(`document.querySelector('.room-lyrics .teleprompter').clientHeight >= 40`), 'Compact lyric tools keep a scrollable work area');
    cli(['screenshot', 'body', '.impeccable/review/karaoke-lyrics-tools.png']);
    tap('Cài đặt lời bài hát');
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'AI settings leave the room without stopping playback');
    assert(evaluate(`Boolean(document.querySelector('[aria-label="Cài đặt lời bài hát"]'))`), 'Shared lyric settings are reachable');
    evaluate(`(() => {
      const config = JSON.parse(localStorage.getItem('dlc.ui.config.v2') || '{}');
      Object.assign(config, { paletteId: 'catppuccin-latte', presetId: 'catppuccin-latte', colorScheme: 'light', bgImage: 'data:image/svg+xml,' + encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="white"/></svg>') });
      localStorage.setItem('dlc.ui.config.v2', JSON.stringify(config));
    })()`);
    cli(['open', `http://127.0.0.1:${port}`]); cli(['wait', '.nav-item']); snapshot();
    tap('Lời bài hát & Studio Karaoke');
    assert.equal(evaluate(`document.documentElement.dataset.scheme`), 'light', 'Karaoke respects the selected light theme');
    assert(evaluate(`document.querySelector('.stage-backdrop.custom-wallpaper').src === JSON.parse(localStorage.getItem('dlc.ui.config.v2')).bgImage`), 'Karaoke uses the configured wallpaper rather than overriding it with cover art');
    tap('Ẩn điều khiển');
    assert(evaluate(`document.querySelector('.stage-backdrop.custom-wallpaper').clientHeight === document.querySelector('.karaoke-room').clientHeight`), 'Wallpaper still fills focused stage');
    escape();
    assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Escape remains available with all tools hidden');
    console.log('PASS: karaoke search, FIFO/priority/remove/skip, beat/vocal sources, plain/missing/generated/saved lyrics, full-stage focus, wallpaper/light theme, desktop/mobile/compact layout');
  }
  assert.deepEqual(findings, [], 'Layout defects');
  const errors = cli(['errors']).errors;
  assert(!errors?.length, JSON.stringify(errors));
} catch (error) {
  console.error(JSON.stringify(cli(['snapshot', '-i'])));
  console.error(JSON.stringify(cli(['errors'])));
  cli(['screenshot', 'body', '.pi/ui-check/failure.png']);
  throw error;
} finally {
  try { cli(['close']); } finally {
    server.kill();
    rmSync('.pi/ui-check/ui-fixture.js', { force: true });
  }
}
