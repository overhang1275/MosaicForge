/* Generated from contracts/jobs/v1. */

export type JobMessage = JobRequest | JobStatus | JobEvent;
export type GenerationSpec = import('./preview').GenerationSpec;
export type PdfLayout = import('./preview').PdfLayout;
export type Ref9 = string;
export type Ref11 = string;
export type Ref13 = string;

export interface JobRequest {
  execution: {
    blockSize: number;
    concurrency: number;
  };
  generation: GenerationSpec;
  maxCandidates: number;
  pdf: PdfLayout;
  projectId: Ref9;
  requestedAcceptedCount: number;
  schemaVersion: '1';
  templateId: Ref9;
}
export interface JobStatus {
  acceptedCount: number;
  artifact: null | JobArtifact;
  evaluatedCount: number;
  jobId: Ref9;
  maxCandidates: number;
  nextCandidateIndex: Ref11 | null;
  projectId: Ref9;
  rejectedCount: number;
  requestedAcceptedCount: number;
  revision: number;
  schemaVersion: '1';
  state: 'queued' | 'running' | 'cancelling' | 'cancelled' | 'failed' | 'succeeded';
}
export interface JobArtifact {
  bytes: Ref11;
  manifestId: Ref9;
  pageCount: number;
  pdfId: Ref9;
  sha256: string;
}
export interface JobEvent {
  requestId: Ref9;
  schemaVersion: '1';
  snapshot: JobStatus;
  traceparent: Ref13;
}
