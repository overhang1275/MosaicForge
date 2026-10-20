# ADR0012 — durable jobs, blocks and advisory coordination

Date: 2026-10-20. User approved submodules 3.4.1, 3.4.2 and 3.4.3.

PostgreSQL owns immutable submissions, ordered progress, outbox and fenced leases.
Valkey Streams carries advisory job IDs; loss or duplication is reconciled from
the database. Neither a queue message nor a Rust block alone can publish success.
The final PDF publisher and public job routes remain deferred to 3.4.4/3.4.5.

Add jobs/v2 instead of reinterpreting jobs/v1.templateId: a submission references
projectId + projectRevision and snapshots the owned immutable SVG. Its generation
templateHash must match the revision. Job idempotency uses canonical sorted JSON
and owner/project/key scope. Operational pilot limits are 12 accepted and one
concurrent block; the schema ceiling of one million is not measured capacity.

Rust exposes a private authenticated block endpoint only when a journal directory
is explicitly configured. It owns the journal volume; Java never reads Rust paths.
Block validity shares the CPU exporter's physical PDF checks, including bleed.
First-N acceptance and candidate budgeting remain ordered and deterministic.

The response requestHash is SHA-256 of the exact UTF-8 request bytes. Java sends
stable sorted-key compact JSON and retains/reconstructs those identical bytes on
retry/reclaim. The same block ID with changed bytes, even whitespace, conflicts.
This avoids inventing a cross-language floating-point canonicalization standard.
recordsSha256 hashes the UTF-8 compact sorted-key JSON array of result records;
its fields contain only decimal index strings, booleans and nullable reasons.
Lease/fencing metadata is not part of the block request, so reclamation can replay
the same immutable work. Stale control tokens cannot confirm results in PostgreSQL.

Valkey uses a separate generated credential in ignored state/local-valkey.env;
existing DB credentials are retained. Only Java and Valkey receive that credential.
No additional host port is published. Rust journals and Valkey have named volumes.
The exact image pulled and pinned is valkey/valkey:8.1.10-alpine with digest
sha256:081c2f5cb575efc901aa80ff9cdbd1ec6a301682fd35e1ebb4b0990a4a4a8507.

Primary references: [Valkey release](https://valkey.io/download/releases/v8-1-10/),
[Valkey license](https://github.com/valkey-io/valkey/blob/8.1/COPYING),
[Lettuce connection timeouts](https://redis.github.io/lettuce/user-guide/connecting-redis/).
Resolved Java client license/version and executed evidence belong in the
verification report. No speed or million-page capacity claim follows from this ADR.
