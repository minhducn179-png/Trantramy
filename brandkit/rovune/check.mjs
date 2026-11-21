import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { contrastRatio, THEME_FAMILIES, getPalette } from '../../src/lib/palettes.ts';

const dir = dirname(fileURLToPath(import.meta.url));
const tokens = JSON.parse(readFileSync(join(dir, 'tokens.json'), 'utf8'));
assert.equal(tokens.name, 'Rovune');
assert.equal(tokens.tagline, 'Giữ nhịp riêng.');
for (const color of Object.values(tokens.colors)) assert.match(color, /^#[0-9A-F]{6}$/);
for (const bg of ['paper', 'apricot', 'mint']) {
  assert.ok(contrastRatio(tokens.colors.ink, tokens.colors[bg]) >= 4.5, `Ink on ${bg}`);
}
assert.ok(contrastRatio(tokens.colors.slate, tokens.colors.ink) >= 4.5);
const vectors = readdirSync(dir).filter((name) => name.endsWith('.svg'));
assert.equal(vectors.length, 9);
execFileSync('python', ['-c', 'import sys, xml.etree.ElementTree as ET; [ET.parse(p) for p in sys.argv[1:]]', ...vectors.map((name) => join(dir, name))]);
for (const name of vectors) {
  const text = readFileSync(join(dir, name), 'utf8');
  assert.ok(!/<script\b|href="https?:/i.test(text), `${name}: no scripts or external assets`);
  if (!['brandboard.svg', 'theme-compatibility.svg'].includes(name)) {
    assert.ok(!/<text\b|<image\b|font-face/i.test(text), `${name}: logo must be font-independent vector`);
  }
}
for (const [name, width, height] of [
  ['brandboard.png', 1600, 1320], ['theme-compatibility.png', 1600, 800],
  ['app-icon-512.png', 512, 512], ['app-icon-32.png', 32, 32],
]) {
  const data = readFileSync(join(dir, name));
  assert.equal(data.subarray(1, 4).toString(), 'PNG');
  assert.equal(data.readUInt32BE(16), width, `${name}: width`);
  assert.equal(data.readUInt32BE(20), height, `${name}: height`);
  if (name.startsWith('app-icon')) assert.equal(data[25], 6, 'PNG icon needs an alpha channel');
}
const fonts = readdirSync(join(dir, 'fonts')).filter((name) => name.endsWith('.woff2'));
assert.equal(fonts.length, 5);
for (const name of fonts) assert.equal(readFileSync(join(dir, 'fonts', name)).subarray(0, 4).toString(), 'wOF2');
for (const pack of ['be-vietnam-pro', 'jetbrains-mono']) {
  assert.match(readFileSync(join(dir, 'fonts', `${pack}-LICENSE.txt`), 'utf8'), /OPEN FONT LICENSE/i);
}
const sheet = readFileSync(join(dir, 'theme-compatibility.svg'), 'utf8');
for (const f of THEME_FAMILIES) {
  assert.ok(sheet.includes(f.name));
  const p = getPalette(f.default);
  assert.ok(contrastRatio(p.colors.accent, p.colors.base) >= 3, `${f.name}: mark contrast`);
}
const provenance = JSON.parse(readFileSync(join(dir, 'artwork.provenance.json'), 'utf8'));
assert.ok(provenance.prompt.startsWith('Create a finished abstract brand key visual for ROVUNE'));
assert.ok(statSync(join(dir, provenance.file)).size > 0);
assert.ok(readFileSync(join(dir, 'README.md'), 'utf8').includes('Chưa đổi tên DLC'));
console.log('PASS: vector XML, independent logos, PNG dimensions/alpha, fonts/licenses, brand contrast, ten theme mappings and artwork provenance');
