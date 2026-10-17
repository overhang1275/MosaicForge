import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { browserPorts } from '../src/editor/client.ts';
import type { PdfExportRequest } from '../../../contracts/generated/typescript/preview.ts';

test('oversized PDF explains recovery while other operations preserve their error message', async (t) => {
  t.mock.method(globalThis, 'fetch', async () =>
    Response.json(
      { code: 'payload_too_large', message: 'Se excedió el tamaño permitido' },
      { status: 413 },
    ),
  );
  const fixture: PdfExportRequest = JSON.parse(
    await readFile(
      new URL(
        '../../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  await assert.rejects(
    browserPorts.exportPdf(
      { templateSvg: fixture.templateSvg, job: fixture.job },
      new AbortController().signal,
    ),
    /Reduce la cantidad de variantes/,
  );
  await assert.rejects(
    browserPorts.inspect(fixture.templateSvg, new AbortController().signal),
    { message: 'Se excedió el tamaño permitido' },
  );
});
