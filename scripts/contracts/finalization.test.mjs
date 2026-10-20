import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import test from 'node:test';
import Ajv from 'ajv';
import { bundleSchema } from './schema.mjs';

test('finalization requires a complete bounded committed-block selection', () => {
  assert.ok(
    existsSync(
      new URL(
        '../../contracts/worker/v1/finalize-request.schema.json',
        import.meta.url,
      ),
    ),
    'finalizer contract must exist',
  );
  const validate = new Ajv({ strict: true }).compile(
    bundleSchema('common', {
      $ref: '../../worker/v1/finalize-request.schema.json',
    }),
  );
  assert.equal(
    validate({ schemaVersion: '1', blocks: [], path: '../escape' }),
    false,
  );
  const response = bundleSchema('common', {
    $ref: '../../worker/v1/finalize-response.schema.json',
  });
  assert.ok(
    response.definitions.FinalizeResponse.required.includes('manifestSha256'),
  );
  assert.ok(
    response.definitions.FinalizeResponse.required.includes('selectionSha256'),
  );
});

const fixture = JSON.parse(
  readFileSync(
    new URL(
      '../../contracts/preview/v1/fixtures/export-pdf.json',
      import.meta.url,
    ),
  ),
);
const id = fixture.requestId;
const hash = 'a'.repeat(64);
const compile = (name) =>
  new Ajv({ strict: true }).compile(
    bundleSchema('common', { $ref: `../../worker/v1/${name}.schema.json` }),
  );

test('finalizer accepts immutable references and rejects empty, excessive and path-controlled selection', () => {
  const validate = compile('finalize-request');
  const request = {
    schemaVersion: '1',
    jobId: id,
    artifactId: id,
    templateSvg: fixture.templateSvg,
    job: {
      ...fixture.job,
      generation: { ...fixture.job.generation, count: 1 },
    },
    requestedAcceptedCount: 12,
    blocks: [{ blockId: id, requestHash: hash, recordsSha256: hash }],
  };
  assert.ok(validate(request), JSON.stringify(validate.errors));
  for (const invalid of [
    { ...request, blocks: [] },
    { ...request, blocks: Array(4096).fill(request.blocks[0]) },
    { ...request, requestedAcceptedCount: 1000001 },
    { ...request, artifactId: '../escape' },
    { ...request, blocks: [{ ...request.blocks[0], path: '/tmp/secret' }] },
  ])
    assert.equal(validate(invalid), false);
});

test('sealed response and manifest require exact typed integrity fields; cancellation never accepts paths', () => {
  const validate = compile('finalize-response');
  const response = {
    schemaVersion: '1',
    jobId: id,
    artifactId: id,
    requestHash: hash,
    documentConfigurationHash: hash,
    pageCount: 12,
    sha256: hash,
    bytes: '1024',
    manifestSha256: hash,
    manifestBytes: '512',
    selectionSha256: hash,
  };
  assert.ok(validate(response), JSON.stringify(validate.errors));
  assert.equal(validate({ ...response, pageCount: 0 }), false);
  assert.equal(validate({ ...response, bytes: 1024 }), false);
  const incomplete = { ...response };
  delete incomplete.manifestSha256;
  assert.equal(validate(incomplete), false);
  const manifest = compile('manifest-record');
  assert.ok(
    manifest({
      kind: 'variant',
      index: '18446744073709551615',
      pageNumber: 12,
    }),
  );
  assert.ok(
    manifest({
      kind: 'completed',
      pageCount: 12,
      sha256: hash,
      selectionSha256: hash,
    }),
  );
  assert.equal(manifest({ kind: 'completed', pageCount: 12 }), false);
  const cancel = compile('cancel-request');
  assert.ok(cancel({ schemaVersion: '1', jobId: id }));
  assert.equal(
    cancel({ schemaVersion: '1', jobId: id, path: '../escape' }),
    false,
  );
});
