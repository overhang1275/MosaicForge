import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { browserPorts } from '../src/editor/client.ts';
import type { PdfExportRequest } from '../../../contracts/generated/typescript/preview.ts';

test('preview cannot count an artifact that the browser cannot decode', async (t) => {
  t.mock.method(globalThis, 'fetch', async () =>
    Response.json({
      variant: { status: 'accepted' },
      artifact: { mimeType: 'image/png', base64: 'AA==' },
    }),
  );
  const descriptor = Object.getOwnPropertyDescriptor(
    globalThis,
    'createImageBitmap',
  );
  Object.defineProperty(globalThis, 'createImageBitmap', {
    configurable: true,
    value: async () => {
      throw new Error('decode failed');
    },
  });
  t.after(() => {
    if (descriptor)
      Object.defineProperty(globalThis, 'createImageBitmap', descriptor);
    else Reflect.deleteProperty(globalThis, 'createImageBitmap');
  });
  const fixture = JSON.parse(
    await readFile(
      new URL(
        '../../../contracts/preview/v1/fixtures/export-svg.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  await assert.rejects(
    browserPorts.preview(fixture, new AbortController().signal),
    /PNG|cargar|decodificar/i,
  );
});

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
      {
        schemaVersion: '2',
        variantIndices: ['0', '1', '2'],
        templateSvg: fixture.templateSvg,
        job: fixture.job,
      },
      new AbortController().signal,
    ),
    /Reduce la cantidad de variantes/,
  );
  await assert.rejects(
    browserPorts.inspect(fixture.templateSvg, new AbortController().signal),
    { message: 'Se excedió el tamaño permitido' },
  );
});
