import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { transpileModule, ScriptTarget } from 'typescript';
import { connect } from 'node:net';

const port = 5198;
const session = `dlc-track-info-${process.pid}`;
const script = `.pi/ui-check/track-info-${process.pid}.js`;
const server = spawn(process.execPath, ['node_modules/vite/bin/vite.js', ...(process.env.DLC_UI_PREVIEW === '1' ? ['preview'] : []), '--host', '127.0.0.1', '--port', String(port), '--strictPort'], { stdio: ['ignore', 'inherit', 'inherit'] });
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
const evaluate = code => cli(['eval', '--stdin'], `(async () => { await new Promise(resolve => setTimeout(resolve, 0)); return (${code}); })()`).result;
const tap = (scope, name, index = 0) => {
  const tree = cli(['snapshot', '-i', '-s', scope]);
  const matches = Object.entries(tree.refs).filter(([ref, r]) => tree.snapshot.includes(`[ref=${ref}]`) && r.role === 'button' && (name === null || r.name === name));
  assert(matches[index], `Missing button ${name} in ${scope}`);
  cli(['click', '@' + matches[index][0]]);
  cli(['snapshot', '-i']);
};
const detail = () => evaluate(`document.querySelector('.sc-track-title')?.textContent`);
const current = () => evaluate(`document.querySelector('.deck .title')?.textContent.trim()`);

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
  writeFileSync(script, transpileModule(readFileSync('tests/ui-fixture.ts', 'utf8'), { compilerOptions: { target: ScriptTarget.ES2022 } }).outputText);
  cli(['--init-script', script, 'open', `http://127.0.0.1:${port}`]);
  cli(['wait', '.nav-item']);
  tap('.sidebar', 'Thư viện');
  tap('.tbody', 'Chi tiết bài hát', 1);
  assert.equal(detail(), 'Bài hát 2', 'Idle info opens the selected song');
  assert.equal(current(), 'Chưa chọn bài hát', 'Info does not start playback');
  tap('.sidebar', 'Thư viện');
  tap('.tbody', 'Phát Bài hát 3');
  assert.equal(detail(), 'Bài hát 3');
  assert.equal(current(), 'Bài hát 3');
  tap('.sidebar', 'Thư viện');
  tap('.tbody', 'Chi tiết bài hát', 1);
  assert.equal(detail(), 'Bài hát 2', 'Inspect B while playing A keeps B selected');
  assert.equal(current(), 'Bài hát 3', 'Info does not replace the playback song');
  assert(evaluate(`window.__uiCalls.filter(c => c.command === 'lyrics_read').at(-1).args.track.path === 'fixture-1.wav'`), 'Embedded lyrics use the inspected song');
  tap('.deck', 'Đến lời bài hát trên trang thông tin');
  assert(evaluate(`document.activeElement.id === 'track-detail-lyrics'`), 'Lyrics shortcut focuses the inspected song');
  assert.equal(evaluate(`document.querySelectorAll('.karaoke-room').length`), 0, 'Inspector shortcut does not open a second lyrics view');
  tap('.deck', 'Hàng chờ phát');
  tap('.deck', 'Đến lời bài hát trên trang thông tin');
  assert.equal(evaluate(`document.querySelectorAll('.queue-drawer').length`), 0, 'Lyrics shortcut closes the queue before focusing');
  tap('.deck', 'Bài tiếp theo');
  assert.equal(current(), 'Bài hát 4');
  assert.equal(detail(), 'Bài hát 2', 'Transport does not overwrite the inspected song');
  tap('.identity', null);
  assert.equal(detail(), 'Bài hát 4', 'Footer opens its displayed song when another song is inspected');
  tap('.identity', null);
  assert.equal(detail(), null, 'Footer toggles its own inspector closed');
  tap('.identity', null);
  assert.equal(detail(), 'Bài hát 4', 'Footer reopens its displayed song');
  tap('.sidebar', 'Trang chủ');
  cli(['wait', '.hero-detail-btn']);
  tap('.hero-actions', 'Xem chi tiết');
  const onlineTitle = detail();
  assert(onlineTitle.startsWith('Bản nhạc tiếng Việt'), 'Home details inspect the online recommendation');
  assert.equal(current(), 'Bài hát 4', 'Home details do not replace playback');
  tap('.detail-stage', 'Phát bài hát này');
  assert.equal(current(), onlineTitle);
  tap('.sidebar', 'Trang chủ');
  cli(['wait', '.hero-detail-btn']);
  tap('.hero-actions', 'Xem chi tiết');
  assert.equal(detail(), onlineTitle);
  tap('.identity', null);
  assert.equal(detail(), null, 'Footer recognizes the same online URL despite different temporary IDs');
  evaluate(`(() => {
    const native = window.__TAURI_INTERNALS__, original = native.invoke.bind(native);
    window.__failedStreams = [];
    native.invoke = async (command, args) => {
      if (command === 'resolve_stream') {
        window.__failedStreams.push(args.url);
        if (window.__failedStreams.length > 4) return new Promise(() => {});
        await new Promise(resolve => setTimeout(resolve, 40));
        throw 'Stream unavailable';
      }
      const result = await original(command, args);
      if (command === 'get_home_data') result.recommended = result.recommended.slice(0, 2);
      return result;
    };
  })()`);
  tap('.sidebar', 'Thư viện');
  tap('.sidebar', 'Trang chủ');
  cli(['wait', '.hero-play-btn']);
  tap('.hero-actions', 'Phát ngay');
  assert(evaluate(`(async () => {
    for (let i = 0; i < 100; i++) {
      if (window.__failedStreams.length >= 2 && !document.querySelector('.stream-pill')) return true;
      await new Promise(resolve => setTimeout(resolve, 20));
    }
    return false;
  })()`), 'Broken streams finish attempting instead of cycling forever');
  assert.equal(evaluate(`window.__failedStreams.length`), 2, 'Each broken song is tried only once');
  assert.equal(current(), 'Bài hát 2', 'Footer stays on the final failed song');
  assert(evaluate(`document.querySelector('.error-pill').textContent.includes('Không lấy được luồng')`), 'The actual stream error remains visible');
  await evaluate(`new Promise(resolve => setTimeout(resolve, 300))`);
  assert.equal(evaluate(`window.__failedStreams.length`), 2, 'Repeat-all does not restart failed songs');
  assert.equal(current(), 'Bài hát 2', 'Track information remains stable after failure');
  assert.equal(cli(['errors']).errors?.length ?? 0, 0, 'No browser errors');
  console.log('PASS: selected-song info, lyrics, playback isolation, transport, footer toggle and two-stream failure stability (mock IPC)');
} finally {
  try { cli(['close']); } finally { server.kill(); rmSync(script, { force: true }); }
}
