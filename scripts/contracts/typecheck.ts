import type {
  GenerationRequest,
  InspectRequest,
  InspectResponse,
  GenerationResponse,
  PreviewError,
  GenerationSpec,
  VariantResult,
} from '../../contracts/generated/typescript/preview.js';

declare const request: GenerationRequest;
const spec: GenerationSpec = request.generation;
const seed: string = spec.seed;
// @ts-expect-error u64 strings must never become JS numbers.
const rounded: number = spec.seed;
declare const response: GenerationResponse;
const variant: VariantResult = response.variant;
const index: string = variant.index;
declare const inspect: InspectRequest;
declare const metadata: InspectResponse;
declare const error: PreviewError;
void [seed, rounded, index, inspect, metadata, error];

import type {
  FinalizeRequest,
  FinalizeResponse,
  ManifestRecord,
} from '../../contracts/generated/typescript/durable.js';
declare const finalize: FinalizeRequest;
declare const sealed: FinalizeResponse;
declare const manifest: ManifestRecord;
const blockHash: string = finalize.blocks[0].recordsSha256;
const sealedBytes: string = sealed.bytes;
// @ts-expect-error Artifact byte counts retain u64 precision in JSON.
const numericBytes: number = sealed.bytes;
void [blockHash, sealedBytes, numericBytes, manifest];
