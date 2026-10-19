# Job contract v1 — design, not yet an implemented job runner

Phase3.1 defines an immutable accepted-output target. The generated OpenAPI job specification is a future API contract; phase3.2 does not register its routes. The synchronous editor remains limited to twelve accepted outputs. One million is a contract ceiling to validate incrementally in3.6, not measured capacity.

## Immutable submission

A submission contains an opaque template/project reference, normalized template hash, existing GenerationSpec with count1, requestedAcceptedCount1..1000000, maxCandidates up to10000000, physical PDF layout and operational blockSize1..10000/concurrency1..8. No caller-supplied filesystem path, database, broker or cloud URL is part of the domain. Seed/index remain decimal u64 strings. Candidate budget must cover the accepted target and fit the remaining u64 range.

GenerationSpec.count1 is a reproducible per-candidate configuration, not a way to bypass the existing per-execution 10000 limit. A worker will derive bounded execution specs. Accepted outputs are the first N valid candidates in ascending global index, regardless of block completion order. Block size/concurrency must not change those decisions or artifact identities. Transient transport retries reuse candidate/block identity; they do not consume new indices or increment evaluated counters twice.

Idempotency-Key is required on job submission. Bind the key to authenticated owner/project and a canonical immutable request digest. Same key/same request returns the same job; same key/different request is409. An idempotent block key includes job ID, generation configuration identity and candidate interval. Persist worker claims/results transactionally before confirming progress; durable outbox/reconciliation and implementation belong to3.3/3.4.

## Lifecycle and counters

States: queued → running → succeeded/failed; queued/running → cancelling → cancelled. Terminal states cannot transition back. Success requires exactly requestedAcceptedCount, a completed single PDF with that page count and atomically published PDF+manifest references. No success with a short series, rejected page, partial file or unconfirmed work.

evaluatedCount=acceptedCount+rejectedCount counts confirmed unique candidate results. acceptedCount never exceeds requestedAcceptedCount. A transient failed computation increments neither result counter; explicit failure information and retry budgets are separate. Exhausting maxCandidates or the available index range before N means failed. nextCandidateIndex is the next uncommitted global index, or null on index exhaustion; never serialize u64MAX+1.

Progress has a monotonic revision for reconnect/snapshot ordering. Cancellation is idempotent, prevents new claims and checks between stages; an already running CPU stage may finish. Discard temporary output and do not publish a final artifact on cancelled/failed jobs. A terminal failure can preserve its committed diagnostics, not expose a partial PDF as a finished download.

## PDF and resource bounds

Keep one vector PDF per job and one ordered incremental selection manifest. Do not materialize a million index strings or previews in a status response. Stream accepted index records and PDF output with bounded working sets; paginate previews/history later. Physical dimensions, bleed, exporter version and generation identity participate in document configuration identity. Operational block size/concurrency do not.

An output-byte quota and storage capacity are checked by later project policy before accepting production work. PDF format size/object limits, disk space, total geometry and recovery of a single interrupted document must be measured before announcing a supported job size. The current base64 preview transport is not the large-output artifact transport; later workers publish streamed artifacts and status carries opaque references.

3.3–3.6 must verify duplicate messages, worker/API restarts, stale progress, cancellation races, ordered selection, quota exhaustion and complete publication before these job routes are enabled.
