import { listen } from "@tauri-apps/api/event";
import { api, type LyricsQueue, type LyricsProgress } from "$lib/api";

class LyricsState {
  revision = $state(0);
  progress = $state<LyricsProgress | null>(null);
  deletion = $state<{ path: string; version: number } | null>(null);
  queue = $state<LyricsQueue>({ running: false, jobs: [], error: "" });
  download = $state<{ done: number; total: number } | null>(null);
  error = $state("");
  busy = $state(false);
  private subscribers = 0;
  private queueVersion = 0;
  private cleanup: (() => void) | null = null;

  subscribe() {
    if (this.subscribers++ === 0) {
      let disposed = false;
      const listeners = [
        listen<string>("lyrics-updated", () => this.revision++),
        listen<LyricsProgress>("lyrics-progress", (e) => { this.progress = e.payload; }),
        listen<LyricsQueue>("lyrics-queue", (e) => { this.queueVersion++; this.queue = e.payload; }),
        listen<{done: number; total: number}>("lyrics-model-progress", (e) => this.download = e.payload),
      ];
      this.cleanup = () => {
        disposed = true;
        for (const listener of listeners) listener.then(unlisten => unlisten()).catch(() => {});
      };
      const version = this.queueVersion;
      api.lyricsQueue().then(q => { if (!disposed && version === this.queueVersion) this.queue = q; }).catch(e => { if (!disposed) this.error = String(e); });
    }
    return () => {
      if (--this.subscribers === 0) { this.cleanup?.(); this.cleanup = null; }
    };
  }
  forget(path: string) {
    lyricDrafts.delete(path);
    lyricFreshDrafts.delete(path);
    this.deletion = { path, version: (this.deletion?.version ?? 0) + 1 };
    this.revision++;
  }
  async start(resume = false) {
    if (this.busy) return;
    this.busy = true;
    this.error = "";
    const version = this.queueVersion;
    try { const queue = await api.startLyricsLibrary(resume); if (version === this.queueVersion) this.queue = queue; }
    catch (e) { this.error = String(e); }
    finally { this.busy = false; }
  }
  async pause() {
    try {
      await api.pauseLyricsLibrary();
      const version = this.queueVersion;
      const queue = await api.lyricsQueue();
      if (version === this.queueVersion) this.queue = queue;
    }
    catch (e) { this.error = String(e); }
  }
}
export const lyricsState = new LyricsState();
export const lyricDrafts = new Map<string,string>();
export const lyricFreshDrafts = new Set<string>();
