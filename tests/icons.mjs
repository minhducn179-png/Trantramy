import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

const read = file => readFileSync(file, 'utf8');
function checkComponents(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const file = join(dir, entry.name);
    if (entry.isDirectory()) checkComponents(file);
    else if (file.endsWith('.svelte')) {
      const source = read(file);
      const markup = source.replace(/<script\b[^>]*>[\s\S]*?<\/script>|<style\b[^>]*>[\s\S]*?<\/style>|<!--[\s\S]*?-->/g, '');
      assert(!/[✨💧🌊📌♥♪≋◖❖▧◎⧉✕⏮⏭⏸▶✓]/u.test(markup), `${file}: legacy icon glyph`);
      if (!['AppSpeakerLogo.svelte', 'WaveformBar.svelte'].includes(entry.name)) {
        assert(!/<svg\b(?![^>]*\bdata-ui-artwork=)/.test(markup), `${file}: use a Lucide component, not a hand-drawn icon`);
      }
    }
  }
}

// Native rasterization reads path data from the rendered Lucide Volume2.
// Fail if a package upgrade changes it to a shape our canvas cannot draw.
function iconData(name) {
  const source = read(`node_modules/@lucide/svelte/dist/icons/${name}.svelte`);
  return JSON.parse(source.match(/export const __iconData = (.*);/)[1]);
}
const volume = iconData('volume-2');
assert(volume.node.length > 0);
assert(volume.node.every(([tag, attrs]) => tag === 'path' && typeof attrs.d === 'string'));
const appIcon = read('static/app-icon.svg');
const staticPaths = [...appIcon.matchAll(/<path d="([^"]+)"/g)].map(match => match[1]);
assert.deepEqual(staticPaths, volume.node.map(([, attrs]) => attrs.d));
for (const [, attrs] of iconData('audio-lines').node) {
  assert(read('src/lib/api.ts').includes(`d="${attrs.d}"`), 'Cover artwork must use Lucide audio-lines');
}
assert.deepEqual(readFileSync('static/favicon.png'), readFileSync('src-tauri/icons/32x32.png'));
assert(read('static/lucide-LICENSE.txt').includes('ISC License'));
checkComponents('src');
console.log('PASS: Lucide UI coverage, native paths, cover icon, favicon and license');
