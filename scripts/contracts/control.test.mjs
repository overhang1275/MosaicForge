import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { validateJob } from './jobs.mjs';
import { execFileSync } from 'node:child_process';

test('new control/job artifacts reproduce without drift', () => {
  execFileSync(
    process.execPath,
    ['scripts/contracts/control-generate.mjs', '--check'],
    { cwd: new URL('../../', import.meta.url), stdio: 'pipe' },
  );
});

test('progress events reject zero trace identity and retain exact exhausted u64', () => {
  const id = '12345678-1234-1234-1234-123456789abc';
  const snapshot = {
    schemaVersion: '1',
    jobId: id,
    projectId: id,
    state: 'failed',
    revision: 5,
    requestedAcceptedCount: 12,
    maxCandidates: 120,
    evaluatedCount: 120,
    acceptedCount: 10,
    rejectedCount: 110,
    nextCandidateIndex: null,
    artifact: null,
  };
  const event = {
    schemaVersion: '1',
    requestId: id,
    traceparent: '00-' + 'a'.repeat(32) + '-' + 'b'.repeat(16) + '-01',
    snapshot,
  };
  assert.equal(validateJob(event, 'event'), true);
  assert.equal(
    validateJob(
      {
        ...event,
        traceparent: '00-' + '0'.repeat(32) + '-' + 'b'.repeat(16) + '-01',
      },
      'event',
    ),
    false,
  );
  assert.equal(
    validateJob(
      { ...event, snapshot: { ...snapshot, state: 'succeeded' } },
      'event',
    ),
    false,
  );
  assert.equal(
    validateJob(
      {
        ...snapshot,
        state: 'cancelled',
        nextCandidateIndex: '18446744073709551615',
      },
      'status',
    ),
    true,
  );
  assert.equal(validateJob({ ...snapshot, state: 'queued' }, 'status'), false);
});

test('job requests preserve u64 precision and bound accepted target, budgets and blocks', async () => {
  const legacy = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  const value = {
    schemaVersion: '1',
    projectId: legacy.requestId,
    templateId: legacy.requestId,
    generation: { ...legacy.job.generation, count: 1 },
    requestedAcceptedCount: 1000000,
    maxCandidates: 10000000,
    pdf: legacy.job.pdf,
    execution: { blockSize: 10000, concurrency: 2 },
  };
  assert.equal(validateJob(value, 'request'), true);
  for (const invalid of [
    { ...value, requestedAcceptedCount: 1000001 },
    { ...value, maxCandidates: 999999 },
    { ...value, generation: { ...value.generation, count: 12 } },
    {
      ...value,
      generation: { ...value.generation, startIndex: '18446744073709551615' },
    },
    { ...value, execution: { blockSize: 10001, concurrency: 2 } },
  ])
    assert.equal(validateJob(invalid, 'request'), false);
  assert.equal(
    validateJob(
      {
        ...value,
        generation: { ...value.generation, startIndex: '18446744073709551615' },
        maxCandidates: 1,
        requestedAcceptedCount: 1,
      },
      'request',
    ),
    true,
  );
});

test('job status never reports short, partial, inconsistent or unpublished success', () => {
  const id = '12345678-1234-1234-1234-123456789abc';
  const status = {
    schemaVersion: '1',
    jobId: id,
    projectId: id,
    state: 'running',
    revision: 4,
    requestedAcceptedCount: 12,
    maxCandidates: 120,
    evaluatedCount: 14,
    acceptedCount: 12,
    rejectedCount: 2,
    nextCandidateIndex: '14',
    artifact: null,
  };
  assert.equal(validateJob(status, 'status'), true);
  assert.equal(validateJob({ ...status, state: 'succeeded' }, 'status'), false);
  const artifact = {
    pdfId: id,
    manifestId: id,
    pageCount: 12,
    sha256: 'a'.repeat(64),
    bytes: '442163',
  };
  assert.equal(
    validateJob({ ...status, state: 'succeeded', artifact }, 'status'),
    true,
  );
  for (const invalid of [
    { ...status, artifact },
    { ...status, state: 'succeeded', artifact: { ...artifact, pageCount: 10 } },
    { ...status, evaluatedCount: 13 },
    { ...status, nextCandidateIndex: '18446744073709551616' },
    { ...status, acceptedCount: 13 },
  ])
    assert.equal(validateJob(invalid, 'status'), false);
});
