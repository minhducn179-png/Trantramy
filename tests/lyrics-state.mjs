import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import ts from "typescript";
import { compileModule } from "svelte/compiler";

const listeners = new Map();
let queueReply, startReply;
globalThis.__lyricsStateTest = {
  listen: async (event, callback) => { listeners.set(event, callback); return () => listeners.delete(event); },
  api: {
    lyricsQueue: () => new Promise(resolve => queueReply = resolve),
    startLyricsLibrary: () => new Promise(resolve => startReply = resolve),
    pauseLyricsLibrary: async () => {},
  },
};
try {
  const source = (await readFile(new URL("../src/lib/lyrics.svelte.ts", import.meta.url), "utf8"))
    .replace('import { listen } from "@tauri-apps/api/event";', "const { listen } = globalThis.__lyricsStateTest;")
    .replace('import { api, type LyricsQueue, type LyricsProgress } from "$lib/api";', "const { api } = globalThis.__lyricsStateTest;");
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
  const runtime = pathToFileURL(createRequire(import.meta.url).resolve("svelte/internal/client")).href;
  const compiled = compileModule(js, { filename: "lyrics.svelte.js", generate: "client", dev: false }).js.code
    .replaceAll('"svelte/internal/client"', JSON.stringify(runtime)).replaceAll("'svelte/internal/client'", JSON.stringify(runtime));
  const { lyricsState, lyricDrafts, lyricFreshDrafts } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);
  const stop = lyricsState.subscribe();
  const old = { running: true, jobs: [], error: "old snapshot" };
  const newest = { running: false, jobs: [], error: "newer event" };
  const emit = () => listeners.get("lyrics-queue")({ payload: newest });
  emit(); queueReply(old); await Promise.resolve();
  assert.equal(lyricsState.queue.error, "newer event", "initial snapshot must not replace a newer queue event");
  const start = lyricsState.start(); emit(); startReply(old); await start;
  assert.equal(lyricsState.queue.error, "newer event", "start response must not replace worker progress");
  const pause = lyricsState.pause(); await Promise.resolve(); emit(); queueReply(old); await pause;
  assert.equal(lyricsState.queue.error, "newer event", "pause snapshot must not replace final progress");
  lyricDrafts.set('deleted.wav', 'Draft');
  lyricDrafts.set('other.wav', 'Keep');
  lyricFreshDrafts.add('deleted.wav');
  listeners.get('lyrics-progress')({ payload: { path: 'other.wav', stage: 4, state: 'running', message: 'Đang căn từng chữ' } });
  assert.equal(lyricsState.progress.path, 'other.wav');
  assert.equal(lyricsState.progress.stage, 4);
  const revision = lyricsState.revision;
  lyricsState.forget('deleted.wav');
  assert.equal(lyricDrafts.has('deleted.wav'), false);
  assert.equal(lyricFreshDrafts.has('deleted.wav'), false);
  assert.equal(lyricDrafts.get('other.wav'), 'Keep');
  assert.equal(lyricsState.deletion.path, 'deleted.wav');
  assert.equal(lyricsState.revision, revision + 1);
  const deletionVersion = lyricsState.deletion.version;
  lyricsState.forget('deleted.wav');
  assert.equal(lyricsState.deletion.version, deletionVersion + 1);
  stop();
  console.log("PASS: Lyrics queue preserves newer progress over delayed IPC snapshots");
} finally {
  delete globalThis.__lyricsStateTest;
}
