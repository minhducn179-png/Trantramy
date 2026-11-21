import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';

class AudioMock extends EventTarget {
  paused = true; ended = false; currentTime = 0; duration = 60; src = ''; volume = .8;
  play() { this.paused = false; this.ended = false; this.dispatchEvent(new Event('play')); return Promise.resolve(); }
  pause() { this.paused = true; this.dispatchEvent(new Event('pause')); }
  finish() { this.currentTime = 60; this.ended = true; this.paused = true; this.dispatchEvent(new Event('ended')); }
}
const calls = [];
let relatedReply;
globalThis.Audio = AudioMock;
globalThis.__karaokeTest = {
  convertFileSrc: path => path,
  coverSrc: () => '',
  onlineToTrack: track => ({ ...track, id: -1, path: track.url, cover_path: null }),
  audioEngine: { bindAudio() {}, syncAudioGain() {} },
  api: {
    recordPlay: async () => {},
    searchOnline: () => { calls.push('related'); return new Promise(resolve => relatedReply = resolve); },
    getRecommendations: async () => [],
    resolveStream: async url => url,
  },
};
try {
  const source = (await readFile(new URL('../src/lib/player.svelte.ts', import.meta.url), 'utf8'))
    .replace(/^import .*;$/gm, '')
    .replace('export type RepeatMode', 'const { convertFileSrc, coverSrc, onlineToTrack, audioEngine, api } = globalThis.__karaokeTest;\nexport type RepeatMode');
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
  const runtime = pathToFileURL(createRequire(import.meta.url).resolve('svelte/internal/client')).href;
  const code = compileModule(js, { filename: 'player.svelte.js', generate: 'client', dev: false }).js.code
    .replaceAll('"svelte/internal/client"', JSON.stringify(runtime)).replaceAll("'svelte/internal/client'", JSON.stringify(runtime));
  const { player } = await import(`data:text/javascript;base64,${Buffer.from(code).toString('base64')}`);
  const track = id => ({ id, path: `track-${id}.wav`, title: `Bài ${id}`, artist: 'Ca sĩ', album: '', duration: 60, cover_path: null, source: 'local', url: null });
  player.shuffle = true; player.repeat = 'one'; player.autoRelated = true;
  player.setKaraoke(true);
  player.playList([track(1), track(2), track(3), track(4)]);
  assert.equal(player.current.id, 1);
  player.moveNext(3);
  assert.deepEqual(player.queue.map(t => t.id), [1, 4, 2, 3]);
  assert.equal(player.current.id, 1, 'priority does not interrupt current song');
  for (const invalid of [-1, 0, 9, NaN, 1.5]) player.moveNext(invalid);
  assert.deepEqual(player.queue.map(t => t.id), [1, 4, 2, 3]);
  player.removeFromQueue(2);
  player.audio.finish();
  assert.equal(player.current.id, 4, 'natural ending advances FIFO despite repeat-one and shuffle');
  player.replaceCurrent(track(5));
  assert.equal(player.current.id, 5);
  assert.deepEqual(player.queue.map(t => t.id), [1, 5, 3], 'replacement preserves upcoming songs');
  player.audio.finish();
  assert.equal(player.current.id, 3);
  player.audio.finish();
  assert.equal(player.playing, false);
  assert.equal(player.current.id, 3, 'room stops at end rather than looping');
  assert.equal(calls.length, 0, 'room never adds related songs');
  player.enqueue(track(6));
  assert.equal(player.current.id, 6, 'calling a song after the queue ends starts the new turn');
  player.setKaraoke(false);
  assert.equal(player.shuffle, true); assert.equal(player.repeat, 'one'); assert.equal(player.autoRelated, true);
  assert.equal(player.current.id, 6, 'closing room keeps current playback');
  player.audio.finish();
  assert.equal(player.current.id, 6, 'normal repeat-one is restored');
  assert.equal(player.audio.currentTime, 0);

  player.shuffle = false; player.repeat = 'off';
  player.playList([track(7)]);
  player.audio.finish();
  assert.equal(calls.length, 1);
  player.setKaraoke(true);
  relatedReply([{ title: 'Not requested', artist: 'Other', url: 'https://example.invalid/unrequested', duration: 60, source: 'youtube' }]);
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.equal(player.queue.length, 1, 'late normal-mode recommendation cannot join karaoke queue');
  player.clearQueue();
  player.enqueue(track(8));
  assert.equal(player.current.id, 8, 'first called song starts without an existing queue');
  player.next();
  assert.equal(player.playing, false, 'skipping the last turn stops playback');
  player.enqueue(track(9));
  assert.equal(player.current.id, 9, 'calling a song after manually skipping the last turn starts it');
  player.removeFromQueue(1);
  player.removeFromQueue(0);
  assert.equal(player.index, -1); assert.equal(player.playing, false);
  console.log('PASS: karaoke FIFO, priority/removal, explicit version replacement, natural end, late lookups and playback preferences');
} finally {
  delete globalThis.__karaokeTest;
  delete globalThis.Audio;
}
