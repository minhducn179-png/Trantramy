import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';

for (const [script, ...args] of [
  ['track-info'], ['lyrics-ui'], ['layout'], ['layout', 'karaoke'],
]) {
  const result = spawnSync(process.execPath, [`tests/${script}.mjs`, ...args], {
    stdio: 'inherit',
    env: { ...process.env, DLC_UI_PREVIEW: '1' },
    timeout: 15 * 60 * 1000,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, `${script} ${args.join(' ')} failed`);
}
