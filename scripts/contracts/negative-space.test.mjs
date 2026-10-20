import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { validateFixture } from './validate.mjs';

test('rules3 preserves explicit scale lock while allowing its versioned validity policy', async () => {
  const request = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/export-svg.json',
        import.meta.url,
      ),
    ),
  );
  request.generation.rulesVersion = '3';
  request.generation.rules.preserveAspectRatio = true;
  request.generation.rules.scaleY = { ...request.generation.rules.scaleX };
  assert.equal(validateFixture(request, 'generation-request', 'export'), true);
  delete request.generation.rules.preserveAspectRatio;
  assert.equal(validateFixture(request, 'generation-request', 'export'), false);
});
