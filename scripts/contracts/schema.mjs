import { readFileSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

export const contractRoot = realpathSync(
  fileURLToPath(new URL('../../contracts/', import.meta.url)),
);
export const schemaNames = [
  'inspect-request',
  'inspect-response',
  'generation-request',
  'generation-response',
  'pdf-export-request',
  'pdf-export-response',
  'error',
  'selected-pdf-export-request',
  'selected-pdf-export-response',
];
export const schemaReference = (name) =>
  name.startsWith('selected-')
    ? `../v2/${name}.schema.json`
    : `${name}.schema.json`;

function localReference(source, ref) {
  const [file, fragment = ''] = ref.split('#');
  if (
    file.includes(':') ||
    file.includes('\\') ||
    isAbsolute(file) ||
    (fragment && !fragment.startsWith('/'))
  ) {
    throw new Error('Only local JSON pointer references are supported');
  }
  const path = realpathSync(file ? resolve(dirname(source), file) : source);
  const diff = relative(contractRoot, path);
  if (diff === '..' || diff.startsWith(`..${sep}`) || isAbsolute(diff))
    throw new Error('Schema reference escapes contracts');
  return [path, fragment];
}

export function bundleSchema(name, override) {
  if (![...schemaNames, 'common'].includes(name))
    throw new Error('Unknown schema');
  const source = resolve(contractRoot, 'preview/v1', schemaReference(name));
  const definitions = {};
  const ids = new Map();
  function visit(value, from) {
    if (Array.isArray(value)) return value.map((v) => visit(v, from));
    if (value === null || typeof value !== 'object') return value;
    if (Object.hasOwn(value, '$ref')) {
      const [path, fragment] = localReference(from, value.$ref);
      const id = `${path}#${fragment}`;
      if (!ids.has(id)) {
        let selected = JSON.parse(readFileSync(path, 'utf8'));
        for (const part of fragment.split('/').slice(1)) {
          const key = part.replaceAll('~1', '/').replaceAll('~0', '~');
          if (!selected || !Object.hasOwn(selected, key))
            throw new Error('Schema pointer not found');
          selected = selected[key];
        }
        const used = [...ids.values()];
        const key =
          selected.title && !used.includes(selected.title)
            ? selected.title
            : `Ref${ids.size}`;
        ids.set(id, key);
        definitions[key] = visit(selected, path);
      }
      return { $ref: `#/definitions/${ids.get(id)}` };
    }
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .filter((k) => !['definitions', '$id'].includes(k))
        .map((k) => [k, visit(value[k], from)]),
    );
  }
  const result = visit(
    override ?? JSON.parse(readFileSync(source, 'utf8')),
    source,
  );
  result.definitions = definitions;
  return result;
}
