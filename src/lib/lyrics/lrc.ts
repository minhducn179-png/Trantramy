export interface LyricWord {
  text: string;
  start: number;
  end: number;
}

export interface LyricLine {
  time: number;
  endTime: number;
  text: string;
  words: LyricWord[];
  hasWordSync?: boolean;
}

export interface ParsedLyrics {
  synced: boolean;
  hasWordSync: boolean;
  lines: LyricLine[];
}

const TS = /\[(\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)\]/g;
const WORD_TS = /<(\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)>|\((\d{1,3}):(\d{1,2}(?:[.:]\d{1,3})?)\)/g;

function parseTimestamp(min: string, sec: string): number {
  return parseInt(min, 10) * 60 + parseFloat(sec.replace(":", "."));
}

function parseEnhancedWords(remainder: string, lineTime: number): LyricWord[] | null {
  const matches = [...remainder.matchAll(WORD_TS)];
  if (matches.length === 0) return null;

  const words: LyricWord[] = [];
  const leadText = remainder.slice(0, matches[0].index).trim();
  if (leadText) {
    const firstTime = parseTimestamp(
      matches[0][1] || matches[0][3],
      matches[0][2] || matches[0][4],
    );
    words.push({
      text: leadText,
      start: lineTime,
      end: Math.max(lineTime, firstTime),
    });
  }

  for (let i = 0; i < matches.length; i++) {
    const m = matches[i];
    const startTime = parseTimestamp(m[1] || m[3], m[2] || m[4]);

    const textStart = (m.index ?? 0) + m[0].length;
    const textEnd = i + 1 < matches.length ? (matches[i + 1].index ?? remainder.length) : remainder.length;
    const wordText = remainder.slice(textStart, textEnd).trim();

    const nextMatch = i + 1 < matches.length ? matches[i + 1] : null;
    const endTime = nextMatch
      ? parseTimestamp(nextMatch[1] || nextMatch[3], nextMatch[2] || nextMatch[4])
      : startTime + Math.max(0.4, wordText.length * 0.12);

    if (wordText) {
      const subWords = wordText.split(/\s+/).filter(Boolean);
      if (subWords.length > 1) {
        const span = Math.max(0, endTime - startTime);
        const subDur = span / subWords.length;
        let c = startTime;
        for (const sw of subWords) {
          words.push({ text: sw, start: c, end: c + subDur });
          c += subDur;
        }
      } else {
        words.push({ text: wordText, start: startTime, end: endTime });
      }
    }
  }

  return words.length > 0 ? words : null;
}

function interpolateWords(line: LyricLine, nextLineTime?: number): void {
  const wordsArr = line.text.split(/\s+/).filter(Boolean);
  if (wordsArr.length === 0) {
    line.words = [];
    return;
  }

  // Căn thời lượng hát tự nhiên theo khoảng cách câu
  let sungDur: number;
  if (nextLineTime !== undefined && nextLineTime > line.time) {
    const gap = nextLineTime - line.time;
    if (gap <= 2.2) {
      sungDur = gap * 0.92;
    } else if (gap <= 5.0) {
      sungDur = gap - 0.45;
    } else if (gap <= 8.5) {
      sungDur = Math.max(wordsArr.length * 0.65, gap - 0.9);
    } else {
      sungDur = Math.min(gap - 1.8, Math.max(2.4, wordsArr.length * 0.75));
    }
  } else {
    sungDur = Math.max(2.5, wordsArr.length * 0.7);
  }

  const weights = wordsArr.map((w, idx) => {
    const clean = w.replace(/[^\p{L}\p{N}]/gu, "");
    let wt = Math.max(1, 1 + 0.14 * Math.max(0, clean.length - 2));
    if (idx === wordsArr.length - 1) wt *= 1.35;
    else if (/[,;.!?~-]$/.test(w)) wt *= 1.25;
    return wt;
  });
  const totalWeight = weights.reduce((a, b) => a + b, 0);

  let cur = line.time;
  const words: LyricWord[] = [];
  for (let j = 0; j < wordsArr.length; j++) {
    const dur = (weights[j] / totalWeight) * sungDur;
    words.push({
      text: wordsArr[j],
      start: cur,
      end: cur + dur,
    });
    cur += dur;
  }

  line.words = words;
  line.endTime = cur;
}

export function getLrcOffset(raw: string): number {
  const match = [...raw.matchAll(/^\s*\[offset:\s*([+-]?\d+)\s*\]\s*$/gim)].at(-1);
  const value = match ? Number(match[1]) : 0;
  return Number.isSafeInteger(value) ? value : 0;
}

export function setLrcOffset(raw: string, milliseconds: number): string {
  if (!Number.isInteger(milliseconds) || Math.abs(milliseconds) > 10000) {
    throw new Error("Bù thời gian phải là số nguyên từ -10000 đến 10000 ms.");
  }
  const rows = raw.split(/\r?\n/).filter(row => !/^\s*\[offset:[^\]]*\]\s*$/i.test(row)).join("\n");
  return milliseconds ? `[offset:${milliseconds}]\n${rows}` : rows;
}

export function parseLrc(raw: string): ParsedLyrics {
  const synced: LyricLine[] = [];
  const plain: LyricLine[] = [];
  let foundWordSync = false;

  for (const row of raw.replace(/\r/g, "").split("\n")) {
    const trimmed = row.trim();
    if (!trimmed) continue;
    if (/^\[[a-z]+:.*\]$/i.test(trimmed)) continue;

    const stamps = [...trimmed.matchAll(TS)];
    if (stamps.length === 0) {
      plain.push({
        time: -1,
        endTime: -1,
        text: trimmed,
        words: [],
        hasWordSync: false,
      });
      continue;
    }

    const remainder = trimmed.replace(TS, "").trim();

    for (const m of stamps) {
      const lineTime = parseTimestamp(m[1], m[2]);
      const enhancedWords = parseEnhancedWords(remainder, lineTime);

      if (enhancedWords) {
        foundWordSync = true;
        const cleanText = enhancedWords.map((w) => w.text).join(" ");
        const lastWord = enhancedWords[enhancedWords.length - 1];
        synced.push({
          time: lineTime,
          endTime: lastWord ? lastWord.end : lineTime + 2,
          text: cleanText || remainder.replace(WORD_TS, "").trim(),
          words: enhancedWords,
          hasWordSync: true,
        });
      } else {
        synced.push({
          time: lineTime,
          endTime: lineTime + 2,
          text: remainder,
          words: [],
          hasWordSync: false,
        });
      }
    }
  }

  if (synced.length > 0) {
    // LRC convention: positive offset shows lyrics earlier. All views use these shifted times.
    const offset = getLrcOffset(raw) / 1000;
    for (const line of synced) {
      line.time = Math.max(0, line.time - offset);
      line.endTime = Math.max(line.time, line.endTime - offset);
      for (const word of line.words) {
        word.start = Math.max(0, word.start - offset);
        word.end = Math.max(word.start, word.end - offset);
      }
    }
    synced.sort((a, b) => a.time - b.time);

    for (let i = 0; i < synced.length; i++) {
      const line = synced[i];
      if (!line.hasWordSync || line.words.length === 0) {
        const nextTime = synced[i + 1]?.time;
        interpolateWords(line, nextTime);
      }
    }

    return { synced: true, hasWordSync: foundWordSync, lines: synced };
  }

  return { synced: false, hasWordSync: false, lines: plain };
}

// Replace spelling by position, never by word value: repeated words keep separate timing slots.
export function correctLyricsText(original: string, corrected: string): string {
  const rows = original.replace(/\r/g, "").split("\n");
  const content = (row: string) => row.replace(TS, " ").replace(WORD_TS, " ").trim();
  const editable = rows.map((row, index) => ({ row, index }))
    .filter(({ row }) => !/^\s*\[[a-z]+:.*\]\s*$/i.test(row) && content(row));
  const replacements = corrected.split(/\r?\n/).map(row => row.trim()).filter(Boolean);
  if (replacements.length !== editable.length) {
    throw new Error("Giữ nguyên số dòng để không lệch mốc thời gian.");
  }
  for (const [i, { row, index }] of editable.entries()) {
    if (/[<>\[\]]/.test(replacements[i]) || [...replacements[i].matchAll(WORD_TS)].length) {
      throw new Error("Chỉ nhập lời, không nhập timestamp trong chế độ sửa chính tả.");
    }
    const words = replacements[i].split(/\s+/);
    if (words.length !== content(row).split(/\s+/).length) {
      throw new Error(`Dòng ${i + 1}: giữ nguyên số chữ. Dùng chế độ LRC nếu cần thêm hoặc bớt chữ.`);
    }
    let next = 0;
    // Timestamp tags stay byte-for-byte unchanged, including explicit ends and pauses.
    rows[index] = row.split(/(\[\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?\]|<\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?>|\(\d{1,3}:\d{1,2}(?:[.:]\d{1,3})?\))/g)
      .map((part, index) => index % 2 ? part : part.replace(/\S+/g, () => words[next++])).join("");
    if (next !== words.length) {
      throw new Error(`Dòng ${i + 1}: định dạng không hỗ trợ sửa nhanh. Hãy sửa trong chế độ LRC.`);
    }
  }
  return rows.join("\n");
}

export function alignLyricsToTranscript(reference: string, transcript: string, duration: number): string {
  const source = parseLrc(transcript);
  const targetLines = reference.split(/\r?\n/).flatMap(row => parseLrc(row).lines);
  const normalize = (word: string) => word.normalize("NFD").replace(/\p{M}/gu, "").replace(/[đĐ]/g, "d").toLowerCase().replace(/[^\p{L}\p{N}]/gu, "");
  const words = source.lines.flatMap(line => line.words);
  const wanted = targetLines.flatMap(line => line.text.split(/\s+/).filter(Boolean));
  if (!source.hasWordSync || source.lines.some(line => !line.hasWordSync) || !wanted.length || !words.length) {
    throw new Error("Cần kết quả nhận diện có mốc từng chữ. Chọn Whisper offline, Groq hoặc OpenAI rồi thử lại.");
  }
  if (wanted.length > 2000 || words.length > 2400 || !Number.isFinite(duration) || duration <= 0) {
    throw new Error("Lời quá dài hoặc chưa biết thời lượng để căn an toàn.");
  }
  if (words.some((word, i) => word.start < 0 || word.end <= word.start || word.end > duration + 0.1 || (i > 0 && word.start < words[i - 1].end - 0.02))) {
    throw new Error("Mốc giọng hát bị chồng hoặc vượt thời lượng. Bản cũ được giữ nguyên.");
  }
  const tokens = words.map(word => normalize(word.text));
  const expected = wanted.map(normalize);
  if (expected.some(word => !word)) throw new Error("Lời có dòng/ký hiệu không phải lời hát. Bỏ ký hiệu riêng rồi thử lại.");
  const earliest: number[] = [];
  let at = 0;
  for (const word of expected) {
    while (at < tokens.length && tokens[at] !== word) at++;
    if (at === tokens.length) throw new Error("Lời và giọng hát nhận diện chưa khớp đủ chữ. Kiểm tra đúng bản thu hoặc sửa lời rồi thử lại; chưa đổi mốc cũ.");
    earliest.push(at++);
  }
  at = tokens.length - 1;
  for (let i = expected.length - 1; i >= 0; i--) {
    while (at >= 0 && tokens[at] !== expected[i]) at--;
    if (at !== earliest[i]) throw new Error("Có chữ lặp chưa xác định được lần hát. Không tự gán mốc; hãy kiểm tra bản nhận diện.");
    at--;
  }
  if (expected.length / tokens.length < 0.85) throw new Error("Lời có sẵn thiếu nhiều chữ so với bản nhận diện. Bổ sung lời trước khi căn.");
  const stamp = (time: number) => {
    const cs = Math.round(time * 100);
    return `${String(Math.floor(cs / 6000)).padStart(2, "0")}:${String(Math.floor(cs / 100) % 60).padStart(2, "0")}.${String(cs % 100).padStart(2, "0")}`;
  };
  let index = 0;
  const rows = targetLines.filter(line => line.text.trim()).map(line => {
    const lineWords = line.text.split(/\s+/).filter(Boolean);
    const matched = lineWords.map(text => ({ ...words[earliest[index++]], text }));
    return `[${stamp(matched[0].start)}]` + matched.map((word, i) =>
      `<${stamp(word.start)}>${word.text}<${stamp(word.end)}>${i < matched.length - 1 ? " " : ""}`).join("");
  });
  const metadata = reference.split(/\r?\n/).filter(row => /^\[(?:ti|ar|al|by):[^\]]*\]$/i.test(row.trim()));
  return [...metadata, ...rows].join("\n");
}

export function checkLyricsTiming(content: string, duration: number): string[] {
  const parsed = parseLrc(content);
  if (!parsed.lines.length) return [];
  if (!parsed.synced) return ["Chưa có mốc thời gian. Dùng Căn theo giọng hát để đối chiếu âm thanh."];
  const issues: string[] = [];
  const stamps = [...content.matchAll(TS), ...content.matchAll(WORD_TS)];
  if (stamps.some(m => Number((m[2] || m[4]).replace(":", ".")) >= 60)) issues.push("Phần giây của timestamp phải nhỏ hơn 60.");
  const rawTimes = [...content.matchAll(TS)].map(m => parseTimestamp(m[1], m[2]));
  if (rawTimes.some((time, i) => i > 0 && time < rawTimes[i - 1])) issues.push("Mốc câu không theo thứ tự. Kiểm tra lại bản LRC.");
  if (parsed.lines.some(line => line.time > duration && duration > 0 || line.words.some(word => word.end <= word.start))) issues.push("Có mốc vượt thời lượng hoặc chữ không có thời lượng.");
  if (parsed.lines.some(line => !line.hasWordSync)) issues.push("Có câu chỉ có mốc câu. Chạy từng chữ ở câu đó là ước lượng.");
  return issues;
}

export function activeIndex(lines: LyricLine[], pos: number): number {
  let lo = 0;
  let hi = lines.length - 1;
  let ans = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].time <= pos) {
      ans = mid;
      lo = mid + 1;
    } else hi = mid - 1;
  }
  return ans;
}

export function activeWordIndex(words: LyricWord[], pos: number): number {
  if (words.length === 0) return -1;
  if (pos < words[0].start) return -1;
  if (pos >= words[words.length - 1].end) return words.length;

  for (let i = 0; i < words.length; i++) {
    if (pos >= words[i].start && pos < words[i].end) return i;
    if (i + 1 < words.length && pos >= words[i].end && pos < words[i + 1].start) {
      return i;
    }
  }
  return words.length - 1;
}

export function wordFillPercent(word: LyricWord, pos: number): number {
  if (pos <= word.start) return 0;
  if (pos >= word.end) return 100;
  const dur = word.end - word.start;
  if (dur <= 0) return 100;
  return Math.min(100, Math.max(0, ((pos - word.start) / dur) * 100));
}

export function autoGenerateTimedLyrics(
  plainText: string,
  duration = 180,
  title = "",
  artist = "",
): string {
  const rawLines = plainText
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !/^\[[a-z]+:.*\]$/i.test(l));

  if (rawLines.length === 0 || !Number.isFinite(duration) || duration <= 0) return "";

  const dur = duration;
  const intro = Math.min(18, dur * 0.06);
  const outro = Math.min(20, dur * 0.05);
  const vocalSpan = dur - intro - outro;

  const weights = rawLines.map((line) => {
    const words = line.split(/\s+/).filter(Boolean);
    const clean = line.replace(/[^\p{L}\p{N}]/gu, "");
    return Math.max(1, words.length * 1.5 + clean.length * 0.1);
  });
  const totalWeight = weights.reduce((a, b) => a + b, 0);

  const out: string[] = [];
  if (title) out.push(`[ti:${title}]`);
  if (artist) out.push(`[ar:${artist}]`);

  let curTime = intro;
  for (let i = 0; i < rawLines.length; i++) {
    const cs = Math.round(curTime * 100);
    const mm = String(Math.floor(cs / 6000)).padStart(2, "0");
    const ss = `${String(Math.floor(cs / 100) % 60).padStart(2, "0")}.${String(cs % 100).padStart(2, "0")}`;
    out.push(`[${mm}:${ss}] ${rawLines[i]}`);

    const span = (weights[i] / totalWeight) * vocalSpan;
    curTime += span;
  }

  return out.join("\n");
}

