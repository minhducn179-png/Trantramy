// Store and configuration for Lyrics & Karaoke Studio UI
export type LyricsLayoutMode = 'vinyl' | 'stage' | 'full';
export type VinylSkin = 'marble' | 'album' | 'black' | 'cyberpunk' | 'gold' | 'custom';
export type VinylCoverPos = 'center' | 'rim' | 'full';
export type LyricsFontSize = 'compact' | 'normal' | 'large';
export type LyricsDimInactive = 'soft' | 'deep' | 'hidden';
export type LyricsBgStyle = 'apple-glass' | 'glow' | 'oled' | 'wallpaper';
/** Chế độ cửa sổ hiển thị lời bài hát:
 *  - 'fullscreen': toàn màn hình (overlay phủ toàn bộ ứng dụng)
 *  - 'mini': thanh bên phải như Spotify (sidebar hiển thị lời bên cạnh stage chính)
 */
export type LyricsWindowMode = 'fullscreen' | 'mini';

export interface LyricsUiConfig {
  layout: LyricsLayoutMode;
  vinylSkin: VinylSkin;
  vinylCoverPos: VinylCoverPos;
  vinylSpeedRpm: number; // 20 đến 60 RPM
  vinylColorHex: string; // Mã màu hex của đĩa than tùy chọn
  vinylHue: number; // Độ góc quang phổ 0 - 360
  showTonearm: boolean;
  bouncingCursor: boolean;
  showSectionTags: boolean;
  fontSize: LyricsFontSize;
  dimInactive: LyricsDimInactive;
  bgStyle: LyricsBgStyle;
  vinylSpeed: '33' | '45';
  windowMode: LyricsWindowMode;
}

const STORAGE_KEY = 'dlc.lyrics.ui_config';

const DEFAULT_CONFIG: LyricsUiConfig = {
  layout: 'vinyl', // Mac dinh la che do Dia than phong thu nhu video
  vinylSkin: 'marble', // Phong cach Liquid Marble ruc ro nhu video
  vinylCoverPos: 'full', // Mac dinh hien anh toan bo mat dia chuan theo anh mau
  vinylSpeedRpm: 33, // Toc do quay dia than mac dinh 33 RPM
  vinylColorHex: '#a855f7', // Mau sac dia than mac dinh (tim pastel sang trong theo anh mau)
  vinylHue: 270, // Tong quang pho mac dinh
  showTonearm: true,
  bouncingCursor: false, // Con tro nhay chu karaoke theo nhip
  showSectionTags: false, // Nhan verse, hook, drop, outro...
  fontSize: 'normal',
  dimInactive: 'deep',
  bgStyle: 'apple-glass',
  vinylSpeed: '33',
  windowMode: 'fullscreen',
};

function loadConfig(): LyricsUiConfig {
  if (typeof localStorage === 'undefined') return { ...DEFAULT_CONFIG };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_CONFIG };
    const parsed = JSON.parse(raw);
    return { ...DEFAULT_CONFIG, ...parsed, bouncingCursor: false, showSectionTags: false };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

class LyricsUiStore {
  config = $state<LyricsUiConfig>(loadConfig());

  save() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.config));
    } catch {
      // ignore
    }
  }

  setLayout(layout: LyricsLayoutMode) {
    this.config.layout = layout;
    this.save();
  }

  setVinylSkin(skin: VinylSkin) {
    this.config.vinylSkin = skin;
    this.save();
  }

  setVinylColorHex(hex: string) {
    this.config.vinylColorHex = hex;
    this.config.vinylSkin = 'custom';
    this.save();
  }

  setVinylHue(hue: number) {
    this.config.vinylHue = (hue % 360 + 360) % 360;
    this.config.vinylSkin = 'custom';
    this.save();
  }

  setTonearm(show: boolean) {
    this.config.showTonearm = show;
    this.save();
  }

  setBouncingCursor(enable: boolean) {
    this.config.bouncingCursor = enable;
    this.save();
  }

  setSectionTags(show: boolean) {
    this.config.showSectionTags = show;
    this.save();
  }

  setFontSize(size: LyricsFontSize) {
    this.config.fontSize = size;
    this.save();
  }

  setDimInactive(dim: LyricsDimInactive) {
    this.config.dimInactive = dim;
    this.save();
  }

  setBgStyle(bg: LyricsBgStyle) {
    this.config.bgStyle = bg;
    this.save();
  }

  setVinylSpeed(speed: '33' | '45') {
    this.config.vinylSpeed = speed;
    this.config.vinylSpeedRpm = speed === '45' ? 45 : 33;
    this.save();
  }

  setVinylSpeedRpm(rpm: number) {
    this.config.vinylSpeedRpm = Math.min(60, Math.max(15, rpm));
    this.config.vinylSpeed = this.config.vinylSpeedRpm >= 40 ? '45' : '33';
    this.save();
  }

  setVinylCoverPos(pos: VinylCoverPos) {
    this.config.vinylCoverPos = pos;
    this.save();
  }

  setWindowMode(mode: LyricsWindowMode) {
    this.config.windowMode = mode;
    this.save();
  }

  reset() {
    this.config = { ...DEFAULT_CONFIG };
    this.save();
  }
}

export const lyricsUi = new LyricsUiStore();
