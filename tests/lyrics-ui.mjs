import assert from 'node:assert/strict';
import { execFileSync, execSync, spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { transpileModule, ScriptTarget } from 'typescript';
import { connect } from 'node:net';

const port = 5197;
const session = `dlc-lyrics-${process.pid}`;
const script = `.pi/ui-check/lyrics-fixture-${process.pid}.js`;
const server = spawn(process.execPath, ['node_modules/vite/bin/vite.js', ...(process.env.DLC_UI_PREVIEW === '1' ? ['preview'] : []), '--host', '127.0.0.1', '--port', String(port), '--strictPort'], { stdio: 'ignore' });
const cli = (args, input) => {
  assert(args.every(a => /^[\w./:@=\\ [\]-]+$/.test(String(a))), 'Unexpected shell argument');
  const options = { input, encoding: 'utf8', timeout: 30000, maxBuffer: 2 ** 22 };
  const output = process.env.AGENT_BROWSER_BIN
    ? execFileSync(process.env.AGENT_BROWSER_BIN, ['--session', session, '--json', ...args], options)
    : execSync(`npx --no-install agent-browser --session ${session} --json ${args.map(a => `"${a}"`).join(' ')}`, options);
  const result = JSON.parse(output);
  assert(result.success, result.error);
  return result.data;
};
const evaluate = code => cli(['eval', '--stdin'], `(async () => { await new Promise(requestAnimationFrame); return (${code}); })()`).result;
let refs;
const snapshot = () => refs = cli(['snapshot', '-i']).refs;
const tap = (name, last = false) => {
  snapshot();
  const matches = Object.entries(refs).filter(([, r]) => r.role === 'button' && (r.name === name || r.name.startsWith(name + ' '))).sort((a, b) => Number(a[0].slice(1)) - Number(b[0].slice(1)));
  assert(matches.length, `Missing button: ${name}`);
  cli(['click', '@' + (last ? matches.at(-1) : matches[0])[0]]); snapshot();
};
try {
  let ready = false;
  for (let i = 0; i < 100 && !ready; i++) {
    assert.equal(server.exitCode, null, 'Vite stays running');
    ready = await new Promise(resolve => {
      const socket = connect(port, '127.0.0.1');
      socket.once('connect', () => { socket.destroy(); resolve(true); });
      socket.once('error', () => resolve(false));
    });
    if (!ready) await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert(ready, 'Vite is ready');
  mkdirSync('.pi/ui-check', { recursive: true });
  const fixture = transpileModule(readFileSync('tests/ui-fixture.ts', 'utf8'), { compilerOptions: { target: ScriptTarget.ES2022 } }).outputText;
  writeFileSync(script, fixture + `
    (() => {
      HTMLMediaElement.prototype.play = async function() {};
      const original = window.__TAURI_INTERNALS__.invoke;
      window.__lyricTest = { fail: false, empty: false, delay: false, calls: [], saved: '', stored: {}, deleted: {}, rewriteFail: false, deleteFail: false };
      window.__TAURI_INTERNALS__.invoke = async (command, args) => {
        const test = window.__lyricTest;
        if (command === 'lyrics_read') {
          if (test.empty || test.deleted[args.track.path]) return null;
          if (test.stored[args.track.path]) return { content: test.stored[args.track.path], source: 'manual', timing: 'plain', review: false };
        }
        if (command === 'lyrics_delete') {
          test.calls.push(command);
          if (test.deleteFail) throw 'Đang có tác vụ lời khác. Chờ xong rồi xóa.';
          test.deleted[args.track.path] = true;
          delete test.stored[args.track.path];
          return null;
        }
        if (command === 'lyrics_generate' && (args.rewrite || test.deleted[args.track.path])) {
          test.pipelinePath = args.track.path;
          test.calls.push(command);
          if (test.rewriteFail) throw 'Không nhận diện được âm thanh.';
          if (test.pipelineDelay) await new Promise(resolve => test.pipelineRelease = resolve);
          if (test.unresolved) return { content: 'Xin ⟦không nghe rõ⟧.', source: 'gemini', timing: 'plain', review: true, draft_only: true, notice: 'Bản nháp chưa hoàn tất. Khoảng cần nghe lại: 00:01.40–00:02.00. Chưa căn mốc từng chữ.' };
          return { content: 'Bản viết lại\\nNội dung mới.', source: 'gemini', timing: 'plain', review: true };
        }
        if (command === 'lyrics_review_text') {
          test.calls.push(command);
          if (test.delay) await new Promise(resolve => test.release = resolve);
          if (test.fail) throw 'Chưa cho phép gửi lời đến Gemini. Mở Thiết lập AI.';
          return { content: args.content.replace('Nam', 'Nam!'), changed_lines: 1 };
        }
        if (command === 'lyrics_transcribe') {
          test.calls.push(command);
          return { content: '[00:00.00]<00:00.00>Xin<00:00.60> <00:00.60>chào<00:01.20> <00:01.20>Việt<00:01.80> <00:01.80>Nam<00:02.20>\\n[00:04.00]<00:04.00>Lời<00:04.20> <00:04.20>bài<00:04.40> <00:04.40>hát<00:04.60> <00:04.60>được<00:04.80> <00:04.80>giữ<00:05.00> <00:05.00>nguyên<00:05.20>', source: 'whisper.cpp', timing: 'word', review: true };
        }
        if (command === 'save_lyrics') {
          test.saved = args.content;
          test.stored[args.path] = args.content;
          test.deleted[args.path] = false;
        }
        return original(command, args);
      };
    })();
  `);
  cli(['--init-script', script, 'open', `http://127.0.0.1:${port}`]);
  cli(['wait', '.nav-item']); snapshot();
  tap('Thư viện');
  const track = Object.entries(refs).find(([, r]) => r.role === 'button' && r.name.startsWith('Phát Bản nhạc tiếng Việt'));
  assert(track); cli(['click', '@' + track[0]]); snapshot();
  tap('Thư viện'); // Outside the inspector, the player button still opens the lyrics drawer.
  tap('Lời bài hát đồng bộ');
  tap('Chỉnh sửa lời bài hát', true);
  tap('AI rà soát', true);
  cli(['wait', '.lyrics-drawer .review-preview']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer .review-preview').textContent.includes('Nam!')`));
  assert.equal(evaluate(`window.__lyricTest.calls.filter(c => c === 'lyrics_transcribe').length`), 0, 'Text-only path never transcribes');
  assert(evaluate(`Boolean([...document.querySelectorAll('.lyrics-drawer button')].find(b => b.textContent?.trim().includes('Lưu lời'))?.disabled)`), 'Preview must be applied before save');
  for (const [width, height, name] of [[1440, 900, 'desktop'], [520, 650, 'narrow']]) {
    cli(['set', 'viewport', String(width), String(height)]); snapshot();
    assert(evaluate(`(() => { const p = document.querySelector('.lyrics-drawer .lyrics-panel'); return p.scrollWidth <= p.clientWidth + 1 && [...p.querySelectorAll('button,textarea')].every(e => e.getBoundingClientRect().right <= innerWidth + 1); })()`), `${name}: controls fit`);
    cli(['screenshot', 'body', `.pi/ui-check/lyrics-review-${name}.png`]);
  }
  tap('Áp dụng bản nháp', true);
  tap('Lưu lời', true);
  assert(evaluate(`window.__lyricTest.saved.includes('<00:01.80>Nam!')`), 'Text correction retains original word timestamp');
  tap('Chỉnh sửa lời bài hát', true);
  const align = Object.entries(refs).find(([, r]) => r.role === 'checkbox' && r.name === 'Căn lại theo giọng hát');
  cli(['check', '@' + align[0]]); snapshot();
  tap('AI rà soát', true);
  cli(['wait', '.lyrics-drawer .review-preview']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer .review-preview').textContent.includes('Đã đối chiếu mốc')`), 'Audio alignment returns a real preview');
  tap('Bỏ kết quả', true);
  evaluate(`(() => { const input = document.querySelector('.lyrics-drawer textarea'); input.value = 'Xin chào\\nViệt Nam\\nLời bài hát được giữ nguyên'; input.dispatchEvent(new Event('input', { bubbles: true })); return input.value; })()`);
  tap('AI rà soát', true);
  cli(['wait', '.lyrics-drawer .review-preview']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer .review-preview').textContent.includes('Đã đối chiếu mốc')`), 'Audio re-alignment accepts changed line breaks without old-timing constraints');
  tap('Bỏ kết quả', true);
  evaluate(`window.__lyricTest.fail = true`);
  tap('AI rà soát', true);
  cli(['wait', '.lyrics-drawer [role=alert]']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer [role=alert]').textContent.includes('Chưa cho phép')`), 'Consent failure is visible');
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer .review-preview').length`), 0);
  evaluate(`Object.assign(window.__lyricTest, { fail: false, delay: true })`);
  tap('AI rà soát', true);
  assert(evaluate(`document.querySelector('.lyrics-drawer textarea').disabled`), 'Editing is locked while review runs');
  tap('Đóng lời bài hát');
  evaluate(`window.__lyricTest.release()`);
  tap('Lời bài hát đồng bộ');
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer .review-preview').length`), 0, 'Late review does not restore a closed preview');
  const savedBeforeRewrite = evaluate(`window.__lyricTest.saved`);
  tap('Viết lại bằng AI', true);
  tap('Hủy', true);
  assert.equal(evaluate(`window.__lyricTest.calls.filter(c => c === 'lyrics_generate').length`), 0, 'Cancel does not send audio');
  evaluate(`window.__lyricTest.rewriteFail = true`);
  tap('Viết lại bằng AI', true);
  tap('Bắt đầu viết lại', true);
  cli(['wait', '.lyrics-drawer [role=alert]']); snapshot();
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforeRewrite, 'Failed rewrite keeps saved lyrics');
  assert.equal(evaluate(`document.querySelectorAll('.replacement-preview').length`), 0);
  evaluate(`window.__lyricTest.rewriteFail = false`);
  tap('Viết lại bằng AI', true);
  tap('Bắt đầu viết lại', true);
  cli(['wait', '.lyrics-drawer .replacement-preview']); snapshot();
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforeRewrite, 'Preview does not overwrite saved lyrics');
  cli(['screenshot', 'body', '.pi/ui-check/lyrics-rewrite-narrow.png']);
  tap('Giữ bản cũ', true);
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforeRewrite);
  tap('Viết lại bằng AI', true);
  tap('Bắt đầu viết lại', true);
  cli(['wait', '.lyrics-drawer .replacement-preview']); snapshot();
  tap('Dùng bản viết lại', true);
  tap('Lưu lời', true);
  assert.equal(evaluate(`window.__lyricTest.saved`), 'Bản viết lại\nNội dung mới.', 'Rewritten plain lyrics do not inherit the old timestamp constraints');
  tap('Xóa lời đã lưu', true);
  tap('Hủy', true);
  assert.equal(evaluate(`window.__lyricTest.calls.filter(c => c === 'lyrics_delete').length`), 0, 'Cancel does not delete');
  evaluate(`window.__lyricTest.deleteFail = true`);
  tap('Xóa lời đã lưu', true);
  tap('Xác nhận xóa lời', true);
  cli(['wait', '.lyrics-drawer [role=alert]']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer [role=alert]').textContent.includes('Đang có tác vụ')`));
  assert.equal(evaluate(`window.__lyricTest.saved`), 'Bản viết lại\nNội dung mới.');
  evaluate(`window.__lyricTest.deleteFail = false`);
  tap('Xóa lời đã lưu', true);
  cli(['screenshot', 'body', '.pi/ui-check/lyrics-delete-narrow.png']);
  tap('Xác nhận xóa lời', true);
  cli(['wait', '.lyrics-drawer .empty']); snapshot();
  assert(evaluate(`[...document.querySelectorAll('.lyrics-panel')].every(p => p.textContent.includes('Tôi đã có lời') && !p.querySelector('textarea'))`), 'Delete clears saved lyrics and drafts in the drawer');
  const deck = cli(['snapshot', '-i', '-s', '.deck']);
  const identity = Object.entries(deck.refs).find(([ref, r]) => deck.snapshot.includes(`[ref=${ref}]`) && r.role === 'button' && r.name.includes('Nghệ sĩ'));
  assert(identity); cli(['click', '@' + identity[0]]); snapshot();
  tap('Đến lời bài hát trên trang thông tin');
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer').length`), 0, 'Inspector uses inline lyrics, not a second panel');
  cli(['wait', '.detail-lyrics .empty']); snapshot();
  assert(evaluate(`document.querySelector('#track-detail-lyrics').textContent.includes('Tôi đã có lời') && !document.querySelector('#track-detail-lyrics textarea')`), 'Delete also clears lyrics and drafts when opening the inline panel');
  tap('Thư viện');
  tap('Lời bài hát đồng bộ');
  cli(['wait', '.lyrics-drawer .empty']); snapshot();
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer .replacement-preview').length`), 0, 'Deleted lyrics stay empty when reopened');
  tap('Tôi đã có lời', true);
  const fresh = Object.entries(refs).find(([, r]) => r.role === 'textbox' && r.name.includes('Chỉnh sửa lời'));
  cli(['fill', '@' + fresh[0], 'Moi']); snapshot();
  tap('Lưu lời', true);
  assert.equal(evaluate(`window.__lyricTest.saved`), 'Moi', 'Manual rewriting works after deletion');
  tap('Thiết lập AI', true);
  cli(['wait', '.lyrics-settings']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-settings').textContent.includes('Model AI viết lời')`), 'Settings shortcut opens correct tab');
  assert(evaluate(`!document.querySelector('.lyrics-settings input[type=checkbox]').disabled`));
  tap('Lưu cài đặt lời bài hát');
  const sent = evaluate(`JSON.parse(JSON.stringify(window.__uiCalls.filter(c => c.command === 'lyrics_save_settings').at(-1).args.config))`);
  assert(sent.audio_pipeline, 'Multi-pass pipeline is always enabled: ' + JSON.stringify(sent));
  assert(sent.context_rewrite, 'Extra rewrite is always enabled: ' + JSON.stringify(sent));
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-settings details .pipeline-steps li').length`), 16, 'Settings expose the same standards embedded into AI prompts');
  for (const [width, height, name] of [[1440, 900, 'desktop'], [520, 650, 'narrow']]) {
    cli(['set', 'viewport', String(width), String(height)]); snapshot();
    const banner = Object.entries(refs).find(([, r]) => r.name && r.name.includes('Tiêu chuẩn'));
    if (banner) cli(['scrollintoview', '@' + banner[0]]); snapshot();
    assert(evaluate(`Boolean(document.querySelector('.standards-banner'))`), 'Standards banner is visible');
    cli(['screenshot', 'body', `.pi/ui-check/lyrics-pipeline-settings-${name}.png`]);
  }
  tap('Thư viện');
  cli(['wait', '.trow']); snapshot();
  evaluate(`window.__lyricTest.empty = true`);
  const next = Object.entries(refs).find(([, r]) => r.role === 'button' && r.name === 'Phát Bài hát 2');
  assert(next); cli(['click', '@' + next[0]]); snapshot();
  tap('Thư viện');
  tap('Lời bài hát đồng bộ');
  assert(evaluate(`document.querySelector('.lyrics-drawer').textContent.includes('Tôi đã có lời')`), 'Beginner has an existing-lyrics entry');
  tap('Tôi đã có lời', true);
  const textbox = Object.entries(refs).find(([, r]) => r.role === 'textbox' && r.name.includes('Chỉnh sửa lời'));
  cli(['fill', '@' + textbox[0], 'Xin']); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer textarea').value === 'Xin'`));
  const savedBeforePipeline = evaluate(`window.__lyricTest.saved`);
  evaluate(`window.__lyricTest.pipelineDelay = true`);
  tap('Viết lại bằng AI', true); tap('Bắt đầu viết lại', true);
  const path = evaluate(`window.__lyricTest.pipelinePath`);
  for (const [stage, message] of [[1, 'Chép phụ đề'], [2, 'Nghe đối chiếu ngữ cảnh'], [3, 'Phân câu theo chỗ ngắt'], [4, 'Căn từng chữ'], [5, 'Kiểm tra trước khi xuất']]) {
    evaluate(`window.__uiEmit('lyrics-progress', ${JSON.stringify({ path, stage, message, state: 'running' })})`); snapshot();
    assert(evaluate(`document.querySelector('.lyrics-drawer .working').textContent.includes(${JSON.stringify(stage + '/5 · ' + message)})`), 'Actual progress event is visible for this track');
    assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer .replacement-preview').length`), 0, 'No intermediate lyric is published');
  }
  evaluate(`window.__uiEmit('lyrics-progress', { path: ${JSON.stringify(path)}, stage: 3, total: 6, message: 'Viết lại ngữ cảnh, dấu câu và viết hoa', state: 'running' })`); snapshot();
  assert(evaluate(`document.querySelector('.lyrics-drawer .working').textContent.includes('3/6 · Viết lại ngữ cảnh')`), 'Optional rewrite adds a real sixth stage to progress');
  cli(['screenshot', 'body', '.pi/ui-check/lyrics-pipeline-progress.png']);
  evaluate(`window.__uiEmit('lyrics-progress', { path: 'other.wav', stage: 2, message: 'Other song', state: 'running' })`); snapshot();
  assert.equal(evaluate(`document.querySelectorAll('.lyrics-drawer .working').length`), 0, 'Another recording cannot display its progress here');
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforePipeline, 'Progress cannot overwrite saved lyrics');
  evaluate(`window.__lyricTest.pipelineRelease()`);
  cli(['wait', '.lyrics-drawer .replacement-preview']); snapshot();
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforePipeline, 'Completed pipeline still requires choosing preview and saving');
  tap('Giữ bản cũ', true);
  evaluate(`Object.assign(window.__lyricTest, { pipelineDelay: false, unresolved: true })`);
  tap('Viết lại bằng AI', true); tap('Bắt đầu viết lại', true);
  cli(['wait', '.lyrics-drawer .replacement-preview']); snapshot();
  assert(evaluate(`document.querySelector('.replacement-preview h4').textContent.includes('còn chỗ chưa nghe rõ')`), 'Uncertain result is a draft, not a completed lyric');
  assert.equal(evaluate(`window.__lyricTest.saved`), savedBeforePipeline, 'Uncertain preview preserves saved lyrics');
  tap('Chỉnh sửa bản nháp');
  assert(evaluate(`document.activeElement === document.querySelector('.lyrics-drawer textarea')`), 'Choosing a draft focuses the editor, including short windows');
  assert(evaluate(`document.querySelector('.lyrics-drawer textarea').value.includes('⟦không nghe rõ⟧')`), 'Unknown audio is visibly marked instead of guessed');
  for (const label of ['Lưu lời', 'AI rà soát', 'Tạo mốc ước lượng']) {
    assert(evaluate(`[...document.querySelectorAll('.lyrics-drawer button')].find(b => b.textContent.trim() === ${JSON.stringify(label)}).disabled`), label + ' cannot treat unresolved words as confirmed');
  }
  cli(['screenshot', 'body', '.pi/ui-check/lyrics-unresolved-draft.png']);
  evaluate(`(() => { const editor = document.querySelector('.lyrics-drawer textarea'); editor.value = 'Xin chào!'; editor.dispatchEvent(new Event('input', { bubbles: true })); })()`); snapshot();
  tap('Lưu lời', true);
  assert.equal(evaluate(`window.__lyricTest.saved`), 'Xin chào!', 'Human-confirmed capitalization and punctuation are preserved when saving');
  assert.equal(cli(['errors']).errors?.length ?? 0, 0, 'No browser errors');
  console.log('PASS: Lyrics review/alignment/privacy, safe delete/rewrite preview/save, multi-pass/context-rewrite consent, shared standards, uncertain draft guards, progress isolation and narrow layout (mock IPC)');
} catch (error) {
  try {
    console.error(JSON.stringify(evaluate(`({ test: window.__lyricTest, panels: [...document.querySelectorAll('.lyrics-panel')].map(p => ({ text: p.innerText, editor: p.querySelector('textarea')?.value })), errors: document.querySelector('[role=alert]')?.textContent })`)));
    console.error(JSON.stringify(cli(['errors'])));
  } catch {}
  throw error;
} finally {
  try { cli(['close']); } finally { server.kill(); rmSync(script, { force: true }); }
}
