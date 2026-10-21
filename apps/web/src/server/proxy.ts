import { randomUUID } from 'node:crypto';
import { pdfStream } from './pdf-stream.ts';
import {
  controlOperations,
  type ControlOperation,
} from '../../../../contracts/generated/typescript/control-api.ts';
import type {
  GenerationRequest,
  GenerationResponse,
  InspectRequest,
  PreviewError,
  PdfExportRequest,
  PdfExportResponse,
  SelectedPdfExportRequest,
  SelectedPdfExportResponse,
} from '../../../../contracts/generated/typescript/preview.ts';
import { validateFixture } from '../../../../scripts/contracts/validate.mjs';

export type Operation = ControlOperation;
export interface ProxyConfig {
  engineUrl: string;
  token: string;
  origin: string;
  timeoutMs?: number;
  pdfTimeoutMs?: number;
}
const MAX_BODY = 32 * 1024 * 1024,
  MAX_RESPONSE = 16 * 1024 * 1024;
const UNKNOWN_REQUEST = '00000000-0000-0000-0000-000000000000';
export class BodyLimit extends Error {}
export function localAddress(value: string): URL {
  const url = new URL(value);
  if (
    url.protocol !== 'http:' ||
    url.hostname !== '127.0.0.1' ||
    url.username ||
    url.password ||
    url.pathname !== '/' ||
    url.search ||
    url.hash ||
    value !== url.origin
  )
    throw new Error('Expected a canonical HTTP loopback origin');
  return url;
}
function error(
  status: number,
  id: string,
  code: PreviewError['code'],
): Response {
  const messages: Record<PreviewError['code'], string> = {
    invalid_request: 'Solicitud inválida',
    invalid_template: 'SVG no admitido o inválido',
    unauthorized: 'El motor requiere credenciales válidas',
    forbidden: 'Solicitud no permitida',
    payload_too_large: 'Se excedió el tamaño permitido',
    unsupported_media_type: 'Se requiere JSON',
    busy: 'El motor está ocupado; vuelve a intentarlo',
    internal_error: 'Falló la operación del motor',
    engine_unavailable: 'El motor no está disponible',
    timeout: 'Se agotó el tiempo de espera',
  };
  return Response.json(
    { requestId: id, code, message: messages[code] } satisfies PreviewError,
    {
      status,
      headers: {
        'cache-control': 'no-store',
        'x-content-type-options': 'nosniff',
      },
    },
  );
}
export async function bounded(
  body: ReadableStream<Uint8Array> | null,
  headers: Headers,
  limit: number,
  signal: AbortSignal,
): Promise<Uint8Array> {
  if (Number(headers.get('content-length')) > limit) throw new BodyLimit();
  if (!body) return new Uint8Array();
  const reader = body.getReader(),
    chunks: Uint8Array[] = [];
  let length = 0;
  const abort = () => {
    void reader.cancel().catch(() => {});
  };
  signal.addEventListener('abort', abort, { once: true });
  try {
    signal.throwIfAborted();
    while (true) {
      const { value, done } = await reader.read();
      signal.throwIfAborted();
      if (done) break;
      length += value.byteLength;
      if (length > limit) throw new BodyLimit();
      chunks.push(value);
    }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.length;
    }
    return bytes;
  } catch (cause) {
    await reader.cancel().catch(() => {});
    throw cause;
  } finally {
    signal.removeEventListener('abort', abort);
    reader.releaseLock();
  }
}
export function isJson(headers: Headers): boolean {
  return (
    headers.get('content-type')?.split(';')[0]?.trim().toLowerCase() ===
    'application/json'
  );
}
const errorStatus: Partial<Record<PreviewError['code'], number>> = {
  invalid_request: 400,
  invalid_template: 400,
  unauthorized: 401,
  forbidden: 403,
  payload_too_large: 413,
  unsupported_media_type: 415,
  busy: 429,
  internal_error: 500,
  engine_unavailable: 502,
  timeout: 504,
};

export function traceContext(value: string | null): string {
  if (
    value &&
    /^00-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$/.test(value) &&
    value.slice(3, 35) !== '0'.repeat(32) &&
    value.slice(36, 52) !== '0'.repeat(16)
  )
    return value;
  return `00-${randomUUID().replaceAll('-', '')}-${randomUUID().replaceAll('-', '').slice(0, 16)}-01`;
}

export function createProxy(config: ProxyConfig) {
  const upstream = localAddress(config.engineUrl);
  const webAddress = localAddress(config.origin);
  const allowedOrigins = new Set([config.origin]);
  if (webAddress.hostname === '127.0.0.1') {
    const localhostOrigin = new URL(config.origin);
    localhostOrigin.hostname = 'localhost';
    allowedOrigins.add(localhostOrigin.origin);
  }
  if (!/^[\x21-\x7e]{32,256}$/.test(config.token))
    throw new Error('Private engine token required');
  const timeoutMs = config.timeoutMs ?? 30_000;
  const pdfTimeoutMs = config.pdfTimeoutMs ?? 120_000;
  if (
    !Number.isInteger(pdfTimeoutMs) ||
    pdfTimeoutMs < 1 ||
    pdfTimeoutMs > 120_000
  )
    throw new Error('Invalid PDF timeout');
  if (!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > 30_000)
    throw new Error('Invalid timeout');
  const execute = async (
    request: Request,
    operation: Operation,
    trace: string,
  ): Promise<Response> => {
    const id = randomUUID();
    if (request.method !== 'POST')
      return new Response(null, { status: 405, headers: { allow: 'POST' } });
    const requestOrigin = request.headers.get('origin');
    let originHost: string | undefined;
    if (requestOrigin && allowedOrigins.has(requestOrigin))
      originHost = new URL(requestOrigin).host;
    if (!originHost || request.headers.get('host') !== originHost)
      return error(403, id, 'forbidden');
    if (!isJson(request.headers))
      return error(415, id, 'unsupported_media_type');
    const deadline = AbortSignal.timeout(
        operation.startsWith('export-pdf') ? pdfTimeoutMs : timeoutMs,
      ),
      signal = AbortSignal.any([deadline, request.signal]);
    let input:
      | InspectRequest
      | GenerationRequest
      | PdfExportRequest
      | SelectedPdfExportRequest;
    try {
      const bytes = await bounded(
        request.body,
        request.headers,
        MAX_BODY,
        signal,
      );
      const value: unknown = JSON.parse(
        new TextDecoder('utf-8', { fatal: true }).decode(bytes),
      );
      if (!value || typeof value !== 'object' || Array.isArray(value))
        return error(400, id, 'invalid_request');
      const envelope = { ...value, requestId: id };
      if (
        !validateFixture(
          envelope,
          operation === 'inspect'
            ? 'inspect-request'
            : operation === 'export-pdf-selection' ||
                operation === 'export-pdf-download'
              ? 'selected-pdf-export-request'
              : operation === 'export-pdf'
                ? 'pdf-export-request'
                : 'generation-request',
          operation,
        )
      )
        return error(400, id, 'invalid_request');
      input = envelope as
        | InspectRequest
        | GenerationRequest
        | PdfExportRequest
        | SelectedPdfExportRequest;
    } catch (cause) {
      return error(
        cause instanceof BodyLimit ? 413 : deadline.aborted ? 504 : 400,
        id,
        cause instanceof BodyLimit
          ? 'payload_too_large'
          : deadline.aborted
            ? 'timeout'
            : 'invalid_request',
      );
    }
    try {
      const response = await fetch(
        new URL(controlOperations[operation].path, upstream),
        {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            authorization: `Bearer ${config.token}`,
            traceparent: trace,
          },
          body: JSON.stringify(input),
          signal,
          redirect: 'error',
          cache: 'no-store',
        },
      );
      if (
        operation === 'export-pdf-download' &&
        response.ok &&
        response.headers.get('content-type')?.split(';')[0] ===
          'application/pdf'
      ) {
        try {
          return pdfStream(response, input as SelectedPdfExportRequest, id);
        } catch {
          await response.body?.cancel();
          return error(502, id, 'engine_unavailable');
        }
      }
      if (!isJson(response.headers)) {
        await response.body?.cancel();
        return error(502, id, 'engine_unavailable');
      }
      const bytes = await bounded(
        response.body,
        response.headers,
        MAX_RESPONSE,
        signal,
      );
      const value: unknown = JSON.parse(
        new TextDecoder('utf-8', { fatal: true }).decode(bytes),
      );
      if (!response.ok) {
        if (!validateFixture(value, 'error', operation))
          return error(502, id, 'engine_unavailable');
        const failure = value as PreviewError;
        if (
          (failure.requestId !== id && failure.requestId !== UNKNOWN_REQUEST) ||
          errorStatus[failure.code] !== response.status
        )
          return error(502, id, 'engine_unavailable');
        return error(response.status, id, failure.code);
      }
      if (
        !validateFixture(
          value,
          operation === 'inspect'
            ? 'inspect-response'
            : operation === 'export-pdf-selection'
              ? 'selected-pdf-export-response'
              : operation === 'export-pdf'
                ? 'pdf-export-response'
                : 'generation-response',
          operation,
        )
      )
        return error(502, id, 'engine_unavailable');
      if ((value as { requestId: string }).requestId !== id)
        return error(502, id, 'engine_unavailable');
      if (operation === 'inspect')
        return Response.json(value, {
          headers: {
            'cache-control': 'no-store',
            'x-content-type-options': 'nosniff',
          },
        });
      if (operation === 'export-pdf' || operation === 'export-pdf-selection') {
        const output = value as PdfExportResponse,
          spec = (input as PdfExportRequest).job.generation;
        const d = output.document;
        if (
          operation === 'export-pdf-selection' &&
          JSON.stringify(
            (value as SelectedPdfExportResponse).variantIndices,
          ) !==
            JSON.stringify((input as SelectedPdfExportRequest).variantIndices)
        )
          return error(502, id, 'engine_unavailable');
        if (
          d.seed !== spec.seed ||
          d.startIndex !== spec.startIndex ||
          d.templateHash !== spec.templateHash ||
          d.count !== spec.count
        )
          return error(502, id, 'engine_unavailable');
        return new Response(Buffer.from(output.artifact.base64, 'base64'), {
          headers: {
            'content-type': 'application/pdf',
            'content-disposition': 'attachment; filename="mosaic.pdf"',
            'content-security-policy': "default-src 'none'; sandbox",
            'x-content-type-options': 'nosniff',
            'cache-control': 'no-store',
            'x-request-id': id,
          },
        });
      }
      const output = value as GenerationResponse,
        spec = (input as GenerationRequest).generation;
      if (
        output.variant.index !== spec.startIndex ||
        output.variant.seed !== spec.seed ||
        output.variant.rulesVersion !== spec.rulesVersion ||
        output.variant.templateHash !== spec.templateHash ||
        (output.artifact &&
          output.artifact.mimeType !==
            (spec.output.format === 'png' ? 'image/png' : 'image/svg+xml'))
      )
        return error(502, id, 'engine_unavailable');
      if (operation !== 'export' || !output.artifact)
        return Response.json(output, {
          headers: {
            'cache-control': 'no-store',
            'x-content-type-options': 'nosniff',
          },
        });
      const artifact = Buffer.from(output.artifact.base64, 'base64');
      return new Response(artifact, {
        headers: {
          'content-type': output.artifact.mimeType,
          'content-disposition': `attachment; filename="variant-${output.variant.index}.${spec.output.format}"`,
          'content-security-policy': "default-src 'none'; sandbox",
          'x-content-type-options': 'nosniff',
          'cache-control': 'no-store',
          'x-request-id': id,
        },
      });
    } catch {
      return error(
        deadline.aborted ? 504 : request.signal.aborted ? 400 : 502,
        id,
        deadline.aborted
          ? 'timeout'
          : request.signal.aborted
            ? 'invalid_request'
            : 'engine_unavailable',
      );
    }
  };
  return async (request: Request, operation: Operation): Promise<Response> => {
    const trace = traceContext(request.headers.get('traceparent'));
    const response = await execute(request, operation, trace);
    response.headers.set('traceparent', trace);
    return response;
  };
}
