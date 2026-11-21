import { convertFileSrc } from "@tauri-apps/api/core";
import { api, coverSrc, onlineToTrack, type OnlineTrack, type Track } from "./api";
import { audioEngine } from "./audioEngine.svelte";

export type RepeatMode = "off" | "all" | "one";

class Player {
  queue = $state<Track[]>([]);
  index = $state(-1);
  playing = $state(false);
  loadingStream = $state(false);
  karaoke = $state(false);
  position = $state(0);
  duration = $state(0);
  volume = $state(0.8);
  shuffle = $state(
    typeof localStorage !== "undefined" && localStorage.getItem("dlc.player.shuffle") === "1",
  );
  repeat = $state<RepeatMode>(
    (typeof localStorage !== "undefined" &&
      (localStorage.getItem("dlc.player.repeat") as RepeatMode)) ||
      "all",
  );
  error = $state("");
  autoRelated = $state(
    typeof localStorage === "undefined" || localStorage.getItem("dlc.player.auto_related") !== "0",
  ); // Auto-play related songs when queue ends

  audio: HTMLAudioElement | null = null;
  private order: number[] = [];
  private loadToken = 0;
  private recordedPlay = false;
  private endedGuardFired = false;
  private stalledTimer: ReturnType<typeof setTimeout> | null = null;
  private fetchingRelated = false; // prevent concurrent fetches
  private karaokeWaiting = false;

  get current(): Track | null {
    return this.queue[this.index] ?? null;
  }

  init() {
    if (this.audio) return;
    const a = new Audio();
    a.volume = this.volume;
    a.addEventListener("timeupdate", () => {
      this.position = a.currentTime;
      // Fallback auto-next: if timeupdate shows we're within 0.5s of the end
      // and the 'ended' event hasn't fired yet, force advance after a short delay
      if (
        !this.endedGuardFired &&
        a.duration > 0 &&
        isFinite(a.duration) &&
        a.currentTime >= a.duration - 0.5
      ) {
        this.endedGuardFired = true;
        setTimeout(() => {
          // Only fire if still at the end (the 'ended' handler didn't already advance)
          if (this.audio && !this.audio.seeking && this.audio.paused && this.audio.ended) {
            this.onEnded();
          } else if (this.audio && this.audio.currentTime >= this.audio.duration - 0.3) {
            this.onEnded();
          }
        }, 800);
      }
      const cur = this.current;
      if (cur && !this.recordedPlay) {
        const threshold = Math.min(30, (a.duration || cur.duration || 60) * 0.5);
        if (a.currentTime >= Math.max(5, threshold)) {
          this.recordedPlay = true;
          api
            .recordPlay(cur.title, cur.artist, cur.source || "local", cur.url ?? null, false)
            .catch(() => {});
        }
      }
    });
    a.addEventListener("durationchange", () => (this.duration = a.duration || 0));
    a.addEventListener("play", () => (this.playing = true));
    a.addEventListener("pause", () => (this.playing = false));
    a.addEventListener("ended", () => this.onEnded());
    a.addEventListener("stalled", () => {
      // If audio stalls near the end, treat it as ended
      if (a.duration > 0 && isFinite(a.duration) && a.currentTime >= a.duration - 1) {
        this.onEnded();
        return;
      }
      // If audio stalls mid-stream for too long, skip to next
      if (this.stalledTimer) clearTimeout(this.stalledTimer);
      this.stalledTimer = setTimeout(() => {
        if (this.audio && this.playing && this.audio.readyState < 3) {
          console.warn("Audio stalled for too long, skipping to next track");
          this.error = `Mất tín hiệu: ${this.current?.title ?? ""}`;
          this.next(true);
        }
      }, 12000);
    });
    a.addEventListener("playing", () => {
      if (this.stalledTimer) { clearTimeout(this.stalledTimer); this.stalledTimer = null; }
    });
    a.addEventListener("error", (ev) => {
      console.error("Audio playback error:", a.error, a.src, ev);
      this.error = `Không phát được: ${this.current?.title ?? ""}`;
      this.next(true);
    });
    this.audio = a;
    audioEngine.bindAudio(a, () => this.volume);
    this.setupMediaSession();
  }

  private setupMediaSession() {
    if (!("mediaSession" in navigator)) return;
    navigator.mediaSession.setActionHandler("play", () => this.toggle());
    navigator.mediaSession.setActionHandler("pause", () => this.toggle());
    navigator.mediaSession.setActionHandler("nexttrack", () => this.next());
    navigator.mediaSession.setActionHandler("previoustrack", () => this.prev());
  }

  /** Replace the queue and start playing at `start`. */
  playList(tracks: Track[], start = 0) {
    this.init();
    this.queue = [...tracks];
    this.buildOrder(start);
    this.load(start);
  }

  playOnlineList(items: OnlineTrack[], start = 0) {
    this.playList(items.map(onlineToTrack), start);
  }

  /** Turn shuffle on and start playing `tracks` from a random track. */
  playRandom(tracks: Track[]) {
    if (tracks.length === 0) return;
    this.shuffle = true;
    this.playList(tracks, Math.floor(Math.random() * tracks.length));
  }

  enqueue(t: Track) {
    this.queue = [...this.queue, t];
    this.order.push(this.queue.length - 1);
    if (this.index < 0) {
      this.init();
      this.load(0);
    } else if (this.karaoke && (this.karaokeWaiting || this.audio?.ended)) {
      this.load(this.queue.length - 1);
    }
  }

  setKaraoke(enabled: boolean) {
    this.karaoke = enabled;
    if (!enabled) this.karaokeWaiting = false;
    this.buildOrder(this.index);
  }

  moveNext(i: number) {
    if (!Number.isInteger(i) || i <= this.index || i >= this.queue.length) return;
    const queue = [...this.queue];
    const [track] = queue.splice(i, 1);
    queue.splice(this.index + 1, 0, track);
    this.queue = queue;
    this.buildOrder(this.index);
  }

  replaceCurrent(track: Track) {
    if (this.index < 0) { this.enqueue(track); return; }
    this.queue = this.queue.map((item, i) => i === this.index ? track : item);
    this.load(this.index);
  }

  enqueueOnline(o: OnlineTrack) {
    this.enqueue(onlineToTrack(o));
  }

  removeFromQueue(i: number) {
    if (i < 0 || i >= this.queue.length) return;
    const wasCurrent = i === this.index;
    this.queue = this.queue.filter((_, k) => k !== i);
    if (i < this.index) this.index--;
    this.buildOrder(Math.min(this.index, this.queue.length - 1));
    if (this.queue.length === 0) {
      this.audio?.pause();
      this.index = -1;
    } else if (wasCurrent) {
      this.load(Math.min(i, this.queue.length - 1));
    }
  }

  private buildOrder(first: number) {
    const idx = this.queue.map((_, i) => i);
    if (this.shuffle && !this.karaoke) {
      for (let i = idx.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [idx[i], idx[j]] = [idx[j], idx[i]];
      }
      const p = idx.indexOf(first);
      if (p > 0) [idx[0], idx[p]] = [idx[p], idx[0]];
    }
    this.order = idx;
  }

  private async load(i: number, autoplay = true) {
    const t = this.queue[i];
    if (!t || !this.audio) return;
    const myToken = ++this.loadToken;
    this.karaokeWaiting = false;
    this.index = i;
    this.error = "";
    this.position = 0;
    this.duration = t.duration || 0;
    this.recordedPlay = false;
    this.endedGuardFired = false;
    if (this.stalledTimer) { clearTimeout(this.stalledTimer); this.stalledTimer = null; }

    const isRemote =
      t.path.startsWith("http://") ||
      t.path.startsWith("https://") ||
      t.source === "spotify";
    if (isRemote) {
      this.audio.pause();
      this.loadingStream = true;
      try {
        let playUrl = t.url || t.path;
        // If it's a Spotify link, search YouTube for corresponding audio stream
        if (t.source === "spotify" || playUrl.includes("spotify.com")) {
          const query = `${t.artist} - ${t.title} official audio`;
          const results = await api.searchOnline(query, "youtube", 1).catch(() => []);
          if (results.length > 0) {
            playUrl = results[0].url;
            t.url = playUrl;
            t.path = playUrl;
            if (!t.cover_path && results[0].thumbnail) {
              t.cover_path = results[0].thumbnail;
            }
          }
        }
        const rawStreamUrl = await api.resolveStream(playUrl);
        if (myToken !== this.loadToken) return;
        this.loadingStream = false;
        this.audio.src =
          rawStreamUrl.startsWith("http://") || rawStreamUrl.startsWith("https://")
            ? rawStreamUrl
            : convertFileSrc(rawStreamUrl);
      } catch (e) {
        if (myToken !== this.loadToken) return;
        this.loadingStream = false;
        this.error = `Không lấy được luồng: ${t.title} (${e})`;
        this.next(true);
        return;
      }
    } else {
      this.loadingStream = false;
      this.audio.src = t.path.startsWith("blob:") ? t.path : convertFileSrc(t.path);
    }

    if (autoplay) {
      this.audio.play().catch((err) => {
        console.warn("Autoplay was prevented:", err);
      });
    }
    this.prefetchNext(i);
    if ("mediaSession" in navigator) {
      const art = coverSrc(t.cover_path);
      navigator.mediaSession.metadata = new MediaMetadata({
        title: t.title,
        artist: t.artist,
        album: t.album,
        artwork: art ? [{ src: art }] : [],
      });
    }
  }

  private prefetchNext(i: number) {
    if (this.queue.length <= 1) return;
    const pos = this.order.indexOf(i);
    const nextIdx =
      pos + 1 < this.order.length
        ? this.order[pos + 1]
        : this.repeat === "all" && !this.karaoke
          ? this.order[0]
          : -1;
    if (nextIdx >= 0 && nextIdx !== i) {
      const nextTrack = this.queue[nextIdx];
      if (nextTrack) {
        const nextUrl = nextTrack.url || nextTrack.path;
        if (nextTrack.source === "spotify" || nextUrl.includes("spotify.com")) {
          const q = `${nextTrack.artist} - ${nextTrack.title} official audio`;
          api.searchOnline(q, "youtube", 1).catch(() => {});
        } else if (nextUrl.startsWith("http://") || nextUrl.startsWith("https://")) {
          // Pre-resolve stream URL in background so next track plays with 0 delay!
          api.resolveStream(nextUrl).catch(() => {});
        }
      }
    }
  }

  toggle() {
    this.init();
    if (!this.current && this.queue.length > 0) {
      this.load(Math.max(0, this.index));
      return;
    }
    if (!this.audio || !this.current) return;
    if (this.audio.paused) this.audio.play().catch(() => {});
    else this.audio.pause();
  }

  play() {
    this.init();
    if (!this.current && this.queue.length > 0) {
      this.load(Math.max(0, this.index));
      return;
    }
    if (this.audio && this.audio.paused) {
      this.audio.play().catch(() => {});
    }
  }

  pause() {
    if (this.audio && !this.audio.paused) {
      this.audio.pause();
    }
  }

  seek(t: number) {
    if (this.audio) this.audio.currentTime = t;
  }

  clearQueue() {
    this.audio?.pause();
    this.queue = [];
    this.order = [];
    this.index = -1;
    this.playing = false;
    this.position = 0;
    this.duration = 0;
  }

  setVolume(v: number) {
    this.volume = v;
    audioEngine.syncAudioGain();
  }

  toggleShuffle() {
    this.shuffle = !this.shuffle;
    try {
      localStorage.setItem("dlc.player.shuffle", this.shuffle ? "1" : "0");
    } catch {}
    this.buildOrder(this.index);
  }

  cycleRepeat() {
    this.repeat = this.repeat === "off" ? "all" : this.repeat === "all" ? "one" : "off";
    try {
      localStorage.setItem("dlc.player.repeat", this.repeat);
    } catch {}
  }

  toggleAutoRelated() {
    this.autoRelated = !this.autoRelated;
    try {
      localStorage.setItem("dlc.player.auto_related", this.autoRelated ? "1" : "0");
    } catch {}
  }

  next(auto = false) {
    if (this.queue.length === 0) return;
    const pos = this.order.indexOf(this.index);
    if (pos + 1 < this.order.length) {
      this.load(this.order[pos + 1]);
    } else if (this.karaoke) {
      this.karaokeWaiting = true;
      this.audio?.pause();
    } else if (this.repeat === "all" || (!auto && this.queue.length > 0)) {
      this.buildOrder(this.order[0] ?? 0);
      this.load(this.order[0]);
    } else if (auto && this.autoRelated) {
      // Queue exhausted & repeat=off → fetch a related song
      this.fetchAndPlayRelated();
    } else {
      this.audio?.pause();
    }
  }

  /** Search for a song related to the current track + listening history, append & play it. */
  private async fetchAndPlayRelated() {
    if (this.fetchingRelated) return;
    this.fetchingRelated = true;
    this.loadingStream = true;
    try {
      const cur = this.current;
      // Build a search query from current track + recent history for variety
      let query = "";
      if (cur) {
        // Primary: artist + "similar songs" or track-based query
        const parts: string[] = [];
        if (cur.artist && cur.artist !== "Unknown") parts.push(cur.artist);
        if (cur.title) parts.push(cur.title);
        query = parts.join(" ") + " similar songs";
      }
      // Fallback: use recommendations from backend (based on play history)
      if (!query) {
        const recs = await api.getRecommendations(false).catch(() => []);
        if (recs.length > 0) {
          const pick = recs[Math.floor(Math.random() * Math.min(recs.length, 5))];
          this.appendOnlineAndPlay(pick);
          return;
        }
        this.audio?.pause();
        return;
      }
      // Search online for related tracks
      const results = await api.searchOnline(query, "youtube", 8).catch(() => []);
      if (results.length === 0) {
        // Fallback to recommendations
        const recs = await api.getRecommendations(false).catch(() => []);
        if (recs.length > 0) {
          const pick = recs[Math.floor(Math.random() * Math.min(recs.length, 5))];
          this.appendOnlineAndPlay(pick);
          return;
        }
        this.audio?.pause();
        return;
      }
      // Filter out tracks already in the queue to avoid repeats
      const queueUrls = new Set(this.queue.map((t) => t.url).filter(Boolean));
      const queueTitles = new Set(this.queue.map((t) => t.title.toLowerCase()));
      const fresh = results.filter(
        (r) => !queueUrls.has(r.url) && !queueTitles.has(r.title.toLowerCase()),
      );
      const pick = fresh.length > 0
        ? fresh[Math.floor(Math.random() * Math.min(fresh.length, 4))]
        : results[Math.floor(Math.random() * Math.min(results.length, 4))];
      this.appendOnlineAndPlay(pick);
    } catch (e) {
      console.error("Auto-related fetch failed:", e);
      this.audio?.pause();
    } finally {
      this.fetchingRelated = false;
      this.loadingStream = false;
    }
  }

  private appendOnlineAndPlay(item: OnlineTrack) {
    if (this.karaoke) return; // A lookup started before entering the room must not add an unrequested song.
    const t = onlineToTrack(item);
    this.queue.push(t);
    const newIdx = this.queue.length - 1;
    this.order.push(newIdx);
    this.load(newIdx);
  }

  prev() {
    if (this.queue.length === 0) return;
    if (this.position > 3) return this.seek(0);
    const pos = this.order.indexOf(this.index);
    const p = pos > 0 ? pos - 1 : this.order.length - 1;
    this.load(this.order[p]);
  }

  private onEnded() {
    // Guard: prevent double-fire from both 'ended' event and fallback timer
    if (this.endedGuardFired && this.audio && this.audio.currentTime < 1) return;
    this.endedGuardFired = true;

    const cur = this.current;
    if (cur) {
      api
        .recordPlay(cur.title, cur.artist, cur.source || "local", cur.url ?? null, true)
        .catch(() => {});
    }
    if (this.repeat === "one" && !this.karaoke) {
      this.seek(0);
      this.audio?.play().catch(() => {});
    } else {
      this.next(true);
    }
  }

  jumpTo(i: number) {
    this.load(i);
  }
}

export const player = new Player();
