import type { Operation } from './proxy.ts';
import { createProxy } from './proxy.ts';

export async function route(
  request: Request,
  operation: Operation,
): Promise<Response> {
  try {
    return await createProxy({
      engineUrl: process.env.MOSAICFORGE_ENGINE_URL ?? 'http://127.0.0.1:4771',
      origin: process.env.MOSAICFORGE_WEB_ORIGIN ?? 'http://127.0.0.1:3000',
      token: process.env.MOSAICFORGE_ENGINE_TOKEN ?? '',
    })(request, operation);
  } catch {
    return Response.json(
      {
        requestId: crypto.randomUUID(),
        code: 'engine_unavailable',
        message: 'Revisa la configuración local del motor',
      },
      { status: 502, headers: { 'cache-control': 'no-store' } },
    );
  }
}
