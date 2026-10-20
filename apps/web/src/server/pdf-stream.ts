import { createHash } from 'node:crypto';
import type {
  PdfDownloadResponse,
  SelectedPdfExportRequest,
} from '../../../../contracts/generated/typescript/preview.ts';
import { validateFixture } from '../../../../scripts/contracts/validate.mjs';

export function pdfStream(
  response: Response,
  input: SelectedPdfExportRequest,
  id: string,
): Response {
  const raw = response.headers.get('x-mosaicforge-result') ?? '';
  const length = response.headers.get('content-length') ?? '';
  if (
    raw.length > 8192 ||
    !/^[1-9][0-9]*$/.test(length) ||
    Number(length) > 1024 ** 3 ||
    Number(length) < 8 ||
    !response.body
  )
    throw new Error('Invalid PDF envelope');
  const metadata: unknown = JSON.parse(raw);
  if (
    !validateFixture(metadata, 'pdf-download-response', 'export-pdf-download')
  )
    throw new Error('Invalid PDF metadata');
  const result = metadata as PdfDownloadResponse;
  const spec = input.job.generation,
    d = result.document;
  if (
    result.requestId !== id ||
    d.seed !== spec.seed ||
    d.templateHash !== spec.templateHash ||
    d.startIndex !== spec.startIndex ||
    d.count !== spec.count ||
    JSON.stringify(result.variantIndices) !==
      JSON.stringify(input.variantIndices)
  )
    throw new Error('Mismatched PDF identity');
  const hash = createHash('sha256');
  let received = 0,
    finalByte: number | null = null,
    prefix = Buffer.alloc(0);
  const checked = new TransformStream<Uint8Array, Uint8Array>({
    transform(chunk, controller) {
      received += chunk.byteLength;
      if (received > Number(length))
        throw new Error('PDF exceeds declared length');
      if (prefix.length < 8)
        prefix = Buffer.concat([prefix, chunk.subarray(0, 8 - prefix.length)]);
      hash.update(chunk);
      // Do not complete HTTP Content-Length before validating the digest at EOF.
      if (chunk.byteLength) {
        if (finalByte !== null) controller.enqueue(Uint8Array.of(finalByte));
        if (chunk.byteLength > 1)
          controller.enqueue(chunk.subarray(0, chunk.byteLength - 1));
        finalByte = chunk[chunk.byteLength - 1];
      }
    },
    flush(controller) {
      if (
        received !== Number(length) ||
        prefix.toString('ascii') !== '%PDF-1.7' ||
        hash.digest('hex') !== d.documentHash
      )
        throw new Error('Incomplete or corrupt PDF download');
      if (finalByte !== null) controller.enqueue(Uint8Array.of(finalByte));
    },
  });
  return new Response(response.body.pipeThrough(checked), {
    headers: {
      'content-type': 'application/pdf',
      'content-length': length,
      'content-disposition': 'attachment; filename="mosaic.pdf"',
      'content-security-policy': "default-src 'none'; sandbox",
      'x-content-type-options': 'nosniff',
      'cache-control': 'no-store',
      'x-request-id': id,
    },
  });
}
