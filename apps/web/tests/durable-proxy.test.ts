import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { createHash } from 'node:crypto';
import { createDurableProxy } from '../src/server/durable-proxy.ts';
const id = '11111111-1111-1111-1111-111111111111';
const origin = 'http://127.0.0.1:3000';
const bytes = Buffer.from('%PDF-1.7\nverified document');
async function stub(
  run: (
    proxy: ReturnType<typeof createDurableProxy>,
    requests: string[],
  ) => Promise<void>,
  corrupt = false,
) {
  const requests: string[] = [];
  const server = createServer((req, res) => {
    requests.push(req.url!);
    if (req.url?.includes('artifacts')) {
      res.writeHead(200, {
        'content-type': 'application/pdf',
        'content-length': bytes.length,
        etag: `"${corrupt ? '0'.repeat(64) : createHash('sha256').update(bytes).digest('hex')}"`,
        'accept-ranges': 'bytes',
      });
      res.end(bytes);
    } else {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(
        JSON.stringify({
          schemaVersion: '2',
          jobId: id,
          projectId: id,
          requestedAcceptedCount: 12,
          maxCandidates: 20,
          acceptedCount: 0,
          rejectedCount: 0,
          evaluatedCount: 0,
          nextCandidateIndex: '0',
          revision: 1,
          state: 'queued',
          artifact: null,
        }),
      );
    }
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const address = server.address();
  assert.ok(address && typeof address !== 'string');
  try {
    await run(
      createDurableProxy({
        engineUrl: `http://127.0.0.1:${address.port}`,
        token: 'a'.repeat(32),
        origin,
      }),
      requests,
    );
  } finally {
    server.closeAllConnections();
    await new Promise<void>((resolve) => server.close(() => resolve()));
  }
}
const request = (path: string, headers: Record<string, string> = {}) =>
  new Request(origin + path, {
    headers: {
      host: '127.0.0.1:3000',
      'sec-fetch-site': 'same-origin',
      ...headers,
    },
  });
test('durable proxy guards native reads and fixed routes', async () =>
  stub(async (proxy, requests) => {
    assert.equal((await proxy(request(`/api/jobs/${id}`))).status, 200);
    for (const req of [
      request(`/api/jobs/${id}`, { origin: 'http://evil.test' }),
      request(`/api/jobs/${id}`, { 'sec-fetch-site': 'cross-site' }),
      request(`/api/jobs/${id}`, { host: 'evil.test' }),
    ])
      assert.equal((await proxy(req)).status, 403);
    assert.equal(
      (await proxy(request(`/api/jobs/${id}?url=http://evil.test`))).status,
      400,
    );
    assert.equal((await proxy(request('/api/jobs/internal'))).status, 404);
    assert.deepEqual(requests, [`/v2/jobs/${id}`]);
  }));
test('durable artifacts stream native bytes and safe headers', async () =>
  stub(async (proxy) => {
    const response = await proxy(request(`/api/jobs/${id}/artifacts/pdf`));
    assert.equal(response.status, 200);
    assert.match(response.headers.get('content-disposition')!, /attachment/);
    assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);
  }));
test('corrupt artifact cannot finish its declared body', async () =>
  stub(async (proxy) => {
    const response = await proxy(request(`/api/jobs/${id}/artifacts/pdf`));
    await assert.rejects(() => response.arrayBuffer());
  }, true));

import type { IncomingMessage, ServerResponse } from 'node:http';
import { readFile } from 'node:fs/promises';
async function customStub(
  handler: (req: IncomingMessage, res: ServerResponse) => void,
  run: (proxy: ReturnType<typeof createDurableProxy>) => Promise<void>,
) {
  const server = createServer(handler);
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const address = server.address();
  assert.ok(address && typeof address !== 'string');
  try {
    await run(
      createDurableProxy({
        engineUrl: `http://127.0.0.1:${address.port}`,
        token: 'a'.repeat(32),
        origin,
      }),
    );
  } finally {
    server.closeAllConnections();
    await new Promise<void>((resolve) => server.close(() => resolve()));
  }
}
const etag = `"${createHash('sha256').update(bytes).digest('hex')}"`;
const artifactHeaders = {
  'content-type': 'application/pdf',
  'content-length': String(bytes.length),
  etag,
  'accept-ranges': 'bytes',
};
test('range/HEAD forwarding preserves validated headers without buffering', async () =>
  customStub(
    (req, res) => {
      assert.equal(req.headers.authorization, `Bearer ${'a'.repeat(32)}`);
      assert.equal(req.headers.range, 'bytes=2-6');
      assert.equal(req.headers['if-range'], etag);
      res.writeHead(206, {
        ...artifactHeaders,
        'content-length': '5',
        'content-range': `bytes 2-6/${bytes.length}`,
      });
      res.end(req.method === 'HEAD' ? undefined : bytes.subarray(2, 7));
    },
    async (proxy) => {
      for (const method of ['GET', 'HEAD']) {
        const response = await proxy(
          new Request(`${origin}/api/jobs/${id}/artifacts/pdf`, {
            method,
            headers: {
              host: '127.0.0.1:3000',
              'sec-fetch-site': 'none',
              range: 'bytes=2-6',
              'if-range': etag,
            },
          }),
        );
        assert.equal(response.status, 206);
        assert.equal(response.headers.get('content-length'), '5');
        assert.equal(
          (await response.arrayBuffer()).byteLength,
          method === 'HEAD' ? 0 : 5,
        );
      }
    },
  ));
test('malformed ranges never reach upstream', async () =>
  stub(async (proxy, requests) => {
    for (const range of [
      'bytes=0-2,4-5',
      'items=0-2',
      'bytes=',
      'bytes=' + '0'.repeat(101) + '-',
    ])
      assert.equal(
        (await proxy(request(`/api/jobs/${id}/artifacts/pdf`, { range })))
          .status,
        400,
      );
    assert.equal(requests.length, 0);
  }));
test('mismatched interval, MIME, encoding and oversized artifacts fail closed', async () => {
  for (const headers of [
    { 'content-range': `bytes 3-7/${bytes.length}`, 'content-length': '5' },
    { 'content-type': 'text/html' },
    { 'content-encoding': 'gzip' },
    { 'content-length': String(1024 ** 3 + 1) },
  ]) {
    await customStub(
      (_req, res) => {
        res.writeHead('content-range' in headers ? 206 : 200, {
          ...artifactHeaders,
          ...headers,
        });
        res.end(bytes);
      },
      async (proxy) => {
        assert.equal(
          (
            await proxy(
              request(`/api/jobs/${id}/artifacts/pdf`, { range: 'bytes=2-6' }),
            )
          ).status,
          502,
        );
      },
    );
  }
});
test('truncated and stalled downloads never finish successfully', async () => {
  await customStub(
    (_req, res) => {
      res.writeHead(200, artifactHeaders);
      res.flushHeaders();
      res.write(bytes.subarray(0, 8));
      setTimeout(() => res.destroy(), 20);
    },
    async (proxy) => {
      const response = await proxy(request(`/api/jobs/${id}/artifacts/pdf`));
      assert.equal(response.status, 200);
      await assert.rejects(() => response.arrayBuffer());
    },
  );
});
test('consumer cancellation propagates to upstream', async () => {
  let closed: () => void = () => {};
  const didClose = new Promise<void>((resolve) => {
    closed = resolve;
  });
  await customStub(
    (_req, res) => {
      res.on('close', closed);
      res.writeHead(200, artifactHeaders);
      res.write(bytes.subarray(0, 8));
    },
    async (proxy) => {
      const response = await proxy(request(`/api/jobs/${id}/artifacts/pdf`));
      const reader = response.body!.getReader();
      assert.equal((await reader.read()).done, false);
      await reader.cancel();
      await Promise.race([
        didClose,
        new Promise((_, reject) =>
          setTimeout(
            () => reject(new Error('upstream was not cancelled')),
            1000,
          ),
        ),
      ]);
    },
  );
});
test('projects forward original JSON including duplicate keys for Java to reject', async () =>
  customStub(
    (req, res) => {
      let received = '';
      req.setEncoding('utf8');
      req.on('data', (part) => {
        received += part;
      });
      req.on('end', () => {
        assert.equal(
          received,
          '{ "schemaVersion":"1", "name":"old", "name":"new" }',
        );
        res.writeHead(400, { 'content-type': 'application/json' });
        res.end(
          JSON.stringify({
            requestId: id,
            code: 'invalid_request',
            message: 'private database text',
          }),
        );
      });
    },
    async (proxy) => {
      const response = await proxy(
        new Request(`${origin}/api/projects`, {
          method: 'POST',
          headers: {
            host: '127.0.0.1:3000',
            origin,
            'content-type': 'application/json',
          },
          body: '{ "schemaVersion":"1", "name":"old", "name":"new" }',
        }),
      );
      assert.equal(response.status, 400);
      assert.doesNotMatch(await response.text(), /private database/);
    },
  ));
test('status counters, identities and premature artifacts are rejected', async () => {
  for (const patch of [
    { acceptedCount: 1, evaluatedCount: 0 },
    { maxCandidates: 1 },
    { nextCandidateIndex: '18446744073709551616' },
    { jobId: '22222222-2222-2222-2222-222222222222' },
    { state: 'succeeded', artifact: null },
  ])
    await customStub(
      (_req, res) => {
        res.writeHead(200, { 'content-type': 'application/json' });
        res.end(
          JSON.stringify({
            schemaVersion: '2',
            jobId: id,
            projectId: id,
            requestedAcceptedCount: 12,
            maxCandidates: 20,
            acceptedCount: 0,
            rejectedCount: 0,
            evaluatedCount: 0,
            nextCandidateIndex: '0',
            revision: 1,
            state: 'queued',
            artifact: null,
            ...patch,
          }),
        );
      },
      async (proxy) => {
        assert.equal((await proxy(request(`/api/jobs/${id}`))).status, 502);
      },
    );
});
test('POST requires matching Origin and valid body/content type', async () =>
  stub(async (proxy, requests) => {
    for (const [headers, body, status] of [
      [{}, '{}', 403],
      [{ origin }, '{}', 415],
      [{ origin, 'content-type': 'application/json' }, '{}', 400],
    ] as const) {
      assert.equal(
        (
          await proxy(
            new Request(`${origin}/api/jobs`, {
              method: 'POST',
              headers: { host: '127.0.0.1:3000', ...headers },
              body,
            }),
          )
        ).status,
        status,
      );
    }
    assert.equal(requests.length, 0);
  }));
test('job submit passes stable idempotency and trace headers', async () => {
  const input = {};
  // Reuse the contract fixture's generation specification.
  const snapshot = JSON.parse(
    await readFile(
      new URL(
        '../../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  Object.assign(input, {
    schemaVersion: '2',
    projectId: id,
    projectRevision: 1,
    generation: snapshot.job.generation,
    pdf: snapshot.job.pdf,
    requestedAcceptedCount: 12,
    maxCandidates: 20,
    execution: { blockSize: 12, concurrency: 1 },
  });
  const raw = JSON.stringify(input),
    trace = '00-' + '1'.repeat(32) + '-' + '2'.repeat(16) + '-01';
  await customStub(
    (req, res) => {
      assert.equal(req.headers['idempotency-key'], 'stable-key-123456');
      assert.equal(req.headers.traceparent, trace);
      let text = '';
      req.setEncoding('utf8');
      req.on('data', (chunk) => (text += chunk));
      req.on('end', () => {
        assert.equal(text, raw);
        res.writeHead(202, { 'content-type': 'application/json' });
        res.end(
          JSON.stringify({
            schemaVersion: '2',
            jobId: id,
            projectId: id,
            requestedAcceptedCount: 12,
            maxCandidates: 20,
            acceptedCount: 0,
            rejectedCount: 0,
            evaluatedCount: 0,
            nextCandidateIndex: '0',
            revision: 1,
            state: 'queued',
            artifact: null,
          }),
        );
      });
    },
    async (proxy) => {
      const response = await proxy(
        new Request(`${origin}/api/jobs`, {
          method: 'POST',
          headers: {
            host: '127.0.0.1:3000',
            origin,
            'content-type': 'application/json',
            'idempotency-key': 'stable-key-123456',
            traceparent: trace,
          },
          body: raw,
        }),
      );
      assert.equal(response.status, 202);
      assert.equal(response.headers.get('traceparent'), trace);
    },
  );
});

test('unsatisfied ranges preserve bounded JSON errors and Content-Range, including HEAD', async () =>
  customStub(
    (req, res) => {
      res.writeHead(416, {
        'content-type': 'application/json',
        'content-range': `bytes */${bytes.length}`,
      });
      res.end(
        req.method === 'HEAD'
          ? undefined
          : JSON.stringify({
              requestId: id,
              code: 'invalid_request',
              message: 'private range details',
            }),
      );
    },
    async (proxy) => {
      for (const method of ['GET', 'HEAD']) {
        const response = await proxy(
          new Request(`${origin}/api/jobs/${id}/artifacts/pdf`, {
            method,
            headers: {
              host: '127.0.0.1:3000',
              'sec-fetch-site': 'none',
              range: 'bytes=100-',
            },
          }),
        );
        assert.equal(response.status, 416);
        assert.equal(
          response.headers.get('content-range'),
          `bytes */${bytes.length}`,
        );
        assert.doesNotMatch(await response.text(), /private range/);
      }
    },
  ));

test('If-Range mismatch is forwarded and safely falls back to a full verified body', async () =>
  customStub(
    (req, res) => {
      assert.equal(req.headers['if-range'], '"older-version"');
      res.writeHead(200, artifactHeaders);
      res.end(bytes);
    },
    async (proxy) => {
      const response = await proxy(
        request(`/api/jobs/${id}/artifacts/pdf`, {
          range: 'bytes=0-2',
          'if-range': '"older-version"',
        }),
      );
      assert.equal(response.status, 200);
      assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);
    },
  ));

test('HEAD artifact errors preserve public status without parsing an absent body', async () => {
  for (const status of [
    400, 401, 403, 404, 409, 413, 415, 418, 429, 500, 502, 503, 504,
  ]) {
    await customStub(
      (req, res) => {
        assert.equal(req.method, 'HEAD');
        res.writeHead(status, {
          'content-type': 'application/json',
          'content-length': '100',
        });
        res.end();
      },
      async (proxy) => {
        const response = await proxy(
          new Request(`${origin}/api/jobs/${id}/artifacts/pdf`, {
            method: 'HEAD',
            headers: { host: '127.0.0.1:3000', 'sec-fetch-site': 'none' },
          }),
        );
        assert.equal(response.status, status === 418 ? 502 : status);
        assert.equal(await response.text(), '');
        assert.equal(response.headers.get('cache-control'), 'no-store');
      },
    );
  }
});
