import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { createProxy } from '../src/server/proxy.ts';

test('cancels upstream response reading when the caller disconnects', async () => {
  let started!: () => void;
  const ready = new Promise<void>((resolve) => {
    started = resolve;
  });
  const srv = await upstream((_req, res) => {
    res.setHeader('content-type', 'application/json');
    res.write('{');
    started();
  });
  try {
    const controller = new AbortController();
    const pending = createProxy({ engineUrl: srv.url, token, origin })(
      request(await fixture('inspect'), {}, controller.signal),
      'inspect',
    );
    await ready;
    controller.abort();
    const response = await pending;
    assert.equal(response.status, 400);
  } finally {
    await srv.close();
  }
});

test('valid busy error is sanitized and returned with new correlation', async () => {
  const srv = await upstream((_req, res) => {
    res.writeHead(429, { 'content-type': 'application/json' });
    res.end(
      JSON.stringify({
        requestId: '00000000-0000-0000-0000-000000000000',
        code: 'busy',
        message: 'SECRET-INTERNAL',
      }),
    );
  });
  try {
    const response = await createProxy({ engineUrl: srv.url, token, origin })(
      request(await fixture('inspect')),
      'inspect',
    );
    assert.equal(response.status, 429);
    const body = await response.json();
    assert.equal(body.code, 'busy');
    assert.notEqual(body.requestId, '00000000-0000-0000-0000-000000000000');
    assert.ok(!body.message.includes('SECRET'));
  } finally {
    await srv.close();
  }
});

const origin = 'http://127.0.0.1:3000';
const token = 'test-only-token-012345678901234567890123';

test('binary PDF streams beyond the old JSON response cap and fails on wrong digest', async () => {
  const original = await fixture('export-pdf');
  const input = {
    ...original,
    schemaVersion: '2',
    variantIndices: ['0', '1', '2'],
  };
  const bytes = Buffer.alloc(17 * 1024 * 1024, 32);
  bytes.write('%PDF-1.7');
  const output = await fixture('accepted-pdf');
  for (const mode of [
    'ok',
    'hash',
    'indices',
    'requestId',
    'pages',
    'length',
  ]) {
    const srv = await upstream((req, res) => {
      let body = '';
      req.on('data', (c) => {
        body += c;
      });
      req.on('end', () => {
        const metadata = {
          schemaVersion: '3',
          requestId: JSON.parse(body).requestId,
          document: {
            ...output.document,
            documentHash:
              mode === 'hash'
                ? '0'.repeat(64)
                : createHash('sha256').update(bytes).digest('hex'),
          },
          variantIndices:
            mode === 'indices' ? ['0', '1', '3'] : ['0', '1', '2'],
        };
        if (mode === 'requestId')
          metadata.requestId = '00000000-0000-0000-0000-000000000000';
        if (mode === 'pages') metadata.document.pageCount = 2;
        res.writeHead(200, {
          'content-type': 'application/pdf',
          'content-length': mode === 'length' ? bytes.length - 1 : bytes.length,
          'x-mosaicforge-result': JSON.stringify(metadata),
        });
        res.end(bytes);
      });
    });
    try {
      const result = await createProxy({ engineUrl: srv.url, token, origin })(
        request(input),
        'export-pdf-download',
      );
      if (['indices', 'requestId', 'pages'].includes(mode))
        assert.equal(result.status, 502);
      else if (mode === 'hash' || mode === 'length')
        await assert.rejects(result.arrayBuffer());
      else {
        assert.equal(result.status, 200);
        assert.deepEqual(Buffer.from(await result.arrayBuffer()), bytes);
      }
    } finally {
      await srv.close();
    }
  }
});

test('generation replies must retain the requested rules version', async () => {
  const input = await fixture('request-max-u64');
  input.generation.rulesVersion = '2';
  input.generation.rules.preserveAspectRatio = true;
  input.generation.rules.scaleY = { ...input.generation.rules.scaleX };
  const output = await fixture('accepted-png');
  const srv = await upstream((req, res) => {
    let bytes = '';
    req.on('data', (chunk) => {
      bytes += chunk;
    });
    req.on('end', () => {
      res.setHeader('content-type', 'application/json');
      res.end(
        JSON.stringify({ ...output, requestId: JSON.parse(bytes).requestId }),
      );
    });
  });
  try {
    const response = await createProxy({ engineUrl: srv.url, token, origin })(
      request(input),
      'preview',
    );
    assert.equal(response.status, 502);
  } finally {
    await srv.close();
  }
});

test('forwards trace context across the Java boundary and preserves timeout errors', async () => {
  const trace = '00-1234567890abcdef1234567890abcdef-1234567890abcdef-01';
  const srv = await upstream((req, res) => {
    assert.equal(req.headers.traceparent, trace);
    res.writeHead(504, { 'content-type': 'application/json' });
    res.end(
      JSON.stringify({
        requestId: '00000000-0000-0000-0000-000000000000',
        code: 'timeout',
        message: 'private internal detail',
      }),
    );
  });
  try {
    const response = await createProxy({ engineUrl: srv.url, token, origin })(
      request(await fixture('inspect'), { traceparent: trace }),
      'inspect',
    );
    assert.equal(response.status, 504);
    assert.equal(response.headers.get('traceparent'), trace);
    assert.equal((await response.json()).code, 'timeout');
  } finally {
    await srv.close();
  }
});

test('selected PDF forwards exact ordered indices to v2 and rejects a different echoed selection', async () => {
  const input = {
    ...(await fixture('export-pdf')),
    schemaVersion: '2',
    variantIndices: ['0', '2', '18446744073709551615'],
  };
  const bytes = Buffer.from('%PDF-1.7\nselected transport fixture');
  const g = input.job.generation;
  for (const mismatch of [false, true]) {
    const srv = await upstream((req, res) => {
      let body = '';
      req.on('data', (chunk) => (body += chunk));
      req.on('end', () => {
        const received = JSON.parse(body);
        assert.equal(req.url, '/v2/export-pdf-selection');
        assert.deepEqual(received.variantIndices, input.variantIndices);
        res.setHeader('content-type', 'application/json');
        res.end(
          JSON.stringify({
            schemaVersion: '2',
            requestId: received.requestId,
            variantIndices: mismatch
              ? ['0', '3', '18446744073709551615']
              : input.variantIndices,
            document: {
              schemaVersion: '1',
              exporterVersion: '1',
              templateHash: g.templateHash,
              seed: g.seed,
              startIndex: g.startIndex,
              count: 3,
              requestedCount: 3,
              acceptedCount: 3,
              rejectedCount: 0,
              pageCount: 3,
              status: 'completed',
              generationConfigurationHash: '0'.repeat(64),
              documentConfigurationHash: '0'.repeat(64),
              documentHash: createHash('sha256').update(bytes).digest('hex'),
            },
            artifact: {
              mimeType: 'application/pdf',
              base64: bytes.toString('base64'),
            },
          }),
        );
      });
    });
    try {
      const response = await createProxy({ engineUrl: srv.url, token, origin })(
        request(input),
        'export-pdf-selection',
      );
      assert.equal(response.status, mismatch ? 502 : 200);
      if (!mismatch)
        assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);
    } finally {
      await srv.close();
    }
  }
});
const fixture = async (name: string) =>
  JSON.parse(
    await readFile(
      new URL(
        `../../../contracts/preview/v1/fixtures/${name}.json`,
        import.meta.url,
      ),
      'utf8',
    ),
  );
function request(
  value: unknown,
  headers: Record<string, string> = {},
  signal?: AbortSignal,
) {
  const requestOrigin = headers.origin ?? origin;
  return new Request(`${requestOrigin}/api/preview`, {
    method: 'POST',
    headers: {
      origin: requestOrigin,
      host: new URL(requestOrigin).host,
      'content-type': 'application/json',
      ...headers,
    },
    body: JSON.stringify(value),
    signal,
  });
}
async function upstream(
  handler: (
    req: import('node:http').IncomingMessage,
    res: import('node:http').ServerResponse,
  ) => void,
) {
  const server = createServer(handler);
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const address = server.address();
  assert.ok(address && typeof address === 'object');
  return {
    url: `http://127.0.0.1:${address.port}`,
    close: () =>
      new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
        server.closeAllConnections();
      }),
  };
}

test('rejects unsafe configuration and private token is mandatory', () => {
  for (const engineUrl of [
    'http://example.com',
    'http://localhost:4771',
    'http://127.0.0.1:4771/path',
    'http://user:pass@127.0.0.1:4771',
    'https://127.0.0.1:4771',
    'http://127.0.0.1:4771?target=x',
  ]) {
    assert.throws(() => createProxy({ engineUrl, token, origin }));
  }
  assert.throws(() =>
    createProxy({ engineUrl: 'http://127.0.0.1:4771', token: '', origin }),
  );
});

test('rejects foreign Origin, media, oversized body and invalid contract before upstream', async () => {
  const proxy = createProxy({ engineUrl: 'http://127.0.0.1:1', token, origin });
  const req = await fixture('inspect');
  assert.equal(
    (await proxy(request(req, { host: 'evil.example' }), 'inspect')).status,
    403,
  );
  assert.equal(
    (await proxy(request(req, { origin: 'https://evil.example' }), 'inspect'))
      .status,
    403,
  );
  assert.equal(
    (
      await proxy(
        request(req, {
          origin: 'http://localhost:3000',
          host: 'localhost:3000',
        }),
        'inspect',
      )
    ).status,
    502,
  );
  assert.equal(
    (
      await proxy(
        request(req, {
          origin: 'http://localhost:3000',
          host: '127.0.0.1:3000',
        }),
        'inspect',
      )
    ).status,
    403,
  );
  assert.equal(
    (await proxy(request(req, { 'content-type': 'text/plain' }), 'inspect'))
      .status,
    415,
  );
  assert.equal(
    (await proxy(request({ ...req, unknown: 1 }), 'inspect')).status,
    400,
  );
  const stream = new ReadableStream({
    start(controller) {
      controller.enqueue(new Uint8Array(32 * 1024 * 1024 + 1));
      controller.close();
    },
  });
  const streamed = new Request(`${origin}/api/inspect`, {
    method: 'POST',
    headers: {
      origin,
      host: '127.0.0.1:3000',
      'content-type': 'application/json',
    },
    body: stream,
    duplex: 'half',
  } as RequestInit);
  assert.equal((await proxy(streamed, 'inspect')).status, 413);
});

test('forwards generated requestId, preserves max u64, strips client secrets and verifies correlation', async () => {
  const response = await fixture('metadata');
  const srv = await upstream((req, res) => {
    assert.equal(req.url, '/v1/inspect');
    assert.equal(req.headers.authorization, `Bearer ${token}`);
    let body = '';
    req.on('data', (chunk) => (body += chunk));
    req.on('end', () => {
      const input = JSON.parse(body);
      assert.notEqual(input.requestId, '12345678-1234-1234-1234-123456789abc');
      res.setHeader('content-type', 'application/json');
      res.end(JSON.stringify({ ...response, requestId: input.requestId }));
    });
  });
  try {
    const res = await createProxy({ engineUrl: srv.url, token, origin })(
      request(await fixture('inspect'), {
        origin: 'http://localhost:3000',
        host: 'localhost:3000',
        authorization: 'SECRET-CLIENT',
      }),
      'inspect',
    );
    assert.equal(res.status, 200);
    assert.equal((await res.json()).templateHash, response.templateHash);
    assert.equal(res.headers.get('cache-control'), 'no-store');
  } finally {
    await srv.close();
  }
});

test('downloads unchanged bytes with safe filename and CSP', async () => {
  const value = await fixture('accepted-svg');
  const srv = await upstream((req, res) => {
    let body = '';
    req.on('data', (c) => (body += c));
    req.on('end', () => {
      const input = JSON.parse(body);
      assert.equal(input.generation.seed, '18446744073709551615');
      assert.equal(input.generation.startIndex, '18446744073709551615');
      value.requestId = input.requestId;
      res.setHeader('content-type', 'application/json');
      res.end(JSON.stringify(value));
    });
  });
  try {
    const res = await createProxy({ engineUrl: srv.url, token, origin })(
      request(await fixture('export-svg')),
      'export',
    );
    assert.equal(res.status, 200);
    assert.deepEqual(
      Buffer.from(await res.arrayBuffer()),
      Buffer.from(value.artifact.base64, 'base64'),
    );
    assert.match(
      res.headers.get('content-disposition') ?? '',
      /^attachment; filename="variant-18446744073709551615.svg"$/,
    );
    assert.match(res.headers.get('content-security-policy') ?? '', /sandbox/);
  } finally {
    await srv.close();
  }
});

test('sanitizes unavailable, timeout, redirects, malformed and oversized upstream', async () => {
  const proxy = createProxy({ engineUrl: 'http://127.0.0.1:1', token, origin });
  assert.equal(
    (await proxy(request(await fixture('inspect')), 'inspect')).status,
    502,
  );
  for (const mode of [
    'timeout',
    'redirect',
    'bad',
    'large',
    'correlation',
    'busy',
  ]) {
    const srv = await upstream((_req, res) => {
      if (mode === 'timeout') return;
      if (mode === 'redirect') {
        res.writeHead(302, { location: 'http://example.com' });
        res.end();
        return;
      }
      res.setHeader('content-type', 'application/json');
      if (mode === 'large') {
        res.end(' '.repeat(16 * 1024 * 1024 + 1));
        return;
      }
      if (mode === 'busy') {
        res.writeHead(429);
        res.end(
          JSON.stringify({
            requestId: '12345678-1234-1234-1234-123456789abc',
            code: 'busy',
            message: 'SECRET-UPSTREAM',
          }),
        );
        return;
      }
      res.end(
        mode === 'correlation'
          ? JSON.stringify({
              requestId: '12345678-1234-1234-1234-123456789abc',
            })
          : 'SECRET-UPSTREAM',
      );
    });
    try {
      const res = await createProxy({
        engineUrl: srv.url,
        token,
        origin,
        timeoutMs: mode === 'timeout' ? 30 : 5000,
      })(request(await fixture('inspect')), 'inspect');
      assert.equal(res.status, mode === 'timeout' ? 504 : 502, mode);
      assert.ok(!JSON.stringify(await res.json()).includes('SECRET-UPSTREAM'));
    } finally {
      await srv.close();
    }
  }
});

test('PDF forwards complete job and downloads one verified document; rejects partial or mismatched responses', async () => {
  const input = await fixture('export-pdf');
  const bytes = Buffer.from('%PDF-1.7\nBFF transport fixture');
  const g = input.job.generation;
  const document = {
    schemaVersion: '1',
    exporterVersion: '1',
    templateHash: g.templateHash,
    seed: g.seed,
    startIndex: g.startIndex,
    count: g.count,
    requestedCount: g.count,
    acceptedCount: g.count,
    rejectedCount: 0,
    pageCount: g.count,
    status: 'completed',
    generationConfigurationHash: '0'.repeat(64),
    documentConfigurationHash: '0'.repeat(64),
    documentHash: createHash('sha256').update(bytes).digest('hex'),
  };
  for (const mode of ['ok', 'partial', 'seed', 'hash', 'mime', 'pages']) {
    const srv = await upstream((req, res) => {
      let body = '';
      req.on('data', (c) => (body += c));
      req.on('end', () => {
        const received = JSON.parse(body);
        assert.deepEqual(received.job, input.job);
        assert.equal(req.url, '/v1/export-pdf');
        const result = {
          requestId: received.requestId,
          document: { ...document },
          artifact: {
            mimeType: 'application/pdf',
            base64: bytes.toString('base64'),
          },
        };
        if (mode === 'partial')
          Object.assign(result.document, {
            acceptedCount: 2,
            rejectedCount: 1,
            pageCount: 2,
          });
        if (mode === 'pages') result.document.pageCount = 2;
        if (mode === 'seed') result.document.seed = '1';
        if (mode === 'hash') result.document.documentHash = '1'.repeat(64);
        if (mode === 'mime') result.artifact.mimeType = 'image/png';
        res.setHeader('content-type', 'application/json');
        res.end(JSON.stringify(result));
      });
    });
    try {
      const res = await createProxy({ engineUrl: srv.url, token, origin })(
        request(input),
        'export-pdf',
      );
      assert.equal(res.status, mode === 'ok' ? 200 : 502, mode);
      if (mode === 'ok') {
        assert.equal(res.headers.get('content-type'), 'application/pdf');
        assert.equal(
          res.headers.get('content-disposition'),
          'attachment; filename="mosaic.pdf"',
        );
        assert.deepEqual(Buffer.from(await res.arrayBuffer()), bytes);
      }
    } finally {
      await srv.close();
    }
  }
});

test('PDF has its own bounded deadline and reports timeout without publishing bytes', async () => {
  const srv = await upstream(() => {});
  try {
    const proxy = createProxy({
      engineUrl: srv.url,
      token,
      origin,
      pdfTimeoutMs: 30,
    });
    const response = await proxy(
      request(await fixture('export-pdf')),
      'export-pdf',
    );
    assert.equal(response.status, 504);
    assert.equal((await response.json()).code, 'timeout');
    assert.throws(() =>
      createProxy({ engineUrl: srv.url, token, origin, pdfTimeoutMs: 120001 }),
    );
  } finally {
    await srv.close();
  }
});
