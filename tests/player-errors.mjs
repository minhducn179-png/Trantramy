import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';

class AudioMock extends EventTarget {
  paused = true; ended = false; currentTime = 0; duration = 60; src = ''; volume = .8;
  play() { this.paused = false; this.dispatchEvent(new Event('play')); return Promise.resolve(); }
  pause() { this.paused = true; this.dispatchEvent(new Event('pause')); }
  fail() { this.dispatchEvent(new Event('error')); }
  finish() { this.currentTime = 60; this.ended = true; this.paused = true; this.dispatchEvent(new Event('ended')); }
}
let resolveStream = async url => url;
let relatedLookups = 0;
const tick = () => new Promise(resolve => setTimeout(resolve, 0));
const originalConsoleError = console.error;
const originalSetTimeout = globalThis.setTimeout;
globalThis.Audio = AudioMock;
globalThis.__playerErrors = {
  convertFileSrc: path => path,
  coverSrc: () => '',
  onlineToTrack: track => ({ ...track, id: -1, path: track.url, cover_path: null }),
  audioEngine: { bindAudio() {}, syncAudioGain() {} },
  api: {
    recordPlay: async () => {},
    searchOnline: async () => { relatedLookups++; return []; },
    getRecommendations: async () => { relatedLookups++; return []; },
    resolveStream: url => resolveStream(url),
  },
};
try {
  const source = (await readFile(new URL('../src/lib/player/player.svelte.ts', import.meta.url), 'utf8'))
    .replace(/^import .*;$/gm, '')
    .replace('export type RepeatMode', 'const { convertFileSrc, coverSrc, onlineToTrack, audioEngine, api } = globalThis.__playerErrors;\nexport type RepeatMode');
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
  const runtime = pathToFileURL(createRequire(import.meta.url).resolve('svelte/internal/client')).href;
  const code = compileModule(js, { filename: 'player.svelte.js', generate: 'client', dev: false }).js.code
    .replaceAll('"svelte/internal/client"', JSON.stringify(runtime)).replaceAll("'svelte/internal/client'", JSON.stringify(runtime));
  const { player } = await import(`data:text/javascript;base64,${Buffer.from(code).toString('base64')}`);
  const track = id => ({ id, path: `track-${id}.wav`, title: `Bài ${id}`, artist: 'Ca sĩ', album: '', duration: 60, cover_path: null, source: 'local', url: null });
  console.error = () => {}; // Expected media errors are asserted through player.error below.
  player.repeat = 'all'; player.autoRelated = true;
  player.playList([track(1), track(2)]);
  player.audio.fail();
  assert.equal(player.current.id, 2, 'One failed song can advance to the next queued song');
  player.audio.fail();
  assert.equal(player.current.id, 2, 'Two failed songs must not alternate forever under repeat-all');
  assert.equal(player.playing, false, 'Exhausted failures stop playback');
  assert.match(player.error, /Bài 2/, 'Keep the final playback error visible');
  for (let i = 0; i < 4; i++) player.audio.fail();
  assert.equal(player.current.id, 2, 'Repeated media errors cannot restart the broken queue');

  player.playList([track(1), track(2)]);
  player.audio.fail();
  player.audio.finish();
  assert.equal(player.current.id, 1, 'A healthy song ending still honors repeat-all');

  const stalls = [];
  globalThis.setTimeout = (callback, delay, ...args) => {
    if (delay !== 12000) return originalSetTimeout(callback, delay, ...args);
    stalls.push(() => callback(...args));
    return 0;
  };
  player.playList([track(1), track(2)]);
  player.audio.currentTime = 0;
  player.audio.readyState = 0;
  player.audio.dispatchEvent(new Event('stalled'));
  stalls.shift()();
  assert.equal(player.current.id, 2);
  player.audio.dispatchEvent(new Event('stalled'));
  stalls.shift()();
  assert.equal(player.current.id, 2, 'Stall timeouts also stop instead of wrapping the broken queue');
  assert.equal(player.playing, false);
  globalThis.setTimeout = originalSetTimeout;

  const requests = [];
  resolveStream = url => new Promise((resolve, reject) => requests.push({ url, resolve, reject }));
  const remote = id => ({ ...track(id), source: 'youtube', url: `https://example.invalid/${id}`, path: `https://example.invalid/${id}` });
  player.playList([remote(3), remote(4)]);
  requests[0].reject(new Error('Stream unavailable'));
  await tick();
  assert.equal(player.current.id, 4);
  requests[1].reject(new Error('Stream unavailable'));
  await tick();
  assert.equal(requests.length, 2, 'Resolution errors stop after one forward pass, without retries or recommendations');
  assert.equal(player.current.id, 4);
  assert.equal(player.loadingStream, false);
  assert.equal(player.playing, false);
  assert.match(player.error, /Bài 4/);
  assert.equal(relatedLookups, 0, 'Playback failures never launch automatic recommendations');
  player.toggle();
  assert.equal(requests.length, 3, 'Explicit play retries the displayed song instead of playing an older source');
  requests[2].resolve('recovered.wav');
  await tick();
  assert.equal(player.audio.src, 'recovered.wav');
  assert.equal(player.playing, true);
  const beforeRetry = requests.length; // A recovered stream can prefetch the next song.
  player.audio.fail();
  player.play();
  assert.equal(requests.length, beforeRetry + 1, 'The explicit play command also retries the displayed song');
  assert.equal(requests.at(-1).url, remote(4).url);
  requests.at(-1).resolve('recovered-again.wav');
  await tick();
  assert.equal(player.audio.src, 'recovered-again.wav');

  player.setKaraoke(true);
  player.playList([track(5)]);
  player.audio.fail();
  player.enqueue(track(6));
  assert.equal(player.current.id, 6, 'Karaoke can resume with a newly requested song after a failure');
  console.log('PASS: bounded media/stream failures, stable track info, explicit retry and normal repeat/karaoke behavior');
} finally {
  console.error = originalConsoleError;
  globalThis.setTimeout = originalSetTimeout;
  delete globalThis.__playerErrors;
  delete globalThis.Audio;
}
