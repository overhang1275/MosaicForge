import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createEditor, type EditorPorts } from '../src/editor/controller.ts';
import type { Snapshot } from '../src/editor/model.ts';
import type {
  GenerationResponse,
  InspectResponse,
  SelectedPdfExportRequest,
} from '../../../contracts/generated/typescript/preview.ts';

const meta: InspectResponse = {
  requestId: '12345678-1234-1234-1234-123456789abc',
  templateHash: 'a'.repeat(64),
  width: 100,
  height: 100,
  viewBox: [0, 0, 100, 100],
  layerIds: [],
  segmentCount: 5,
  engineVersion: '0.1.0',
  rulesVersion: '1',
};
function accepted(snapshot: Snapshot): GenerationResponse {
  return {
    requestId: meta.requestId,
    variant: {
      schemaVersion: '1',
      index: snapshot.generation.startIndex,
      seed: snapshot.generation.seed,
      templateHash: meta.templateHash,
      configurationHash: 'b'.repeat(64),
      engineVersion: '0.1.0',
      rulesVersion: '1',
      backend: 'cpu',
      device: 'cpu',
      status: 'accepted',
      artifactHash: 'c'.repeat(64),
    },
    artifact: { mimeType: 'image/png', base64: 'AA==' },
  };
}
function setup(overrides: Partial<EditorPorts> = {}) {
  const sent: Snapshot[] = [],
    revoked: string[] = [],
    saved: string[] = [],
    exports: Snapshot[] = [],
    pdfExports: Omit<SelectedPdfExportRequest, 'requestId'>[] = [];
  let inspections = 0,
    urls = 0;
  const ports: EditorPorts = {
    inspect: async () => {
      inspections++;
      return meta;
    },
    preview: async (snap) => {
      sent.push(structuredClone(snap));
      return accepted(snap);
    },
    export: async (snap) => {
      exports.push(snap);
      return new Blob(['artifact']);
    },
    exportPdf: async (request) => {
      pdfExports.push(structuredClone(request));
      return new Blob(['%PDF-test'], { type: 'application/pdf' });
    },
    createUrl: () => `blob:test-${++urls}`,
    revokeUrl: (url) => revoked.push(url),
    save: (_blob, name) => saved.push(name),
    ...overrides,
  };
  const editor = createEditor(ports);
  return {
    editor,
    sent,
    revoked,
    saved,
    exports,
    pdfExports,
    get inspections() {
      return inspections;
    },
  };
}
const svg = () => new File(['<svg/>'], 'art.svg', { type: 'image/svg+xml' });

test('requested twelve means twelve accepted outputs with real deterministic replacement indices', async () => {
  const attempted: string[] = [];
  const ctx = await imported({
    preview: async (snapshot) => {
      const index = snapshot.generation.startIndex;
      attempted.push(index);
      const response = accepted(snapshot);
      if (['2', '7'].includes(index)) {
        response.variant.status = 'rejected';
        response.variant.reason = 'empty_corners';
        delete response.variant.artifactHash;
        delete response.artifact;
      }
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '12' });
  await ctx.editor.generate();
  assert.equal(
    ctx.editor.getSnapshot().cards.filter((card) => card.status === 'accepted')
      .length,
    12,
  );
  assert.deepEqual(
    attempted,
    Array.from({ length: 14 }, (_, index) => String(index)),
  );
  assert.deepEqual(
    ctx.editor.getSnapshot().cards.map((card) => card.index),
    ['0', '1', '3', '4', '5', '6', '8', '9', '10', '11', '12', '13'],
  );
  ctx.editor.updateSettings({ count: '1', seed: '99', startIndex: '5' });
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports[0].job.generation.count, 12);
  assert.equal(ctx.pdfExports[0].job.generation.seed, '2026');
  assert.deepEqual(
    (ctx.pdfExports[0] as unknown as { variantIndices: string[] })
      .variantIndices,
    ['0', '1', '3', '4', '5', '6', '8', '9', '10', '11', '12', '13'],
  );
});

test('impossible rules stop at bounded attempts and never declare success or export a partial PDF', async () => {
  let attempts = 0;
  const ctx = await imported({
    preview: async (snapshot) => {
      attempts++;
      const response = accepted(snapshot);
      response.variant.status = 'rejected';
      response.variant.reason = 'empty_corners';
      delete response.variant.artifactHash;
      delete response.artifact;
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '1' });
  await ctx.editor.generate();
  assert.equal(attempts, 10);
  assert.match(ctx.editor.getSnapshot().error ?? '', /intentos|aceptadas/i);
  assert.doesNotMatch(ctx.editor.getSnapshot().message, /Serie terminada/);
  await ctx.editor.exportPdf();
  assert.equal(ctx.saved.length, 0);
});

test('retry at u64 maximum fails explicitly without wrapping or exporting a short series', async () => {
  const indices: string[] = [];
  const ctx = await imported({
    preview: async (snapshot) => {
      indices.push(snapshot.generation.startIndex);
      const response = accepted(snapshot);
      response.variant.status = 'rejected';
      delete response.artifact;
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '1', startIndex: '18446744073709551615' });
  await ctx.editor.generate();
  assert.deepEqual(indices, ['18446744073709551615']);
  assert.match(ctx.editor.getSnapshot().error ?? '', /u64/);
  assert.doesNotMatch(ctx.editor.getSnapshot().message, /Serie terminada/);
  await ctx.editor.exportPdf();
  assert.equal(ctx.saved.length, 0);
});

test('replacement of the first candidate sets the PDF start to the first accepted real index', async () => {
  const ctx = await imported({
    preview: async (snapshot) => {
      const response = accepted(snapshot);
      if (snapshot.generation.startIndex === '0') {
        response.variant.status = 'rejected';
        delete response.artifact;
      }
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '2' });
  await ctx.editor.generate();
  await ctx.editor.exportPdf();
  assert.deepEqual(ctx.pdfExports[0].variantIndices, ['1', '2']);
  assert.equal(ctx.pdfExports[0].job.generation.startIndex, '1');
});

test('exhausting retries with some accepted outputs still blocks a short PDF', async () => {
  const ctx = await imported({
    preview: async (snapshot) => {
      const response = accepted(snapshot);
      if (snapshot.generation.startIndex !== '0') {
        response.variant.status = 'rejected';
        delete response.artifact;
      }
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '2' });
  await ctx.editor.generate();
  assert.equal(ctx.editor.getSnapshot().cards.length, 1);
  assert.equal(ctx.editor.getSnapshot().cards[0].status, 'accepted');
  assert.equal(ctx.editor.getSnapshot().generationStats?.attempted, 20);
  assert.match(ctx.editor.getSnapshot().message, /incompleta/);
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports.length, 0);
});
test('engine failure stops queued requests and leaves errors without invented previews', async () => {
  let calls = 0;
  const ctx = await imported({
    preview: async () => {
      calls++;
      throw new Error('El motor está ocupado; vuelve a intentarlo');
    },
  });
  ctx.editor.updateSettings({ count: '3' });
  await ctx.editor.generate();
  assert.equal(calls, 1);
  assert.deepEqual(
    ctx.editor.getSnapshot().cards.map((c) => c.status),
    ['error'],
  );
  assert.ok(ctx.editor.getSnapshot().cards.every((c) => !c.url));
  assert.equal(ctx.editor.getSnapshot().isGenerating, false);
});

test('PDF includes the entire immutable generated series as vector SVG, preserving exact u64 indices', async () => {
  const ctx = await imported();
  ctx.editor.updateSettings({
    count: '3',
    seed: '7',
    startIndex: '18446744073709551613',
  });
  await ctx.editor.generate();
  ctx.editor.updateSettings({ count: '12', seed: '99', startIndex: '0' });
  ctx.editor.updatePdfSettings({
    trimWidthMm: '120',
    trimHeightMm: '80',
    bleedMm: '3',
  });
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports.length, 1);
  const request = ctx.pdfExports[0];
  assert.equal(request.templateSvg, '<svg/>');
  assert.equal(request.job.generation.seed, '7');
  assert.equal(request.job.generation.startIndex, '18446744073709551613');
  assert.equal(request.job.generation.count, 3);
  assert.equal(request.job.generation.output.format, 'svg');
  assert.deepEqual(request.job.pdf, {
    exporterVersion: '1',
    trimWidthMm: 120,
    trimHeightMm: 80,
    bleedMm: 3,
  });
  assert.deepEqual(ctx.saved, ['mosaic.pdf']);
  assert.equal(ctx.editor.getSnapshot().isExportingPdf, false);
});

test('PDF refuses a series with rejected or failed variants rather than silently omitting pages', async () => {
  for (const failure of ['rejected', 'error'] as const) {
    const ctx = await imported({
      preview: async (snapshot) => {
        if (failure === 'rejected' || snapshot.generation.startIndex === '1') {
          if (failure === 'error') throw new Error('Error de motor');
          const response = accepted(snapshot);
          response.variant.status = 'rejected';
          delete response.artifact;
          return response;
        }
        return accepted(snapshot);
      },
    });
    ctx.editor.updateSettings({ count: '3' });
    await ctx.editor.generate();
    await ctx.editor.exportPdf();
    assert.equal(ctx.pdfExports.length, 0);
    assert.equal(ctx.saved.length, 0);
    assert.match(
      ctx.editor.getSnapshot().error ?? '',
      /completa|aceptadas|todas/i,
    );
  }
});

test('PDF requires a complete series and cannot export pending pages', async () => {
  let release!: (value: GenerationResponse) => void;
  let pendingSnapshot!: Snapshot;
  const ctx = await imported({
    preview: (snapshot) => {
      pendingSnapshot = snapshot;
      return new Promise((resolve) => {
        release = resolve;
      });
    },
  });
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports.length, 0);
  ctx.editor.updateSettings({ count: '1' });
  const pending = ctx.editor.generate();
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports.length, 0);
  release(accepted(pendingSnapshot));
  await pending;
});

test('PDF physical settings validate before transport and default to 100 mm without bleed', async () => {
  const ctx = await imported();
  assert.deepEqual(ctx.editor.getSnapshot().pdfSettings, {
    trimWidthMm: '100',
    trimHeightMm: '100',
    bleedMm: '0',
  });
  ctx.editor.updateSettings({ count: '1' });
  await ctx.editor.generate();
  for (const patch of [
    { trimWidthMm: '' },
    { trimWidthMm: '0' },
    { trimWidthMm: '0.5' },
    { trimWidthMm: '2001' },
    { trimWidthMm: '100', trimHeightMm: '-1' },
    { trimHeightMm: '100', bleedMm: '-1' },
    { bleedMm: '51' },
  ]) {
    ctx.editor.updatePdfSettings(patch);
    await ctx.editor.exportPdf();
    assert.equal(ctx.pdfExports.length, 0);
    assert.ok(ctx.editor.getSnapshot().error);
  }
  ctx.editor.updatePdfSettings({ bleedMm: '0' });
  await ctx.editor.exportPdf();
  assert.equal(ctx.pdfExports.length, 1);
});

test('reimporting the template aborts an active PDF and never saves its obsolete result', async () => {
  let release!: (value: Blob) => void;
  let signal!: AbortSignal;
  const ctx = await imported({
    exportPdf: (_request, receivedSignal) => {
      signal = receivedSignal;
      return new Promise((resolve) => {
        release = resolve;
      });
    },
  });
  ctx.editor.updateSettings({ count: '1' });
  await ctx.editor.generate();
  const exporting = ctx.editor.exportPdf();
  await ctx.editor.importTemplate();
  assert.equal(signal.aborted, true);
  release(new Blob(['%PDF-test'], { type: 'application/pdf' }));
  await exporting;
  assert.equal(ctx.saved.length, 0);
});

test('PDF server rejection never saves a partial document and permits retry', async () => {
  let attempts = 0;
  const ctx = await imported({
    exportPdf: async () => {
      if (++attempts === 1)
        throw new Error('El trabajo omitiría páginas rechazadas.');
      return new Blob(['%PDF-test'], { type: 'application/pdf' });
    },
  });
  ctx.editor.updateSettings({ count: '1' });
  await ctx.editor.generate();
  await ctx.editor.exportPdf();
  assert.equal(ctx.saved.length, 0);
  assert.match(ctx.editor.getSnapshot().error ?? '', /omitiría/);
  assert.equal(ctx.editor.getSnapshot().isExportingPdf, false);
  await ctx.editor.exportPdf();
  assert.deepEqual(ctx.saved, ['mosaic.pdf']);
});

test('new template aborts PDF export and obsolete transport response cannot trigger a download', async () => {
  let release!: (value: Blob) => void;
  let signal!: AbortSignal;
  const ctx = await imported({
    exportPdf: (_request, receivedSignal) => {
      signal = receivedSignal;
      return new Promise((resolve) => {
        release = resolve;
      });
    },
  });
  ctx.editor.updateSettings({ count: '1' });
  await ctx.editor.generate();
  const exporting = ctx.editor.exportPdf();
  assert.equal(ctx.editor.getSnapshot().isExportingPdf, true);
  await ctx.editor.generate();
  await ctx.editor.download('0', 'svg');
  assert.equal(ctx.sent.length, 1);
  assert.equal(ctx.exports.length, 0);
  ctx.editor.chooseFile(new File(['<svg/>'], 'other.svg'));
  assert.equal(signal.aborted, true);
  release(new Blob(['%PDF-test'], { type: 'application/pdf' }));
  await exporting;
  assert.equal(ctx.saved.length, 0);
  assert.equal(ctx.editor.getSnapshot().isExportingPdf, false);
});
async function imported(overrides: Partial<EditorPorts> = {}) {
  const ctx = setup(overrides);
  ctx.editor.chooseFile(svg());
  await ctx.editor.importTemplate();
  return ctx;
}

test('selection does not send source until explicit import and upload failures preserve settings', async () => {
  const ctx = setup();
  ctx.editor.chooseFile(svg());
  assert.equal(ctx.inspections, 0);
  assert.equal(ctx.editor.getSnapshot().candidateName, 'art.svg');
  await ctx.editor.importTemplate();
  assert.equal(ctx.inspections, 1);
  assert.equal(
    ctx.editor.getSnapshot().template?.metadata.templateHash,
    meta.templateHash,
  );
  ctx.editor.updateSettings({ seed: '18446744073709551615' });
  ctx.editor.chooseFile(new File(['bad'], 'evil.txt'));
  await ctx.editor.importTemplate();
  assert.equal(ctx.inspections, 1);
  assert.ok(ctx.editor.getSnapshot().error);
  assert.equal(ctx.editor.getSnapshot().settings.seed, '18446744073709551615');
});
test('u64 indices remain exact and overflow refuses whole series before request', async () => {
  const ctx = await imported();
  ctx.editor.updateSettings({
    seed: '18446744073709551615',
    startIndex: '18446744073709551614',
    count: '2',
  });
  await ctx.editor.generate();
  assert.deepEqual(
    ctx.sent.map((s) => s.generation.startIndex),
    ['18446744073709551614', '18446744073709551615'],
  );
  assert.equal(ctx.sent[1].generation.seed, '18446744073709551615');
  assert.ok(ctx.sent.every((s) => s.generation.count === 1));
  ctx.editor.updateSettings({ startIndex: '18446744073709551615', count: '2' });
  await ctx.editor.generate();
  assert.equal(ctx.sent.length, 2);
  assert.match(ctx.editor.getSnapshot().error ?? '', /u64|índice/i);
});
test('generation is sequential and rejected cards contain no invented image', async () => {
  let active = 0,
    peak = 0;
  const ctx = await imported({
    preview: async (snap) => {
      active++;
      peak = Math.max(peak, active);
      await new Promise((r) => setTimeout(r, 2));
      active--;
      const response = accepted(snap);
      if (snap.generation.startIndex === '1') {
        response.variant.status = 'rejected';
        response.variant.reason = 'Cobertura insuficiente';
        delete response.variant.artifactHash;
        delete response.artifact;
      }
      return response;
    },
  });
  ctx.editor.updateSettings({ count: '3' });
  await ctx.editor.generate();
  assert.equal(peak, 1);
  assert.deepEqual(
    ctx.editor.getSnapshot().cards.map((c) => c.status),
    ['accepted', 'accepted', 'accepted'],
  );
  assert.deepEqual(
    ctx.editor.getSnapshot().cards.map((card) => card.index),
    ['0', '2', '3'],
  );
  assert.ok(ctx.editor.getSnapshot().cards.every((card) => card.url));
});
test('configuration edits invalidate stale responses even if transport ignores abort', async () => {
  let release!: (value: GenerationResponse) => void;
  let snapshot!: Snapshot;
  let signal!: AbortSignal;
  const ctx = await imported({
    preview: (snap, sig) => {
      snapshot = snap;
      signal = sig;
      return new Promise((r) => {
        release = r;
      });
    },
  });
  ctx.editor.updateSettings({ count: '1' });
  const pending = ctx.editor.generate();
  await Promise.resolve();
  ctx.editor.updateSettings({ seed: '99' });
  assert.equal(signal.aborted, true);
  release(accepted(snapshot));
  await pending;
  assert.ok(
    ctx.editor.getSnapshot().cards.every((c) => c.status !== 'accepted'),
  );
  assert.equal(ctx.editor.getSnapshot().settings.seed, '99');
  assert.equal(ctx.editor.getSnapshot().isGenerating, false);
});
test('download uses immutable result snapshot and URLs are released on replacement/dispose', async () => {
  const ctx = await imported();
  ctx.editor.updateSettings({ count: '1', seed: '7' });
  await ctx.editor.generate();
  ctx.editor.updateSettings({ seed: '8' });
  await ctx.editor.download('0', 'svg');
  assert.equal(ctx.exports[0].generation.seed, '7');
  assert.equal(ctx.exports[0].generation.output.format, 'svg');
  assert.equal(ctx.saved[0], 'variant-0.svg');
  await ctx.editor.generate();
  assert.deepEqual(ctx.revoked, ['blob:test-1']);
  ctx.editor.dispose();
  assert.deepEqual(ctx.revoked, ['blob:test-1', 'blob:test-2']);
});
test('changing template while inspect is pending ignores obsolete result and caps preview count', async () => {
  let finish!: (value: InspectResponse) => void;
  const ctx = setup({
    inspect: () =>
      new Promise((r) => {
        finish = r;
      }),
  });
  ctx.editor.chooseFile(svg());
  const pending = ctx.editor.importTemplate();
  await new Promise((r) => setTimeout(r, 0));
  ctx.editor.chooseFile(new File(['<svg/>'], 'new.svg'));
  finish(meta);
  await pending;
  assert.equal(ctx.editor.getSnapshot().template, null);
  assert.equal(ctx.editor.getSnapshot().candidateName, 'new.svg');
  const other = await imported();
  other.editor.updateSettings({ count: '13' });
  await other.editor.generate();
  assert.equal(other.sent.length, 0);
  assert.ok(other.editor.getSnapshot().error);
});
