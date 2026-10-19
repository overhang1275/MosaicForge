import { compile } from 'json-schema-to-typescript';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { bundleSchema, schemaNames, schemaReference } from './schema.mjs';
import { writeValidators } from './validators.mjs';

const output = new URL(
  '../../contracts/generated/typescript/preview.ts',
  import.meta.url,
);

export async function generatePreviewTypes(override) {
  const root = override ?? {
    title: 'PreviewMessage',
    oneOf: schemaNames.map((name) => ({ $ref: schemaReference(name) })),
  };
  return (
    await compile(bundleSchema('common', root), 'PreviewMessage', {
      bannerComment:
        '/* Generated from contracts/preview/v1 and engine/v1. Run scripts/contracts/generate.mjs. */',
      $refOptions: { resolve: { file: false, http: false } },
      style: { singleQuote: true, semi: true },
    })
  ).replaceAll('\r\n', '\n');
}

export async function checkGenerated(expected) {
  try {
    return (
      (await readFile(output, 'utf8')).replaceAll('\r\n', '\n') === expected
    );
  } catch (error) {
    if (error.code === 'ENOENT') return false;
    throw error;
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const expected = await generatePreviewTypes();
  await writeValidators(process.argv.includes('--check'));
  if (process.argv.includes('--check')) {
    if (!(await checkGenerated(expected))) {
      process.stderr.write(
        'Generated preview types are missing or out of date.\n',
      );
      process.exitCode = 1;
    }
  } else {
    await mkdir(new URL('.', output), { recursive: true });
    await writeFile(output, expected);
  }
}
