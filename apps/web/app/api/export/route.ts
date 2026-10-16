import { route } from '../../../src/server/route';
export const runtime = 'nodejs';
export const POST = (request: Request) => route(request, 'export');
