import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { THEME_FAMILIES, getPalette } from '../../src/lib/palettes.ts';

const dir = dirname(fileURLToPath(import.meta.url));
const colors = { ink: '#101010', paper: '#F5F0E8', apricot: '#FFC799', mint: '#99FFE4', slate: '#A0A0A0' };
const mark = '<path d="M26 78V31C26 23 32 17 40 17H51C68 17 78 27 78 41C78 55 68 65 51 65H44M55 65L78 82"/>';
const letters = '<path d="M0 40V0H14C24 0 28 5 28 12S24 24 14 24H0M15 24L30 40M56 0C46 0 42 8 42 20S46 40 56 40S70 32 70 20S66 0 56 0Z M84 0L98 40L112 0M126 0V26C126 35 131 40 140 40S154 35 154 26V0M170 40V0L198 40V0M244 0H216V40H244M216 20H238"/>';
const strokes = (content, color, width) => `<g fill="none" stroke="${color}" stroke-width="${width}" stroke-linecap="round" stroke-linejoin="round">${content}</g>`;
const svg = (title, viewBox, body, extra = '') => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" role="img" aria-label="${title}"><title>${title}</title>${extra}${body}</svg>\n`;
const save = (name, body) => writeFileSync(join(dir, name), body);
const icon = `<rect width="128" height="128" rx="29" fill="${colors.ink}"/><rect x=".5" y=".5" width="127" height="127" rx="28.5" fill="none" stroke="#303030"/><g transform="translate(14 14)">${strokes(mark, colors.apricot, 10)}</g>`;

save('logo.svg', svg('Rovune Open Loop mark', '0 0 100 100', strokes(mark, colors.ink, 10)));
save('logo-inverse.svg', svg('Rovune Open Loop mark, inverse', '0 0 100 100', strokes(mark, colors.paper, 10)));
save('wordmark.svg', svg('ROVUNE custom wordmark', '-5 -5 254 50', strokes(letters, colors.ink, 5)));
save('wordmark-inverse.svg', svg('ROVUNE custom wordmark, inverse', '-5 -5 254 50', strokes(letters, colors.paper, 5)));
for (const [name, color] of [['lockup.svg', colors.ink], ['lockup-inverse.svg', colors.paper]]) {
  save(name, svg('Rovune horizontal signature', '0 0 440 100', `${strokes(mark, color, 10)}<g transform="translate(128 30) scale(1.2)">${strokes(letters, color, 5)}</g>`));
}
save('app-icon.svg', svg('Rovune desktop app icon', '0 0 128 128', icon));
save('tokens.json', JSON.stringify({ name: 'Rovune', pronunciation: 'rô-vun', tagline: 'Giữ nhịp riêng.', colors, typography: { ui: 'Be Vietnam Pro', data: 'JetBrains Mono', wordmark: 'Custom vector lettering' } }, null, 2) + '\n');

mkdirSync(join(dir, 'fonts'), { recursive: true });
let faces = '';
for (const [pack, family, subset, weight] of [
  ['be-vietnam-pro', 'Rovune Sans', 'latin', 400],
  ['be-vietnam-pro', 'Rovune Sans', 'vietnamese', 400],
  ['be-vietnam-pro', 'Rovune Sans', 'latin', 600],
  ['be-vietnam-pro', 'Rovune Sans', 'vietnamese', 600],
  ['jetbrains-mono', 'Rovune Mono', 'latin', 400],
]) {
  const base = join(dir, '../../node_modules/@fontsource', pack);
  const name = `${pack}-${subset}-${weight}-normal.woff2`;
  copyFileSync(join(base, 'files', name), join(dir, 'fonts', name));
  const css = readFileSync(join(base, `${weight}.css`), 'utf8');
  const face = css.split('@font-face').find((block) => block.includes(name));
  const range = face.match(/unicode-range:\s*([^;]+);/)[1];
  const data = readFileSync(join(dir, 'fonts', name)).toString('base64');
  faces += `@font-face{font-family:'${family}';font-style:normal;font-weight:${weight};src:url(data:font/woff2;base64,${data}) format('woff2');unicode-range:${range}}`;
}
for (const pack of ['be-vietnam-pro', 'jetbrains-mono']) {
  copyFileSync(join(dir, '../../node_modules/@fontsource', pack, 'LICENSE'), join(dir, 'fonts', `${pack}-LICENSE.txt`));
}
const styles = `<style>${faces}text{font-family:'Rovune Sans',sans-serif}.mono{font-family:'Rovune Mono',monospace}.secondary{fill:${colors.slate}}</style>`;
const art = readFileSync(join(dir, 'brand-art.png')).toString('base64');
const t = (x, y, size, text, color = colors.paper, extra = '') => `<text x="${x}" y="${y}" font-size="${size}" fill="${color}" ${extra}>${text}</text>`;
let board = `<rect width="1600" height="1320" fill="${colors.ink}"/><image x="600" y="0" width="1000" height="590" preserveAspectRatio="xMidYMid slice" href="data:image/png;base64,${art}"/>`;
board += `<g transform="translate(54 46) scale(.64)">${strokes(mark, colors.apricot, 10)}</g>`;
board += t(132, 87, 16, 'ROVUNE / DESKTOP MUSIC PLAYER', colors.paper, 'letter-spacing="2"');
board += `<g transform="translate(70 210) scale(3.05)">${strokes(letters, colors.apricot, 5)}</g>`;
board += t(64, 405, 46, 'Giữ nhịp riêng.');
board += t(67, 457, 20, 'Thư viện riêng. Giao diện theo gu bạn.', colors.slate);
board += t(67, 550, 14, 'OPEN LOOP / BRAND SYSTEM / CONCEPT v1', colors.slate, 'letter-spacing="2"');
board += `<rect y="590" width="1600" height="218" fill="${colors.paper}"/>`;
board += t(64, 635, 18, 'Màu thương hiệu', colors.ink, 'font-weight="600"');
const swatches = [['Ink', colors.ink], ['Paper', colors.paper], ['Apricot', colors.apricot], ['Mint', colors.mint], ['Slate', colors.slate]];
for (const [i, [label, color]] of swatches.entries()) {
  const x = 64 + i * 298;
  board += `<rect x="${x}" y="657" width="280" height="73" rx="2" fill="${color}" stroke="#CFCAC3"/>`;
  board += t(x, 766, 16, label, colors.ink, 'font-weight="600"');
  board += t(x + 145, 766, 14, color, colors.ink, 'class="mono"');
}
board += `<path d="M800 846V1128M64 1160H1536" stroke="#343434"/>`;
board += t(64, 869, 16, 'Kiểu chữ', colors.slate);
board += t(64, 925, 38, 'Be Vietnam Pro', colors.paper, 'font-weight="600"');
board += t(64, 980, 32, 'Aa Ăă Ââ Êê Ôô Ơơ Ưư Đđ');
board += t(64, 1044, 27, '03:45 / FLAC / 48 kHz', colors.apricot, 'class="mono"');
board += t(64, 1092, 16, 'Giao diện: Be Vietnam Pro · Dữ liệu: JetBrains Mono', colors.slate);
board += t(860, 869, 16, 'Một dấu hiệu, mọi ngữ cảnh', colors.slate);
board += `<rect x="860" y="897" width="154" height="154" rx="12" fill="${colors.paper}"/><g transform="translate(887 924)">${strokes(mark, colors.ink, 10)}</g>`;
board += `<rect x="1044" y="897" width="154" height="154" rx="12" fill="#242424"/><g transform="translate(1071 924)">${strokes(mark, colors.paper, 10)}</g>`;
board += `<g transform="translate(1228 897) scale(1.203125)">${icon}</g>`;
board += t(860, 1092, 15, 'Logo sáng / Logo đảo màu / App icon', colors.slate);
board += t(64, 1221, 30, 'Logo một màu. Cá tính không đổi.');
board += t(64, 1264, 17, 'Đổi sắc theo theme. Giữ nguyên hình, tỷ lệ và khoảng trống.', colors.slate);
for (const [i, family] of THEME_FAMILIES.entries()) {
  const p = getPalette(family.default);
  board += `<g transform="translate(${966 + i * 54} 1204) scale(.38)">${strokes(mark, p.colors.accent, 10)}</g>`;
}
board += t(968, 1285, 14, '10 bộ theme / 31 biến thể', colors.slate);
save('brandboard.svg', svg('Rovune brand identity board', '0 0 1600 1320', board, styles));

let sheet = `<rect width="1600" height="800" fill="${colors.paper}"/>${t(64, 70, 28, 'Rovune / Theme compatibility', colors.ink, 'font-weight="600"')}`;
for (const [i, family] of THEME_FAMILIES.entries()) {
  const p = getPalette(family.default);
  const x = 64 + (i % 5) * 298, y = 110 + Math.floor(i / 5) * 310;
  sheet += `<rect x="${x}" y="${y}" width="278" height="268" rx="16" fill="${p.colors.base}"/><g transform="translate(${x + 89} ${y + 35})">${strokes(mark, p.colors.accent, 10)}</g>`;
  sheet += t(x + 22, y + 195, 19, family.name, p.colors.text, 'font-weight="600"');
  sheet += t(x + 22, y + 229, 15, p.name, p.colors.text);
}
sheet += t(64, 770, 16, 'Màu theo palette. Hình logo luôn giữ nguyên. Đây là mẫu nhận diện, chưa thay icon trong app.', colors.ink);
save('theme-compatibility.svg', svg('Rovune across ten theme families', '0 0 1600 800', sheet, styles));
console.log('Built Rovune vector assets, embedded-font brandboards, tokens and font licenses.');
