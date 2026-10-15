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
