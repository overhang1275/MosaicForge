import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createEditor, type EditorPorts } from '../src/editor/controller.ts';
import type { Snapshot } from '../src/editor/model.ts';
import type {
  GenerationResponse,
  InspectResponse,
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
    exports: Snapshot[] = [];
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
    get inspections() {
      return inspections;
    },
  };
}
const svg = () => new File(['<svg/>'], 'art.svg', { type: 'image/svg+xml' });
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
    ['error', 'error', 'error'],
  );
  assert.ok(ctx.editor.getSnapshot().cards.every((c) => !c.url));
  assert.equal(ctx.editor.getSnapshot().isGenerating, false);
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
    ['accepted', 'rejected', 'accepted'],
  );
  assert.equal(ctx.editor.getSnapshot().cards[1].url, undefined);
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
