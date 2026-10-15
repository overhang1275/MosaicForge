/* Generated from contracts/preview/v1 and engine/v1. Run scripts/contracts/generate.mjs. */

export type PreviewMessage = InspectRequest | InspectResponse | GenerationRequest | GenerationResponse | PreviewError;
export type RequestId = string;
export type Ref3 = string;
export type Ref10 = string;
export type Ref14 = string;
export type Ref15 = string;
export type Ref17 =
  | 'invalid_request'
  | 'invalid_template'
  | 'unauthorized'
  | 'forbidden'
  | 'payload_too_large'
  | 'unsupported_media_type'
  | 'busy'
  | 'internal_error'
  | 'engine_unavailable'
  | 'timeout';

export interface InspectRequest {
  requestId: RequestId;
  templateSvg: string;
}
export interface InspectResponse {
  engineVersion: '0.1.0';
  height: number;
  /**
   * @maxItems 20000
   */
  layerIds: string[];
  requestId: RequestId;
  rulesVersion: '1';
  segmentCount: number;
  templateHash: Ref3;
  /**
   * @minItems 4
   * @maxItems 4
   */
  viewBox: [number, number, number, number];
  width: number;
}
export interface GenerationRequest {
  generation: GenerationSpec;
  requestId: RequestId;
  templateSvg: string;
}
export interface GenerationSpec {
  count: number;
  engineVersion: '0.1.0';
  output: Ref6;
  rules: Ref7;
  rulesVersion: '1';
  schemaVersion: '1';
  seed: Ref10;
  startIndex: Ref10;
  templateHash: Ref3;
}
export interface Ref6 {
  background: string;
  format: 'svg' | 'png';
  height: number;
  width: number;
}
export interface Ref7 {
  crop?: Ref8 | null;
  distortion: number;
  mirrorX: boolean;
  mirrorY: boolean;
  opacity: Ref9;
  /**
   * @minItems 0
   * @maxItems 256
   */
  palette?: string[];
  rotationDegrees: Ref9;
  scaleX: Ref9;
  scaleY: Ref9;
  shuffleLayers: boolean;
  translateX: Ref9;
  translateY: Ref9;
}
export interface Ref8 {
  height: number;
  width: number;
  x: number;
  y: number;
}
export interface Ref9 {
  max: number;
  min: number;
}
export interface GenerationResponse {
  artifact?: Artifact | null;
  requestId: RequestId;
  variant: VariantResult;
}
export interface Artifact {
  base64: string;
  mimeType: 'image/png' | 'image/svg+xml';
}
export interface VariantResult {
  artifactHash?: Ref14 | null;
  backend: 'cpu';
  configurationHash: Ref14;
  device: string;
  engineVersion: '0.1.0';
  index: Ref15;
  reason?: string | null;
  rulesVersion: '1';
  schemaVersion: '1';
  seed: Ref15;
  status: 'accepted' | 'rejected';
  templateHash: Ref14;
}
export interface PreviewError {
  code: Ref17;
  message: string;
  requestId: RequestId;
}
