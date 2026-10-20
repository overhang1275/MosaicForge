import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import Ajv from 'ajv';
import { bundleSchema } from './schema.mjs';

const validate = (name) =>
  new Ajv({ strict: true, validateFormats: false }).compile(
    bundleSchema('common', { $ref: `../../projects/v1/${name}.schema.json` }),
  );
test('project requests reject unknown owner, invalid revision and invalid configuration', async () => {
  const create = validate('create-request');
  assert.equal(create({ schemaVersion: '1', name: 'My project' }), true);
  for (const value of [
    { schemaVersion: '1', name: '' },
    { schemaVersion: '1', name: ' ' },
    { schemaVersion: '1', name: 'a'.repeat(121) },
    { schemaVersion: '1', name: 'My project', ownerId: 'spoof' },
  ])
    assert.equal(create(value), false);
  const snapshot = JSON.parse(
    await readFile(
      new URL(
        '../../contracts/preview/v1/fixtures/export-pdf.json',
        import.meta.url,
      ),
      'utf8',
    ),
  );
  snapshot.schemaVersion = '2';
  snapshot.variantIndices = ['0', '2', '18446744073709551615'];
  const revision = validate('revision-request');
  const value = { schemaVersion: '1', expectedRevision: 0, snapshot };
  assert.equal(revision(value), true);
  assert.equal(revision({ ...value, expectedRevision: -1 }), false);
  assert.equal(revision({ ...value, unexpected: true }), false);
  assert.equal(
    revision({
      ...value,
      snapshot: { ...snapshot, templateSvg: 'x'.repeat(5242881) },
    }),
    false,
  );
});
test('generated project schemas, types and OpenAPI match source contracts', () => {
  const result = spawnSync(
    process.execPath,
    [
      new URL('./projects-generate.mjs', import.meta.url).pathname.replace(
        /^\/([A-Za-z]:)/,
        '$1',
      ),
      '--check',
    ],
    { encoding: 'utf8' },
  );
  assert.equal(result.status, 0, result.stderr);
});
