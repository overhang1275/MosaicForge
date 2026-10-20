import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { pipeToNodeResponse } from 'next/dist/server/pipe-readable.js';
import type { SelectedPdfExportRequest } from '../../../contracts/generated/typescript/preview.ts';
import { pdfStream } from '../src/server/pdf-stream.ts';

test('a corrupt PDF cannot complete over HTTP before its stream digest is checked', async () => {
  const fixtures = new URL(
    '../../../contracts/preview/v1/fixtures/',
    import.meta.url,
  );
  const input = JSON.parse(
    await readFile(new URL('export-pdf.json', fixtures), 'utf8'),
  );
  input.schemaVersion = '2';
  input.variantIndices = ['0', '1', '2'];
  const metadata = JSON.parse(
    await readFile(new URL('accepted-pdf.json', fixtures), 'utf8'),
  );
  delete metadata.artifact;
  metadata.schemaVersion = '3';
  metadata.variantIndices = input.variantIndices;
  metadata.document.documentHash = '0'.repeat(64);
  const bytes = Buffer.alloc(64 * 1024, 32);
  bytes.write('%PDF-1.7');
  const server = createServer(async (_request, response) => {
    const source = new ReadableStream<Uint8Array>({
      start(controller) {
        controller.enqueue(bytes);
        // The final bytes may arrive before the upstream connection closes.
        setTimeout(() => controller.close(), 50);
      },
    });
    const checked = pdfStream(
      new Response(source, {
        headers: {
          'content-length': String(bytes.length),
          'x-mosaicforge-result': JSON.stringify(metadata),
        },
      }),
      input as SelectedPdfExportRequest,
      input.requestId,
    );
    for (const [name, value] of checked.headers)
      response.setHeader(name, value);
    try {
      await pipeToNodeResponse(checked.body!, response);
    } catch {
      // Next aborts the HTTP response when the checked body rejects.
    }
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  try {
    const address = server.address();
    assert.ok(address && typeof address !== 'string');
    await assert.rejects(async () => {
      const response = await fetch(`http://127.0.0.1:${address.port}`);
      await response.blob();
    });
  } finally {
    server.closeAllConnections();
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
