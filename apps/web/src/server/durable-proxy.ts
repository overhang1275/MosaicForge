import { createHash, randomUUID } from 'node:crypto';
import validators from '../../../../contracts/generated/durable-validators.cjs';
import type { DurableJobStatus } from '../../../../contracts/generated/typescript/durable.ts';
import type { ProjectError } from '../../../../contracts/generated/typescript/projects.ts';
import {
  bounded,
  BodyLimit,
  isJson,
  localAddress,
  traceContext,
  type ProxyConfig,
} from './proxy.ts';

const uuid = '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}';
const valid = validators as Record<string, (value: unknown) => boolean>;
const errorStatuses: Record<ProjectError['code'], number> = {
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
  not_found: 404,
  conflict: 409,
  unavailable: 503,
};
function failure(
  status: number,
  id: string,
  code: ProjectError['code'],
): Response {
  return Response.json(
    {
      requestId: id,
      code,
      message: 'No se pudo completar la solicitud',
    } satisfies ProjectError,
    {
      status,
      headers: {
        'cache-control': 'no-store',
        'x-content-type-options': 'nosniff',
      },
    },
  );
}
interface Mapping {
  path: string;
  methods: string[];
  response: string;
  input?: string;
  jobId?: string;
  kind?: 'pdf' | 'manifest';
}
function mapping(url: URL): Mapping | null {
  const p = url.pathname;
  if (p === '/api/jobs')
    return {
      path: '/v2/jobs',
      methods: ['POST'],
      input: 'durable-job-request',
      response: 'durable-job-status',
    };
  let m = p.match(
    new RegExp(`^/api/jobs/(${uuid})(?:/(cancel|artifacts/(pdf|manifest)))?$`),
  );
  if (m)
    return {
      path: p.replace('/api/jobs', '/v2/jobs'),
      methods: m[2] === 'cancel' ? ['POST'] : m[3] ? ['GET', 'HEAD'] : ['GET'],
      response: 'durable-job-status',
      jobId: m[1],
      kind: m[3] as Mapping['kind'],
    };
  if (p === '/api/projects')
    return {
      path: '/v1/projects',
      methods: ['GET', 'POST'],
      response: 'project-list-response',
      input: 'project-create-request',
    };
  if (p === '/api/projects/me')
    return {
      path: '/v1/me',
      methods: ['GET'],
      response: 'project-me-response',
    };
  m = p.match(
    new RegExp(
      `^/api/projects/(${uuid})(?:/revisions(?:/([1-9][0-9]{0,9}))?)?$`,
    ),
  );
  if (!m || (m[2] && Number(m[2]) > 2147483647)) return null;
  return {
    path: p.replace('/api/projects', '/v1/projects'),
    methods: p.endsWith('/revisions') ? ['POST'] : ['GET'],
    response:
      m[2] || p.endsWith('/revisions')
        ? 'project-revision-response'
        : 'project-project-response',
    input: 'project-revision-request',
  };
}
function semanticStatus(value: unknown, jobId?: string): boolean {
  const s = value as DurableJobStatus;
  return (
    (!jobId || s.jobId === jobId) &&
    s.evaluatedCount === s.acceptedCount + s.rejectedCount &&
    s.evaluatedCount <= s.maxCandidates &&
    s.maxCandidates >= s.requestedAcceptedCount &&
    s.acceptedCount <= s.requestedAcceptedCount &&
    (s.nextCandidateIndex === null ||
      BigInt(s.nextCandidateIndex) <= 18446744073709551615n) &&
    (s.state === 'succeeded') === (s.artifact !== null) &&
    (s.state !== 'succeeded' ||
      (s.acceptedCount === s.requestedAcceptedCount &&
        s.artifact!.pageCount === s.requestedAcceptedCount &&
        BigInt(s.artifact!.bytes) > 0n &&
        BigInt(s.artifact!.bytes) <= 1073741824n))
  );
}
function artifact(
  response: Response,
  request: Request,
  map: Mapping,
  signal: AbortSignal,
): Response {
  const h = response.headers,
    length = h.get('content-length') ?? '',
    etag = h.get('etag') ?? '',
    type = map.kind === 'pdf' ? 'application/pdf' : 'application/x-ndjson';
  if (
    ![200, 206].includes(response.status) ||
    h.get('content-type') !== type ||
    !/^"[a-f0-9]{64}"$/.test(etag) ||
    !/^(0|[1-9][0-9]*)$/.test(length) ||
    Number(length) > 1024 ** 3 ||
    h.get('accept-ranges') !== 'bytes' ||
    (h.has('content-encoding') && h.get('content-encoding') !== 'identity')
  )
    throw new Error('Invalid artifact headers');
  const range = h.get('content-range');
  if (response.status === 206) {
    const m = range?.match(
      /^bytes (0|[1-9][0-9]*)-(0|[1-9][0-9]*)\/([1-9][0-9]*)$/,
    );
    const requested = request.headers
      .get('range')
      ?.match(/^bytes=(\d*)-(\d*)$/);
    if (!m || !requested) throw new Error('Invalid partial response');
    const [start, end, total] = m.slice(1).map(Number);
    if (
      total > 1024 ** 3 ||
      start > end ||
      end >= total ||
      end - start + 1 !== Number(length)
    )
      throw new Error('Invalid range length');
    const expectedStart = requested[1]
      ? Number(requested[1])
      : Math.max(0, total - Number(requested[2]));
    const expectedEnd =
      requested[1] && requested[2]
        ? Math.min(total - 1, Number(requested[2]))
        : total - 1;
    if (start !== expectedStart || end !== expectedEnd)
      throw new Error('Mismatched range');
    const ifRange = request.headers.get('if-range');
    if (ifRange && ifRange !== etag)
      throw new Error('Mismatched range validator');
  } else if (range || Number(length) === 0)
    throw new Error('Invalid full response');
  const headers = new Headers({
    'content-type': type,
    'content-length': length,
    etag,
    'accept-ranges': 'bytes',
    'cache-control': 'no-store',
    'x-content-type-options': 'nosniff',
    'content-security-policy': "default-src 'none'; sandbox",
    'content-disposition': `attachment; filename="mosaic-${map.jobId}.${map.kind === 'pdf' ? 'pdf' : 'ndjson'}"`,
  });
  if (range) headers.set('content-range', range);
  if (request.method === 'HEAD') {
    void response.body?.cancel();
    return new Response(null, { status: response.status, headers });
  }
  if (!response.body) throw new Error('Missing artifact body');
  const hash = createHash('sha256');
  let received = 0,
    last: number | null = null;
  const checked = new TransformStream<Uint8Array, Uint8Array>({
    transform(chunk, controller) {
      signal.throwIfAborted();
      received += chunk.length;
      if (received > Number(length)) throw new Error('Artifact exceeds length');
      hash.update(chunk);
      if (chunk.length) {
        if (last !== null) controller.enqueue(Uint8Array.of(last));
        if (chunk.length > 1) controller.enqueue(chunk.subarray(0, -1));
        last = chunk[chunk.length - 1];
      }
    },
    flush(controller) {
      signal.throwIfAborted();
      if (
        received !== Number(length) ||
        (response.status === 200 && hash.digest('hex') !== etag.slice(1, -1))
      )
        throw new Error('Incomplete or corrupt artifact');
      if (last !== null) controller.enqueue(Uint8Array.of(last));
    },
  });
  return new Response(response.body.pipeThrough(checked, { signal }), {
    status: response.status,
    headers,
  });
}
export function createDurableProxy(config: ProxyConfig) {
  const upstream = localAddress(config.engineUrl),
    web = localAddress(config.origin);
  if (!/^[\x21-\x7e]{32,256}$/.test(config.token))
    throw new Error('API token required');
  const origins = new Set([
    web.origin,
    web.origin.replace('127.0.0.1', 'localhost'),
  ]);
  const hosts = new Set([...origins].map((o) => new URL(o).host));
  const timeout = config.timeoutMs ?? 30000,
    downloadTimeout = config.pdfTimeoutMs ?? 120000;
  if (
    !Number.isInteger(timeout) ||
    timeout < 1 ||
    timeout > 30000 ||
    !Number.isInteger(downloadTimeout) ||
    downloadTimeout < 1 ||
    downloadTimeout > 120000
  )
    throw new Error('Invalid deadline');
  return async (request: Request): Promise<Response> => {
    const id = randomUUID(),
      trace = traceContext(request.headers.get('traceparent'));
    const finish = (response: Response) => {
      response.headers.set('traceparent', trace);
      return response;
    };
    const fail = (status: number, code: ProjectError['code']) =>
      finish(failure(status, id, code));
    const origin = request.headers.get('origin'),
      host = request.headers.get('host');
    if (
      !host ||
      !hosts.has(host) ||
      (origin
        ? !origins.has(origin) || new URL(origin).host !== host
        : (request.method !== 'GET' && request.method !== 'HEAD') ||
          !['same-origin', 'none'].includes(
            request.headers.get('sec-fetch-site') ?? '',
          ))
    )
      return fail(403, 'forbidden');
    const url = new URL(request.url),
      map = mapping(url);
    if (!map) return fail(404, 'not_found');
    if (!map.methods.includes(request.method))
      return finish(
        new Response(null, {
          status: 405,
          headers: { allow: map.methods.join(', ') },
        }),
      );
    for (const [key, value] of url.searchParams) {
      if (
        url.pathname !== '/api/projects' ||
        request.method !== 'GET' ||
        !['cursor', 'limit'].includes(key) ||
        url.searchParams.getAll(key).length !== 1 ||
        (key === 'cursor'
          ? !new RegExp(`^${uuid}$`).test(value)
          : !/^(?:[1-9]|[1-9][0-9]|100)$/.test(value))
      )
        return fail(400, 'invalid_request');
    }
    const deadline = AbortSignal.timeout(map.kind ? downloadTimeout : timeout),
      signal = AbortSignal.any([deadline, request.signal]);
    const headers = new Headers({
      authorization: `Bearer ${config.token}`,
      traceparent: trace,
      'x-request-id': id,
      'accept-encoding': 'identity',
    });
    let body: string | undefined;
    try {
      if (request.method === 'POST') {
        if (map.input) {
          if (!isJson(request.headers))
            return fail(415, 'unsupported_media_type');
          const bytes = await bounded(
            request.body,
            request.headers,
            32 * 1024 ** 2,
            signal,
          );
          body = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
          if (!valid[map.input](JSON.parse(body)))
            return fail(400, 'invalid_request');
          headers.set('content-type', 'application/json');
        } else if (
          (await bounded(request.body, request.headers, 0, signal)).length
        )
          return fail(400, 'invalid_request');
        if (map.response === 'durable-job-status' && !map.jobId) {
          const key = request.headers.get('idempotency-key') ?? '';
          if (!/^[\x21-\x7e]{16,128}$/.test(key))
            return fail(400, 'invalid_request');
          headers.set('idempotency-key', key);
        }
      }
      if (map.kind) {
        const range = request.headers.get('range'),
          ifRange = request.headers.get('if-range');
        if (
          (range &&
            (range.length > 100 || !/^bytes=(?:\d+-\d*|-\d+)$/.test(range))) ||
          (ifRange && (ifRange.length > 100 || !/^[\x20-\x7e]+$/.test(ifRange)))
        )
          return fail(400, 'invalid_request');
        if (range) headers.set('range', range);
        if (ifRange) headers.set('if-range', ifRange);
      }
    } catch (cause) {
      return fail(
        cause instanceof BodyLimit ? 413 : deadline.aborted ? 504 : 400,
        cause instanceof BodyLimit
          ? 'payload_too_large'
          : deadline.aborted
            ? 'timeout'
            : 'invalid_request',
      );
    }
    try {
      const response = await fetch(new URL(map.path + url.search, upstream), {
        method: request.method,
        headers,
        body,
        signal,
        redirect: 'error',
        cache: 'no-store',
      });
      if (map.kind && [200, 206].includes(response.status)) {
        try {
          return finish(artifact(response, request, map, signal));
        } catch {
          await response.body?.cancel();
          return fail(502, 'engine_unavailable');
        }
      }
      if (!isJson(response.headers)) {
        await response.body?.cancel();
        return fail(502, 'engine_unavailable');
      }
      const unsatisfied = map.kind && response.status === 416;
      const contentRange = response.headers.get('content-range') ?? '';
      if (
        unsatisfied &&
        (!request.headers.has('range') ||
          !/^bytes \*\/(0|[1-9][0-9]*)$/.test(contentRange) ||
          Number(contentRange.split('/')[1]) > 1024 ** 3)
      ) {
        await response.body?.cancel();
        return fail(502, 'engine_unavailable');
      }
      if (unsatisfied && request.method === 'HEAD') {
        return finish(
          new Response(null, {
            status: 416,
            headers: {
              'content-range': contentRange,
              'cache-control': 'no-store',
              'x-content-type-options': 'nosniff',
            },
          }),
        );
      }
      // HEAD has the GET status and headers, but no error document to validate.
      if (request.method === 'HEAD' && !response.ok) {
        return finish(
          new Response(null, {
            status: Object.values(errorStatuses).includes(response.status)
              ? response.status
              : 502,
            headers: {
              'content-type': 'application/json',
              'cache-control': 'no-store',
              'x-content-type-options': 'nosniff',
            },
          }),
        );
      }
      const bytes = await bounded(
          response.body,
          response.headers,
          16 * 1024 ** 2,
          signal,
        ),
        value: unknown = JSON.parse(
          new TextDecoder('utf-8', { fatal: true }).decode(bytes),
        );
      if (!response.ok) {
        if (!valid['project-error'](value))
          return fail(502, 'engine_unavailable');
        const e = value as ProjectError;
        if (unsatisfied && e.code === 'invalid_request') {
          const output = fail(416, e.code);
          output.headers.set('content-range', contentRange);
          return output;
        }
        return errorStatuses[e.code] === response.status
          ? fail(response.status, e.code)
          : fail(502, 'engine_unavailable');
      }
      const schema =
        map.path === '/v1/projects' && request.method === 'POST'
          ? 'project-project-response'
          : map.response;
      const expected =
        request.method === 'POST'
          ? map.path === '/v2/jobs'
            ? [202]
            : map.path.endsWith('/cancel')
              ? [200, 202]
              : [201]
          : [200];
      if (
        !expected.includes(response.status) ||
        !valid[schema](value) ||
        (schema === 'durable-job-status' && !semanticStatus(value, map.jobId))
      )
        return fail(502, 'engine_unavailable');
      return finish(
        Response.json(value, {
          status: response.status,
          headers: {
            'cache-control': 'no-store',
            'x-content-type-options': 'nosniff',
          },
        }),
      );
    } catch {
      return fail(
        deadline.aborted ? 504 : 502,
        deadline.aborted ? 'timeout' : 'engine_unavailable',
      );
    }
  };
}
