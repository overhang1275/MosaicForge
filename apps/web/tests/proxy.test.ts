import assert from 'node:assert/strict';
import { test } from 'node:test';
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
  return new Request(`${origin}/api/preview`, {
    method: 'POST',
    headers: {
      origin,
      host: '127.0.0.1:3000',
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
      new Request(
        'http://localhost:3000/api/inspect',
        request(await fixture('inspect'), { authorization: 'SECRET-CLIENT' }),
      ),
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
