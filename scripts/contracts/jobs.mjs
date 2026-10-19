import Ajv from 'ajv';
import { bundleSchema } from './schema.mjs';
import { validSpec } from './validate.mjs';
const ajv = new Ajv({ strict: true });
export const jobSchema = (name) =>
  bundleSchema('common', { $ref: `../../jobs/v1/${name}.schema.json` });
const schemas = Object.fromEntries(
  ['request', 'status', 'event'].map((name) => [
    name,
    ajv.compile(jobSchema(name)),
  ]),
);
const maxU64 = 18446744073709551615n;
const u64 = (value) =>
  typeof value === 'string' &&
  /^(0|[1-9][0-9]{0,19})$/.test(value) &&
  BigInt(value) <= maxU64;
export function validateJob(value, name) {
  if (!schemas[name]?.(value)) return false;
  if (name === 'request')
    return (
      validSpec(value.generation, 'export', 4096) &&
      value.generation.output.format === 'svg' &&
      value.maxCandidates >= value.requestedAcceptedCount &&
      BigInt(value.generation.startIndex) + BigInt(value.maxCandidates) - 1n <=
        maxU64
    );
  if (name === 'event')
    return (
      validateJob(value.snapshot, 'status') &&
      !/^0+$/.test(value.traceparent.slice(3, 35)) &&
      !/^0+$/.test(value.traceparent.slice(36, 52))
    );
  const completed = value.state === 'succeeded';
  return (
    value.evaluatedCount === value.acceptedCount + value.rejectedCount &&
    value.acceptedCount <= value.requestedAcceptedCount &&
    value.evaluatedCount <= value.maxCandidates &&
    value.maxCandidates >= value.requestedAcceptedCount &&
    (value.nextCandidateIndex === null || u64(value.nextCandidateIndex)) &&
    (value.state !== 'queued' || value.evaluatedCount === 0) &&
    completed === (value.artifact !== null) &&
    (!completed ||
      (value.acceptedCount === value.requestedAcceptedCount &&
        value.artifact.pageCount === value.requestedAcceptedCount &&
        u64(value.artifact.bytes) &&
        BigInt(value.artifact.bytes) > 0n))
  );
}
