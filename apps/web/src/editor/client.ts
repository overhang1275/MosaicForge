import type {
  GenerationResponse,
  InspectResponse,
} from '../../../../contracts/generated/typescript/preview';
import type { EditorPorts } from './controller';
function failure(value: unknown, operation?: string): string {
  if (
    operation === 'export-pdf' &&
    value &&
    typeof value === 'object' &&
    'code' in value &&
    value.code === 'payload_too_large'
  ) {
    return 'El PDF supera el límite de exportación de esta vista. Reduce la cantidad de variantes o la complejidad del SVG.';
  }
  return value &&
    typeof value === 'object' &&
    'message' in value &&
    typeof value.message === 'string'
    ? value.message
    : 'No se pudo completar la solicitud al motor.';
}
async function post(
  operation: string,
  body: unknown,
  signal: AbortSignal,
): Promise<Response> {
  const response = await fetch(`/api/${operation}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
    signal,
    cache: 'no-store',
  });
  if (!response.ok) {
    const error: unknown = await response.json().catch(() => null);
    throw new Error(failure(error, operation));
  }
  return response;
}
export const browserPorts: EditorPorts = {
  inspect: async (templateSvg, signal) => {
    const response = await post('inspect', { templateSvg }, signal);
    return (await response.json()) as InspectResponse;
  },
  preview: async (snapshot, signal) => {
    const response = await post('preview', snapshot, signal);
    return (await response.json()) as GenerationResponse;
  },
  export: async (snapshot, format, signal) => {
    const response = await post('export', snapshot, signal);
    if (
      response.headers.get('content-type')?.split(';')[0] !==
      (format === 'png' ? 'image/png' : 'image/svg+xml')
    ) {
      const value: unknown = await response.json().catch(() => null);
      throw new Error(
        value && typeof value === 'object' && 'variant' in value
          ? 'La variante fue rechazada al exportar.'
          : failure(value),
      );
    }
    return await response.blob();
  },
  exportPdf: async (request, signal) => {
    const response = await post('export-pdf', request, signal);
    if (
      response.headers.get('content-type')?.split(';')[0] !== 'application/pdf'
    ) {
      const error: unknown = await response.json().catch(() => null);
      throw new Error(failure(error));
    }
    return await response.blob();
  },
  createUrl: (blob) => URL.createObjectURL(blob),
  revokeUrl: (url) => URL.revokeObjectURL(url),
  save: (blob, name) => {
    const url = URL.createObjectURL(blob),
      link = document.createElement('a');
    link.href = url;
    link.download = name;
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  },
};
