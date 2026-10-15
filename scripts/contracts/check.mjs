import Ajv from 'ajv';
import { createHash } from 'node:crypto';
import { bundleSchema, schemaNames } from './schema.mjs';

const ajv = new Ajv({
  strict: true,
  allErrors: false,
  coerceTypes: false,
  useDefaults: false,
  removeAdditional: false,
});
const validators = new Map(
  schemaNames.map((name) => [name, ajv.compile(bundleSchema(name))]),
);
const MAX_U64 = 18446744073709551615n;
const u64 = (value) =>
  typeof value === 'string' &&
  /^(0|[1-9][0-9]{0,19})$/.test(value) &&
  BigInt(value) <= MAX_U64;
const utf8 = (text) =>
  typeof text === 'string' &&
  text.length > 0 &&
  Buffer.byteLength(text, 'utf8') <= 5 * 1024 * 1024;

function validSpec(spec, operation) {
  if (
    !u64(spec.seed) ||
    !u64(spec.startIndex) ||
    spec.count !== 1 ||
    spec.output.width > 512 ||
    spec.output.height > 512
  )
    return false;
  if (operation === 'preview' && spec.output.format !== 'png') return false;
  if (Buffer.byteLength(JSON.stringify(spec), 'utf8') > 1024 * 1024)
    return false;
  const r = spec.rules;
  for (const key of [
    'translateX',
    'translateY',
    'rotationDegrees',
    'scaleX',
    'scaleY',
    'opacity',
  ]) {
    if (
      ![r[key].min, r[key].max].every(Number.isFinite) ||
      r[key].min > r[key].max
    )
      return false;
  }
  if (
    r.scaleX.min <= 0 ||
    r.scaleY.min <= 0 ||
    r.opacity.min < 0 ||
    r.opacity.max > 1
  )
    return false;
  if (!Number.isFinite(r.distortion) || r.distortion < 0) return false;
  const c = r.crop;
  return (
    !c ||
    [c.x, c.y, c.width, c.height, c.x + c.width, c.y + c.height].every(
      Number.isFinite,
    )
  );
}

function validResponse(value, operation) {
  const v = value.variant;
  if (!u64(v.seed) || !u64(v.index)) return false;
  if (v.status === 'rejected')
    return (
      value.artifact == null &&
      v.artifactHash == null &&
      typeof v.reason === 'string'
    );
  if (v.reason != null || !value.artifact || typeof v.artifactHash !== 'string')
    return false;
  if (operation === 'preview' && value.artifact.mimeType !== 'image/png')
    return false;
  const encoded = value.artifact.base64;
  const bytes = Buffer.from(encoded, 'base64');
  return (
    bytes.toString('base64') === encoded &&
    createHash('sha256').update(bytes).digest('hex') === v.artifactHash
  );
}

export function validateFixture(value, schemaName, operation) {
  if (!['inspect', 'preview', 'export'].includes(operation)) return false;
  if (!validators.get(schemaName)?.(value)) return false;
  if (schemaName === 'inspect-request') return utf8(value.templateSvg);
  if (schemaName === 'generation-request')
    return utf8(value.templateSvg) && validSpec(value.generation, operation);
  if (schemaName === 'generation-response')
    return validResponse(value, operation);
  if (schemaName === 'inspect-response')
    return (
      [value.width, value.height, ...value.viewBox].every(Number.isFinite) &&
      value.viewBox[2] > 0 &&
      value.viewBox[3] > 0
    );
  return true;
}
