import Ajv from 'ajv';
import standaloneCode from 'ajv/dist/standalone/index.js';
import { readFile, writeFile } from 'node:fs/promises';
import { bundleSchema, schemaNames } from './schema.mjs';

const output = new URL(
  '../../contracts/generated/preview-validators.cjs',
  import.meta.url,
);
export function generateValidators() {
  const ajv = new Ajv({
    strict: true,
    code: { source: true },
    coerceTypes: false,
    useDefaults: false,
    removeAdditional: false,
  });
  const exports = {};
  for (const [index, name] of schemaNames.entries()) {
    const id = `preview-${index}`;
    ajv.addSchema(bundleSchema(name), id);
    exports[name] = id;
  }
  // Ajv's Unicode length helper is small and inlined to keep the BFF independent of Ajv at runtime.
  return (
    standaloneCode(ajv, exports).replaceAll(
      'require("ajv/dist/runtime/ucs2length").default',
      '((s) => Array.from(s).length)',
    ) + '\n'
  );
}
export async function writeValidators(check = false) {
  const expected = generateValidators();
  if (check) {
    const actual = await readFile(output, 'utf8').catch(() => '');
    if (actual.replaceAll('\r\n', '\n') !== expected)
      throw new Error('Generated validators are missing or out of date');
  } else await writeFile(output, expected);
}
