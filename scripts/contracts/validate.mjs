import { createHash } from 'node:crypto';
import validators from '../../contracts/generated/preview-validators.cjs';

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
  if (!['inspect', 'preview', 'export', 'export-pdf'].includes(operation))
    return false;
  if (!validators[schemaName]?.(value)) return false;
  if (schemaName === 'pdf-export-request') {
    const g = value.job.generation;
    return (
      utf8(value.templateSvg) &&
      g.count >= 1 &&
      g.count <= 12 &&
      validSpec({ ...g, count: 1 }, 'export') &&
      g.output.format === 'svg' &&
      BigInt(g.startIndex) + BigInt(g.count) - 1n <= MAX_U64
    );
  }
  if (schemaName === 'pdf-export-response') {
    const d = value.document,
      encoded = value.artifact.base64,
      bytes = Buffer.from(encoded, 'base64');
    return (
      u64(d.seed) &&
      u64(d.startIndex) &&
      d.count >= 1 &&
      d.count <= 12 &&
      BigInt(d.startIndex) + BigInt(d.count) - 1n <= MAX_U64 &&
      d.count === d.requestedCount &&
      d.count === d.acceptedCount &&
      d.count === d.pageCount &&
      d.rejectedCount === 0 &&
      d.status === 'completed' &&
      bytes.subarray(0, 8).toString() === '%PDF-1.7' &&
      bytes.toString('base64') === encoded &&
      createHash('sha256').update(bytes).digest('hex') === d.documentHash
    );
  }
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
