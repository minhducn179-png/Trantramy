import { listen } from "@tauri-apps/api/event";

export const EQ_FREQUENCIES = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000] as const;

export const EQ_PRESETS: Record<string, { name: string; gains: number[] }> = {
  flat: {
    name: "Phẳng (Chuẩn)",
    gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  },
  bass: {
    name: "Tăng Bass (Trầm)",
    gains: [6, 5, 3.5, 1.5, 0, 0, 0, 0, 0, 0],
  },
  treble: {
    name: "Tăng Treble (Sáng)",
    gains: [0, 0, 0, 0, 0, 1, 2.5, 4, 5.5, 6],
  },
  vocal: {
    name: "Giọng hát (Vocal)",
    gains: [-2, -1, 0.5, 2, 4, 4, 3, 1.5, 0.5, -1],
  },
  pop: {
    name: "Nhạc Pop",
    gains: [-1, 1.5, 3, 3.5, 2.5, 0, -1, 1.5, 3, 3.5],
  },
  rock: {
    name: "Rock",
    gains: [4.5, 3.5, 1.5, -1, -1.5, 0, 1.5, 3, 4, 4.5],
  },
  edm: {
    name: "EDM & Dance",
    gains: [6, 4.5, 2, 0, -1, 1, 2.5, 4, 5, 4.5],
  },
  acoustic: {
    name: "Acoustic & Chill",
    gains: [3, 2.5, 1.5, 1, 2, 2, 2.5, 3, 3.5, 2.5],
  },
  classical: {
    name: "Cổ điển (Classical)",
    gains: [4, 3, 2, 1.5, -0.5, -0.5, 0, 1.5, 2.5, 3],
  },
  jazz: {
    name: "Jazz & Soul",
    gains: [3.5, 2.5, 1, 1.5, -1, -1, 0, 1, 2.5, 3],
  },
};

const STORAGE_KEY = "dlc.audio.engine.v1";

class AudioEngine {
  private attachedAudio: HTMLAudioElement | null = null;
  private getBaseVolume: () => number = () => 0.8;
  private liveSpectrum: number[] = new Array(32).fill(0);
  private spectrumListening = false;

  // Reactive properties
  enabled = $state(true);
  normalize = $state(true);
  preAmp = $state(0); // -12dB to +12dB
  preset = $state("flat");
  gains = $state<number[]>([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

  constructor() {
    this.loadSettings();
    this.initSpectrumListener();
  }

  private initSpectrumListener() {
    if (this.spectrumListening || typeof window === "undefined") return;
    this.spectrumListening = true;
    listen<number[]>("dlc-audio-spectrum", (ev) => {
      if (Array.isArray(ev.payload) && ev.payload.length > 0) {
        this.liveSpectrum = ev.payload;
      }
    }).catch(() => {});
  }

  private loadSettings() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        const d = JSON.parse(raw);
        if (typeof d.enabled === "boolean") this.enabled = d.enabled;
        if (typeof d.normalize === "boolean") this.normalize = d.normalize;
        if (typeof d.preAmp === "number") this.preAmp = d.preAmp;
        if (typeof d.preset === "string") this.preset = d.preset;
        if (Array.isArray(d.gains) && d.gains.length === 10) {
          this.gains = d.gains.map((n: unknown) => (typeof n === "number" ? n : 0));
        }
      }
    } catch {}
  }

  saveSettings() {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          enabled: this.enabled,
          normalize: this.normalize,
          preAmp: this.preAmp,
          preset: this.preset,
          gains: this.gains,
        }),
      );
    } catch {}
  }

  bindAudio(audio: HTMLAudioElement, getBaseVolume: () => number) {
    this.attachedAudio = audio;
    this.getBaseVolume = getBaseVolume;
    this.syncAudioGain();
  }

  syncAudioGain() {
    if (!this.attachedAudio) return;
    const base = this.getBaseVolume();
    if (base <= 0) {
      this.attachedAudio.volume = 0;
      return;
    }
    if (!this.enabled) {
      this.attachedAudio.volume = Math.max(0, Math.min(1, base));
      return;
    }
    const avgBand = this.gains.reduce((acc, g) => acc + g, 0) / 10;
    const dbOffset = this.preAmp + avgBand * 0.35;
    let mult = Math.pow(10, dbOffset / 24);
    if (this.normalize) {
      // Soft-knee limiter / normalizer to prevent harsh clipping and balance loudness
      mult = 0.94 * Math.tanh(mult * 0.95) / Math.tanh(0.95);
    }
    this.attachedAudio.volume = Math.max(0, Math.min(1, base * mult));
  }

  setGain(bandIdx: number, val: number) {
    if (bandIdx < 0 || bandIdx >= this.gains.length) return;
    const clamped = Math.max(-12, Math.min(12, Math.round(val * 10) / 10));
    this.gains[bandIdx] = clamped;
    this.preset = "custom";
    this.syncAudioGain();
    this.saveSettings();
  }

  setPreAmp(val: number) {
    this.preAmp = Math.max(-12, Math.min(12, Math.round(val * 10) / 10));
    this.syncAudioGain();
    this.saveSettings();
  }

  applyPreset(presetKey: string) {
    const p = EQ_PRESETS[presetKey];
    if (!p) return;
    this.preset = presetKey;
    this.gains = [...p.gains];
    this.syncAudioGain();
    this.saveSettings();
  }

  toggleEnabled() {
    this.enabled = !this.enabled;
    this.syncAudioGain();
    this.saveSettings();
  }

  toggleNormalize() {
    this.normalize = !this.normalize;
    this.syncAudioGain();
    this.saveSettings();
  }

  reset() {
    this.preAmp = 0;
    this.applyPreset("flat");
  }

  getFrequencyData(targetArray: Uint8Array): void {
    const len = targetArray.length;
    const spec = this.liveSpectrum;
    const specLen = spec.length;
    if (!this.attachedAudio || this.attachedAudio.paused || specLen === 0) {
      for (let i = 0; i < len; i++) {
        targetArray[i] = 0;
      }
      return;
    }
    for (let i = 0; i < len; i++) {
      const pos = (i / Math.max(1, len - 1)) * (specLen - 1);
      const i0 = Math.floor(pos);
      const i1 = Math.min(specLen - 1, i0 + 1);
      const frac = pos - i0;
      const rawVal = spec[i0] * (1 - frac) + spec[i1] * frac;
      const bandGain = this.enabled
        ? this.gains[Math.min(9, Math.floor(i / Math.max(1, len / 10)))] || 0
        : 0;
      const val = rawVal + (rawVal > 4 ? bandGain * 3.5 + this.preAmp * 1.5 : 0);
      targetArray[i] = Math.min(255, Math.max(0, Math.round(val)));
    }
  }
}

export const audioEngine = new AudioEngine();
