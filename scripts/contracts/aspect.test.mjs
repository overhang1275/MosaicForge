import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { validateFixture } from './validate.mjs';

test('aspect locking opts into rules2 and rejects ambiguous or incompatible settings', async () => {
  const input = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/export-svg.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  const generation = input.generation;
  assert.equal(validateFixture(input, 'generation-request', 'export'), true);
  generation.rulesVersion = '2';
  generation.rules.preserveAspectRatio = true;
  generation.rules.scaleY = { ...generation.rules.scaleX };
  assert.equal(validateFixture(input, 'generation-request', 'export'), true);
  const altered = (mutate) => {
    const copy = structuredClone(input);
    mutate(copy.generation);
    return validateFixture(copy, 'generation-request', 'export');
  };
  assert.equal(
    altered((g) => {
      g.rulesVersion = '1';
    }),
    false,
  );
  assert.equal(
    altered((g) => {
      delete g.rules.preserveAspectRatio;
    }),
    false,
  );
  assert.equal(
    altered((g) => {
      g.rules.scaleY.max += 1;
    }),
    false,
  );
  assert.equal(
    altered((g) => {
      g.rules.preserveAspectRatio = false;
      g.rules.scaleY.max += 1;
    }),
    true,
  );
});
