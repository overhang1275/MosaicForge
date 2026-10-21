import { createDurableProxy } from './durable-proxy.ts';
export async function durableRoute(request: Request): Promise<Response> {
  try {
    return await createDurableProxy({
      engineUrl: process.env.MOSAICFORGE_API_URL ?? 'http://127.0.0.1:4772',
      origin: process.env.MOSAICFORGE_WEB_ORIGIN ?? 'http://127.0.0.1:3000',
      token: process.env.MOSAICFORGE_API_TOKEN ?? '',
    })(request);
  } catch {
    return Response.json(
      {
        requestId: crypto.randomUUID(),
        code: 'engine_unavailable',
        message: 'Revisa la configuración local de la API',
      },
      {
        status: 502,
        headers: {
          'cache-control': 'no-store',
          'x-content-type-options': 'nosniff',
        },
      },
    );
  }
}
