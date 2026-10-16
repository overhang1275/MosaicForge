import {
  initialSettings,
  type EditorState,
  type Snapshot,
  type EditorSettings,
  type PreviewCard,
  type RangeName,
  type PdfSettings,
} from './model.ts';
import type {
  GenerationResponse,
  InspectResponse,
  PdfExportRequest,
} from '../../../../contracts/generated/typescript/preview.ts';
export interface EditorPorts {
  inspect(svg: string, signal: AbortSignal): Promise<InspectResponse>;
  preview(snapshot: Snapshot, signal: AbortSignal): Promise<GenerationResponse>;
  export(
    snapshot: Snapshot,
    format: 'svg' | 'png',
    signal: AbortSignal,
  ): Promise<Blob>;
  exportPdf(
    request: Omit<PdfExportRequest, 'requestId'>,
    signal: AbortSignal,
  ): Promise<Blob>;
  createUrl(blob: Blob): string;
  revokeUrl(url: string): void;
  save(blob: Blob, name: string): void;
}
const MAX_U64 = 18446744073709551615n;
function u64(text: string): bigint {
  if (!/^(0|[1-9][0-9]{0,19})$/.test(text) || BigInt(text) > MAX_U64)
    throw new Error('Seed e índice deben ser enteros decimales u64.');
  return BigInt(text);
}
function number(text: string): number {
  if (!text.trim() || !Number.isFinite(Number(text)))
    throw new Error('Completa los valores numéricos de las reglas.');
  return Number(text);
}
function snapshotFor(
  svg: string,
  meta: InspectResponse,
  settings: EditorSettings,
  index: string,
): Snapshot {
  const ranges = Object.fromEntries(
    Object.entries(settings.ranges).map(([key, range]) => [
      key,
      { min: number(range.min), max: number(range.max) },
    ]),
  ) as Pick<Snapshot['generation']['rules'], RangeName>;
  const width = number(settings.width),
    height = number(settings.height);
  if (![width, height].every((n) => Number.isInteger(n) && n >= 1 && n <= 512))
    throw new Error('La previsualización admite de 1 a 512 píxeles por eje.');
  return {
    templateSvg: svg,
    generation: {
      schemaVersion: '1',
      templateHash: meta.templateHash,
      seed: settings.seed,
      startIndex: index,
      count: 1,
      engineVersion: meta.engineVersion,
      rulesVersion: meta.rulesVersion,
      rules: {
        ...ranges,
        mirrorX: settings.mirrorX,
        mirrorY: settings.mirrorY,
        shuffleLayers: settings.shuffleLayers,
        distortion: number(settings.distortion),
        ...(settings.palette.trim()
          ? {
              palette: settings.palette
                .split(',')
                .map((c) => c.trim())
                .filter(Boolean),
            }
          : {}),
        ...(settings.cropEnabled
          ? {
              crop: {
                x: number(settings.crop.x),
                y: number(settings.crop.y),
                width: number(settings.crop.width),
                height: number(settings.crop.height),
              },
            }
          : {}),
      },
      output: { format: 'png', width, height, background: settings.background },
    },
  };
}
function message(error: unknown): string {
  return error instanceof Error
    ? error.message
    : 'No se pudo completar la operación.';
}
export function createEditor(ports: EditorPorts) {
  let state: EditorState = {
    candidateName: null,
    template: null,
    isInspecting: false,
    isGenerating: false,
    isExportingPdf: false,
    pdfSettings: { trimWidthMm: '100', trimHeightMm: '100', bleedMm: '0' },
    settings: initialSettings(),
    cards: [],
    selectedIndex: null,
    error: null,
    message: 'Importa un SVG para comenzar.',
    downloading: null,
  };
  const listeners = new Set<() => void>(),
    snapshots = new Map<string, Snapshot>();
  let candidate: File | null = null,
    revision = 0,
    active: AbortController | null = null,
    downloadAbort: AbortController | null = null;
  let seriesSnapshot: Snapshot | null = null;
  function update(patch: Partial<EditorState>) {
    state = { ...state, ...patch };
    listeners.forEach((listener) => listener());
  }
  function clearCards() {
    state.cards.forEach((card) => {
      if (card.url) ports.revokeUrl(card.url);
    });
    snapshots.clear();
    seriesSnapshot = null;
  }
  function invalidate() {
    revision++;
    active?.abort();
    active = null;
    update({
      isInspecting: false,
      isGenerating: false,
      cards: state.cards.map((card) =>
        ['pending', 'generating'].includes(card.status)
          ? {
              ...card,
              status: 'error',
              error: 'Generación cancelada; vuelve a generar.',
            }
          : card,
      ),
    });
  }
  function card(index: string, patch: Partial<PreviewCard>) {
    update({
      cards: state.cards.map((c) =>
        c.index === index ? { ...c, ...patch } : c,
      ),
    });
  }
  const editor = {
    getSnapshot: () => state,
    subscribe: (listener: () => void) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    chooseFile: (file: File | null) => {
      invalidate();
      downloadAbort?.abort();
      candidate = file;
      clearCards();
      update({
        candidateName: file?.name ?? null,
        template: null,
        cards: [],
        selectedIndex: null,
        error: null,
        downloading: null,
        isExportingPdf: false,
        message: file
          ? 'Archivo seleccionado. Pulsa Importar SVG para enviarlo al motor.'
          : 'Importa un SVG para comenzar.',
      });
    },
    importTemplate: async () => {
      if (!candidate) {
        update({ error: 'Selecciona un archivo SVG.' });
        return;
      }
      invalidate();
      downloadAbort?.abort();
      downloadAbort = null;
      update({ downloading: null, isExportingPdf: false });
      const ticket = revision,
        file = candidate;
      const abort = new AbortController();
      active = abort;
      if (
        !/\.svg$/i.test(file.name) ||
        file.size === 0 ||
        file.size > 5 * 1024 * 1024
      ) {
        update({ error: 'Elige un SVG de hasta 5 MiB.', isInspecting: false });
        return;
      }
      update({
        isInspecting: true,
        error: null,
        message: 'Inspeccionando la plantilla…',
      });
      try {
        const svg = await file.text();
        if (ticket !== revision) return;
        const metadata = await ports.inspect(svg, abort.signal);
        if (ticket !== revision) return;
        clearCards();
        update({
          template: { name: file.name, svg, metadata },
          cards: [],
          selectedIndex: null,
          error: null,
          message: 'Plantilla importada. Ajusta las reglas y genera variantes.',
        });
      } catch (error) {
        if (ticket === revision)
          update({
            error: message(error),
            message: 'No se pudo importar la plantilla.',
          });
      } finally {
        if (ticket === revision) {
          active = null;
          update({ isInspecting: false });
        }
      }
    },
    updateSettings: (patch: Partial<EditorSettings>) => {
      invalidate();
      update({
        settings: { ...state.settings, ...structuredClone(patch) },
        error: null,
        message: 'Ajustes modificados. Genera una nueva serie para aplicarlos.',
      });
    },
    updatePdfSettings: (patch: Partial<PdfSettings>) => {
      update({ pdfSettings: { ...state.pdfSettings, ...patch }, error: null });
    },
    generate: async () => {
      if (state.downloading || state.isExportingPdf) {
        update({ error: 'Espera a que termine la descarga.' });
        return;
      }
      if (!state.template) {
        update({ error: 'Primero importa una plantilla SVG.' });
        return;
      }
      let requests: Snapshot[];
      try {
        u64(state.settings.seed);
        const start = u64(state.settings.startIndex),
          count = number(state.settings.count);
        if (!Number.isInteger(count) || count < 1 || count > 12)
          throw new Error('Genera entre 1 y 12 variantes por serie.');
        if (start + BigInt(count - 1) > MAX_U64)
          throw new Error('La serie excede el índice máximo u64.');
        requests = Array.from({ length: count }, (_, i) =>
          snapshotFor(
            state.template!.svg,
            state.template!.metadata,
            state.settings,
            (start + BigInt(i)).toString(),
          ),
        );
      } catch (error) {
        update({ error: message(error) });
        return;
      }
      invalidate();
      const ticket = revision,
        abort = new AbortController();
      active = abort;
      clearCards();
      seriesSnapshot = structuredClone(requests[0]);
      seriesSnapshot.generation.count = requests.length;
      seriesSnapshot.generation.output.format = 'svg';
      requests.forEach((s) =>
        snapshots.set(s.generation.startIndex, structuredClone(s)),
      );
      update({
        cards: requests.map((s) => ({
          index: s.generation.startIndex,
          status: 'pending',
        })),
        selectedIndex: requests[0].generation.startIndex,
        isGenerating: true,
        error: null,
        message: 'Generando variantes…',
      });
      for (const request of requests) {
        if (ticket !== revision) break;
        const index = request.generation.startIndex;
        card(index, { status: 'generating' });
        try {
          const response = await ports.preview(
            structuredClone(request),
            abort.signal,
          );
          if (ticket !== revision) break;
          if (response.variant.status === 'rejected') {
            card(index, { status: 'rejected', result: response.variant });
            continue;
          }
          if (!response.artifact || response.artifact.mimeType !== 'image/png')
            throw new Error(
              'La respuesta no contiene una previsualización PNG.',
            );
          const bytes = Uint8Array.from(atob(response.artifact.base64), (c) =>
            c.charCodeAt(0),
          );
          const url = ports.createUrl(new Blob([bytes], { type: 'image/png' }));
          card(index, { status: 'accepted', result: response.variant, url });
        } catch (error) {
          if (ticket !== revision) break;
          card(index, { status: 'error', error: message(error) });
          update({
            error: message(error),
            cards: state.cards.map((c) =>
              c.status === 'pending'
                ? {
                    ...c,
                    status: 'error',
                    error: 'Serie detenida. Vuelve a generar.',
                  }
                : c,
            ),
          });
          break;
        }
      }
      if (ticket === revision) {
        active = null;
        update({
          isGenerating: false,
          message: 'Serie terminada. Selecciona una variante para revisarla.',
        });
      }
    },
    select: (index: string) => {
      if (state.cards.some((c) => c.index === index))
        update({ selectedIndex: index });
    },
    download: async (index: string, format: 'svg' | 'png') => {
      const saved = snapshots.get(index);
      if (
        !saved ||
        state.cards.find((c) => c.index === index)?.status !== 'accepted' ||
        state.isGenerating ||
        state.isExportingPdf ||
        state.downloading
      )
        return;
      const snapshot = structuredClone(saved);
      snapshot.generation.output.format = format;
      const abort = new AbortController();
      downloadAbort = abort;
      update({ downloading: index, error: null });
      try {
        const blob = await ports.export(snapshot, format, abort.signal);
        if (abort.signal.aborted) return;
        ports.save(blob, `variant-${index}.${format}`);
      } catch (error) {
        if (!abort.signal.aborted) update({ error: message(error) });
      } finally {
        if (downloadAbort === abort) {
          downloadAbort = null;
          update({ downloading: null });
        }
      }
    },
    exportPdf: async () => {
      if (state.downloading || state.isExportingPdf) return;
      if (
        !seriesSnapshot ||
        state.isGenerating ||
        state.isInspecting ||
        state.cards.length !== seriesSnapshot.generation.count ||
        state.cards.some((item) => item.status !== 'accepted')
      ) {
        update({
          error:
            'El PDF requiere una serie completa con todas las variantes aceptadas. Vuelve a generar si hay rechazos o errores.',
        });
        return;
      }
      let request: Omit<PdfExportRequest, 'requestId'>;
      try {
        const trimWidthMm = number(state.pdfSettings.trimWidthMm);
        const trimHeightMm = number(state.pdfSettings.trimHeightMm);
        const bleedMm = number(state.pdfSettings.bleedMm);
        if (
          trimWidthMm < 1 ||
          trimWidthMm > 2000 ||
          trimHeightMm < 1 ||
          trimHeightMm > 2000 ||
          bleedMm < 0 ||
          bleedMm > 50
        )
          throw new Error(
            'El ancho y alto del PDF deben estar entre 1 y 2000 mm; el sangrado, entre 0 y 50 mm.',
          );
        const saved = structuredClone(seriesSnapshot);
        request = {
          templateSvg: saved.templateSvg,
          job: {
            schemaVersion: '1',
            generation: saved.generation,
            pdf: { exporterVersion: '1', trimWidthMm, trimHeightMm, bleedMm },
          },
        };
      } catch (error) {
        update({ error: message(error) });
        return;
      }
      const abort = new AbortController();
      downloadAbort = abort;
      update({
        isExportingPdf: true,
        error: null,
        message: `Preparando PDF vectorial de ${state.cards.length} páginas…`,
      });
      try {
        const blob = await ports.exportPdf(request, abort.signal);
        if (abort.signal.aborted) return;
        ports.save(blob, 'mosaic.pdf');
        update({ message: 'PDF vectorial completo preparado para descargar.' });
      } catch (error) {
        if (!abort.signal.aborted)
          update({
            error: message(error),
            message:
              'No se descargó el PDF. Corrige el error y vuelve a intentarlo.',
          });
      } finally {
        if (downloadAbort === abort) {
          downloadAbort = null;
          update({ isExportingPdf: false });
        }
      }
    },
    cancel: () => {
      invalidate();
      update({
        message:
          'Generación cancelada. El motor puede seguir terminando la operación actual.',
      });
    },
    dispose: () => {
      invalidate();
      downloadAbort?.abort();
      clearCards();
      listeners.clear();
    },
  };
  return editor;
}
