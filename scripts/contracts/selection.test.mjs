import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { validateFixture } from './validate.mjs';

test('v2 selection accepts gaps and exact u64; rejects unsafe or inconsistent lists', async () => {
  const fixture = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  const value = {
    ...fixture,
    schemaVersion: '2',
    variantIndices: ['0', '2', '18446744073709551615'],
  };
  const cases = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v2/selection-cases.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  for (const { indices, valid } of cases) {
    assert.equal(
      validateFixture(
        { ...value, variantIndices: indices },
        'selected-pdf-export-request',
        'export-pdf-selection',
      ),
      valid,
      indices.join(','),
    );
  }
});
