import assert from "node:assert/strict";
import { parseLrc, activeIndex, activeWordIndex, wordFillPercent, autoGenerateTimedLyrics, correctLyricsText, getLrcOffset, setLrcOffset, alignLyricsToTranscript, checkLyricsTiming } from "../src/lib/lyrics/lrc.ts";

const enhancedSample = `
[ti:Test Song]
[ar:DLC Artist]
[00:10.00]<00:10.00>Xin <00:10.50>chào <00:11.00>Việt <00:11.50>Nam <00:12.20>
[00:15.00]Tôi yêu âm nhạc (guitar solo)
[00:20.00](Điệp khúc) (00:20.50) Lời (00:21.00) hát
`;

const resEnhanced = parseLrc(enhancedSample);
assert.equal(resEnhanced.synced, true);
assert.equal(resEnhanced.hasWordSync, true);
assert.equal(resEnhanced.lines.length, 3);

const line1 = resEnhanced.lines[0];
assert.equal(line1.text, "Xin chào Việt Nam");
assert.equal(line1.words.length, 4);
assert.equal(line1.words[0].text, "Xin");
assert.equal(line1.words[0].start, 10);
assert.equal(line1.words[0].end, 10.5);
assert.equal(line1.words[3].text, "Nam");
assert.equal(line1.words[3].start, 11.5);
assert.equal(line1.words[3].end, 12.2);

const line2 = resEnhanced.lines[1];
assert.ok(line2.text.includes("(guitar solo)"));
assert.equal(line2.words.length, 6);
assert.equal(line2.words[0].text, "Tôi");
assert.equal(line2.words[0].start, 15);
assert.ok(line2.words[5].end > 15);

const line3 = resEnhanced.lines[2];
assert.ok(line3.text.includes("(Điệp khúc)"));
assert.ok(line3.words.some((w) => w.text.includes("(Điệp khúc)")));
assert.ok(line3.words.some((w) => w.text === "Lời"));

assert.equal(wordFillPercent(line1.words[0], 9.9), 0);
assert.equal(wordFillPercent(line1.words[0], 10.25), 50);
assert.equal(wordFillPercent(line1.words[0], 10.5), 100);
assert.equal(wordFillPercent(line1.words[0], 11), 100);

assert.equal(activeWordIndex(line1.words, 9.9), -1);
assert.equal(activeWordIndex(line1.words, 10.2), 0);
assert.equal(activeWordIndex(line1.words, 10.7), 1);
assert.equal(activeWordIndex(line1.words, 11.2), 2);
assert.equal(activeWordIndex(line1.words, 11.8), 3);
assert.equal(activeWordIndex(line1.words, 13), 4);

assert.equal(activeIndex(resEnhanced.lines, 5), -1);
assert.equal(activeIndex(resEnhanced.lines, 10), 0);
assert.equal(activeIndex(resEnhanced.lines, 14.9), 0);
assert.equal(activeIndex(resEnhanced.lines, 15), 1);
assert.equal(activeIndex(resEnhanced.lines, 30), 2);

const plainSample = "Just unsynced lyric line 1\nLine 2";
const resPlain = parseLrc(plainSample);
assert.equal(resPlain.synced, false);
assert.equal(resPlain.hasWordSync, false);
assert.equal(resPlain.lines.length, 2);

const generated = autoGenerateTimedLyrics("Line 1\nLine 2\nLine 3", 120, "Song", "Singer");
assert.ok(generated.includes("[ti:Song]"));
assert.ok(generated.includes("[ar:Singer]"));
assert.ok(generated.includes("[00:"));
const parsedGen = parseLrc(generated);
assert.equal(parsedGen.synced, true);
assert.equal(parsedGen.lines.length, 3);
assert.ok(parsedGen.lines[0].time >= 4);
assert.ok(parsedGen.lines[2].time > parsedGen.lines[0].time);

assert.equal(autoGenerateTimedLyrics("", 120), "");
assert.equal(autoGenerateTimedLyrics("Real words", Number.NaN), "");
assert.equal(autoGenerateTimedLyrics("Real words", -1), "");
const short = parseLrc(autoGenerateTimedLyrics("One\nTwo", 3));
assert.ok(short.lines.every(line => line.time >= 0 && line.time < 3));

const repeated = "[ti:Bài hát]\n[00:01.00]<00:01.00>La <00:01.10>la <00:01.30><00:02.00>la <00:04.00>";
const corrected = correctLyricsText(repeated, "Là lá lạ!");
assert.equal(corrected, "[ti:Bài hát]\n[00:01.00]<00:01.00>Là <00:01.10>lá <00:01.30><00:02.00>lạ! <00:04.00>");
const separate = parseLrc(corrected).lines[0].words;
assert.deepEqual(separate.map(({ start, end }) => [start, end]), [[1, 1.1], [1.1, 1.3], [2, 4]]);
assert.equal(wordFillPercent(separate[0], 1.05).toFixed(0), "50");
assert.equal(wordFillPercent(separate[2], 3), 50);
assert.equal(wordFillPercent(separate[2], 1.5), 0);
assert.throws(() => correctLyricsText(repeated, "Là lá lạ thêm"), /Dòng 1/);
assert.throws(() => correctLyricsText(repeated, "Là lá\nlạ"), /số dòng/);
assert.throws(() => correctLyricsText(repeated, "<00:01.00>Là lá lạ"), /không nhập timestamp/);
assert.equal(correctLyricsText("[00:01.00](Điệp khúc) tôi yeu", "(Điệp khúc) tôi yêu"), "[00:01.00](Điệp khúc) tôi yêu");
assert.equal(correctLyricsText("[00:01.00][00:05.00]Xin chao", "Xin chào"), "[00:01.00][00:05.00]Xin chào");
assert.equal(correctLyricsText("[00:01.00]<00:01.00>xin<00:01.10>chao<00:01.20>", "Xin chào"), "[00:01.00]<00:01.00>Xin<00:01.10>chào<00:01.20>");
const rapid = parseLrc("[00:01.00]<00:01.00>la la <00:01.10>").lines[0];
assert.equal(rapid.words[1].end, 1.1);
assert.ok(rapid.words[0].end <= rapid.words[1].start);

const earlierRaw = setLrcOffset(repeated, 500);
assert.equal(getLrcOffset(earlierRaw), 500);
const earlier = parseLrc(earlierRaw).lines[0];
assert.equal(earlier.time, 0.5);
assert.deepEqual(earlier.words.map(({ start, end }) => [start, end]), [[0.5, 0.6000000000000001], [0.6000000000000001, 0.8], [1.5, 3.5]]);
assert.equal(wordFillPercent(earlier.words[2], 2.5), 50);
assert.equal(parseLrc(setLrcOffset(repeated, -500)).lines[0].time, 1.5);
assert.equal(parseLrc(setLrcOffset(repeated, 2000)).lines[0].words[0].start, 0);
assert.equal(parseLrc(setLrcOffset(repeated, 2000)).lines[0].words[0].end, 0);
assert.equal(setLrcOffset(earlierRaw, 0), repeated);
assert.equal((setLrcOffset(earlierRaw, -50).match(/\[offset:/g) ?? []).length, 1);
assert.equal(getLrcOffset("[offset:100]\n[offset:-250]"), -250);
assert.equal(getLrcOffset("[offset:bad]"), 0);
assert.equal(getLrcOffset("[offset:999999999999999999999]"), 0);
assert.throws(() => setLrcOffset(repeated, Number.NaN), /số nguyên/);
assert.throws(() => setLrcOffset(repeated, 10001), /số nguyên/);
assert.ok(correctLyricsText(earlierRaw, "Là lá lạ").includes("[offset:500]"));
assert.equal(parseLrc("[offset:500]\nPlain words").synced, false);
const offsetLines = parseLrc("[offset:500]\n[00:01.00]Xin chào\n[00:02.00]Việt Nam");
assert.equal(offsetLines.lines[0].time, 0.5);
assert.ok(offsetLines.lines[0].words.at(-1).end <= 1.5);

const voice = "[00:01.00]<00:01.00>Toi<00:01.10> <00:01.30>yeu<00:02.00> <00:03.00>am<00:03.20> <00:03.20>nhac<00:04.00>";
const aligned = alignLyricsToTranscript("[offset:500]\n[ti:Test]\nTôi yêu\nÂm nhạc!", voice, 5);
assert.equal(aligned, "[ti:Test]\n[00:01.00]<00:01.00>Tôi<00:01.10> <00:01.30>yêu<00:02.00>\n[00:03.00]<00:03.00>Âm<00:03.20> <00:03.20>nhạc!<00:04.00>");
assert.deepEqual(parseLrc(aligned).lines.flatMap(l => l.words).map(w => [w.start, w.end]), [[1, 1.1], [1.3, 2], [3, 3.2], [3.2, 4]]);
assert.throws(() => alignLyricsToTranscript("Tôi thích âm nhạc", voice, 5), /chưa khớp/);
assert.throws(() => alignLyricsToTranscript("Tôi yêu", "[00:01.00]Tôi yêu", 5), /mốc từng chữ/);
assert.throws(() => alignLyricsToTranscript("Tôi yêu âm nhạc", voice, 2), /vượt thời lượng/);
assert.throws(() => alignLyricsToTranscript("Tôi yêu âm nhạc", voice, NaN), /thời lượng/);
assert.throws(() => alignLyricsToTranscript("Tôi yêu", "[00:01.00]<00:01.00>Tôi<00:01.10> <00:01.10>yêu<00:01.20> <00:01.20>yêu<00:01.30>", 5), /chữ lặp/);
assert.throws(() => alignLyricsToTranscript("Tôi", voice, 5), /thiếu nhiều chữ/);
assert.equal(checkLyricsTiming(aligned, 5).length, 0);
assert.equal(alignLyricsToTranscript("[00:08.00]Tôi yêu\n[00:00.00]Âm nhạc!", voice, 5), aligned.replace("[ti:Test]\n", ""));
assert.equal(alignLyricsToTranscript("Tôi yêu\n[00:00.00]Âm nhạc!", voice, 5), aligned.replace("[ti:Test]\n", ""));
assert.ok(checkLyricsTiming("[00:62.00]Sai mốc", 80).some(s => s.includes("nhỏ hơn 60")));
assert.match(checkLyricsTiming("Xin chào", 5)[0], /Chưa có mốc/);
assert.ok(checkLyricsTiming("[00:06.00]Sau\n[00:02.00]Trước", 5).some(s => s.includes("thứ tự")));
assert.ok(checkLyricsTiming("[00:06.00]Sau", 5).some(s => s.includes("vượt")));
console.log("PASS: Lyrics timing, repetitions, spelling corrections, offsets and conservative audio alignment");
