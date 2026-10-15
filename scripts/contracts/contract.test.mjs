import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readFile } from 'node:fs/promises';
import { validateFixture } from './check.mjs';
import { generatePreviewTypes, checkGenerated } from './generate.mjs';
import { bundleSchema } from './schema.mjs';

const fixtures = new URL(
  '../../contracts/preview/v1/fixtures/',
  import.meta.url,
);
const read = async (name) =>
  JSON.parse(await readFile(new URL(name, fixtures), 'utf8'));

test('shared_fixture_verdicts', async () => {
  const cases = await read('index.json');
  assert.ok(cases.length >= 50);
  for (const c of cases) {
    assert.equal(
      validateFixture(await read(c.file), c.schema, c.operation),
      c.valid,
      c.file,
    );
  }
});

test('u64_roundtrip', async () => {
  const value = await read('request-max-u64.json');
  assert.equal(validateFixture(value, 'generation-request', 'preview'), true);
  const copy = JSON.parse(JSON.stringify(value));
  assert.equal(copy.generation.seed, '18446744073709551615');
  assert.equal(copy.generation.startIndex, '18446744073709551615');
});

test('unicode_limit_counts_bytes', async () => {
  const value = await read('inspect.json');
  value.templateSvg = 'é'.repeat((5 * 1024 * 1024) / 2);
  assert.equal(validateFixture(value, 'inspect-request', 'inspect'), true);
  value.templateSvg += 'é';
  assert.equal(validateFixture(value, 'inspect-request', 'inspect'), false);
});

test('rejects_external_refs', () => {
  for (const $ref of [
    '../../../../AGENTS.md',
    'https://example.com/schema.json',
    'file:///etc/passwd',
  ]) {
    assert.throws(() => bundleSchema('inspect-request', { $ref }));
  }
});

test('generated_output_is_stable', async () => {
  const a = await generatePreviewTypes();
  const b = await generatePreviewTypes();
  assert.equal(a, b);
  assert.match(a, /interface GenerationSpec/);
  assert.equal((a.match(/interface GenerationSpec/g) || []).length, 1);
  assert.equal(await checkGenerated(a), true);
  assert.equal(await checkGenerated(a + '\n// changed'), false);
});

test('schema_change_changes_generated_type', async () => {
  const original = { title: 'Probe', type: 'string', enum: ['a'] };
  assert.notEqual(
    await generatePreviewTypes(original),
    await generatePreviewTypes({ ...original, enum: ['b'] }),
  );
});
