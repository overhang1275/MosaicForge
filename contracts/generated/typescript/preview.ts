/* Generated from contracts/preview/v1 and engine/v1. Run scripts/contracts/generate.mjs. */

export type PreviewMessage =
  | InspectRequest
  | InspectResponse
  | GenerationRequest
  | GenerationResponse
  | PdfExportRequest
  | PdfExportResponse
  | PreviewError
  | SelectedPdfExportRequest
  | SelectedPdfExportResponse
  | PdfDownloadResponse;
export type RequestId = string;
export type Ref3 = string;
export type Ref10 = string;
export type Ref14 = string;
export type Ref15 = string;
export type Ref21 =
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
/**
 * @minItems 1
 * @maxItems 12
 */
export type Ref25 =
  | [string]
  | [string, string]
  | [string, string, string]
  | [string, string, string, string]
  | [string, string, string, string, string]
  | [string, string, string, string, string, string]
  | [string, string, string, string, string, string, string]
  | [string, string, string, string, string, string, string, string]
  | [string, string, string, string, string, string, string, string, string]
  | [string, string, string, string, string, string, string, string, string, string]
  | [string, string, string, string, string, string, string, string, string, string, string]
  | [string, string, string, string, string, string, string, string, string, string, string, string];

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
  rulesVersion: '1' | '2';
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
  /**
   * Rules v2 requires an explicit flag. When true, scaleX and scaleY ranges must match and share one sampled factor. Forbidden in rules v1.
   */
  preserveAspectRatio?: boolean;
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
  rulesVersion: '1' | '2';
  schemaVersion: '1';
  seed: Ref15;
  status: 'accepted' | 'rejected';
  templateHash: Ref14;
}
export interface PdfExportRequest {
  job: PdfJobSpec;
  requestId: RequestId;
  templateSvg: string;
}
export interface PdfJobSpec {
  generation: GenerationSpec;
  pdf: PdfLayout;
  schemaVersion: '1';
}
export interface PdfLayout {
  bleedMm: number;
  exporterVersion: '1';
  trimHeightMm: number;
  trimWidthMm: number;
}
export interface PdfExportResponse {
  artifact: {
    base64: string;
    mimeType: 'application/pdf';
  };
  document: PdfDocumentResult;
  requestId: RequestId;
}
export interface PdfDocumentResult {
  acceptedCount: number;
  count: number;
  documentConfigurationHash: string;
  documentHash?: string | null;
  exporterVersion: '1';
  generationConfigurationHash: string;
  pageCount: number;
  rejectedCount: number;
  requestedCount: number;
  schemaVersion: '1';
  seed: string;
  startIndex: string;
  status: 'completed' | 'no_pages';
  templateHash: string;
}
export interface PreviewError {
  code: Ref21;
  message: string;
  requestId: RequestId;
}
export interface SelectedPdfExportRequest {
  job: PdfJobSpec;
  requestId: RequestId;
  schemaVersion: '2';
  templateSvg: string;
  /**
   * @minItems 1
   * @maxItems 12
   */
  variantIndices:
    | [string]
    | [string, string]
    | [string, string, string]
    | [string, string, string, string]
    | [string, string, string, string, string]
    | [string, string, string, string, string, string]
    | [string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string]
    | [string, string, string, string, string, string, string, string, string, string, string, string];
}
export interface SelectedPdfExportResponse {
  artifact: Ref24;
  document: PdfDocumentResult;
  requestId: RequestId;
  schemaVersion: '2';
  variantIndices: Ref25;
}
export interface Ref24 {
  base64: string;
  mimeType: 'application/pdf';
}
export interface PdfDownloadResponse {
  document: PdfDocumentResult;
  requestId: RequestId;
  schemaVersion: '3';
  variantIndices: Ref25;
}
