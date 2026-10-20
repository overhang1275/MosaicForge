import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';
import Ajv from 'ajv';
import { bundleSchema } from './schema.mjs';

test('durable contracts identify an immutable project revision and bounded CPU block', () => {
  assert.ok(
    existsSync(
      new URL('../../contracts/jobs/v2/request.schema.json', import.meta.url),
    ),
    'jobs v2 contract must exist',
  );
  const ajv = new Ajv({ strict: true });
  const request = ajv.compile(
    bundleSchema('common', {
      $ref: '../../worker/v1/block-request.schema.json',
    }),
  );
  assert.equal(
    request({
      schemaVersion: '1',
      jobId: '../escape',
      blockId: 'bad',
      candidateCount: 10001,
    }),
    false,
  );
  const job = bundleSchema('common', {
    $ref: '../../jobs/v2/request.schema.json',
  });
  assert.ok(
    job.definitions.DurableJobRequest.required.includes('projectRevision'),
  );
  assert.equal(
    job.definitions.DurableJobRequest.properties.templateId,
    undefined,
  );
});

test('durable contracts regenerate without drift and accept the versioned pilot envelopes', () => {
  execFileSync(
    process.execPath,
    ['scripts/contracts/durable-generate.mjs', '--check'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
  const fixture = JSON.parse(
    readFileSync(
      new URL(
        '../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
    ),
  );
  const ajv = new Ajv({ strict: true });
  const schema = (name) =>
    ajv.compile(bundleSchema('common', { $ref: `../../${name}.schema.json` }));
  const validateJob = schema('jobs/v2/request');
  const job = {
    schemaVersion: '2',
    projectId: fixture.requestId,
    projectRevision: 1,
    generation: { ...fixture.job.generation, count: 1 },
    requestedAcceptedCount: 12,
    maxCandidates: 120,
    pdf: fixture.job.pdf,
    execution: { blockSize: 4, concurrency: 1 },
  };
  assert.ok(validateJob(job));
  assert.equal(validateJob({ ...job, projectRevision: 0 }), false);
  assert.equal(validateJob({ ...job, templateId: fixture.requestId }), false);
  const validateBlock = schema('worker/v1/block-request');
  const block = {
    schemaVersion: '1',
    jobId: fixture.requestId,
    blockId: fixture.requestId,
    templateSvg: fixture.templateSvg,
    job: { ...fixture.job, generation: job.generation },
    startIndex: '18446744073709551615',
    candidateCount: 1,
    acceptedLimit: 1,
  };
  assert.ok(validateBlock(block));
  assert.equal(validateBlock({ ...block, candidateCount: 10001 }), false);
  assert.equal(validateBlock({ ...block, path: '../escape' }), false);
});
