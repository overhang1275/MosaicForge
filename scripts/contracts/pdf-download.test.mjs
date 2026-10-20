import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { validateFixture } from './validate.mjs';

test('binary PDF metadata validates complete ordered selection without base64', async () => {
  const value = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/accepted-pdf.json',
        import.meta.url,
      ),
    ),
  );
  delete value.artifact;
  value.schemaVersion = '3';
  value.variantIndices = ['0', '1', '2'];
  assert.equal(
    validateFixture(value, 'pdf-download-response', 'export-pdf-download'),
    true,
  );
  value.document.pageCount = 2;
  assert.equal(
    validateFixture(value, 'pdf-download-response', 'export-pdf-download'),
    false,
  );
});
