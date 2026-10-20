# ADR0013 — sealed vector job artifacts and streamed downloads

Date: 2026-10-20. User approved 3.4.4–3.4.6 ("haz los 3").

Extend the existing Rust/Java HTTP boundary with versioned finalization,
cancellation and artifact operations. PostgreSQL owns the job state; Rust owns
the private files. No service reads another service's storage paths. No new
language, database, broker or public port is introduced.

Rust consumes ordered committed block references, revalidates their hashes and
identity, and streams exactly the requested number of accepted indices into one
vector multipage PDF. The PDF and an ordered NDJSON manifest are sealed together
by a same-filesystem directory rename. Temporary output is never downloadable.
An immutable artifact UUID identifies the collection; the path also names the
typed member (`pdf` or `manifest`). Callers cannot supply filesystem paths.

Finalization uses a stable artifact UUID and exact request bytes persisted in
PostgreSQL. Fenced leases and heartbeats protect the final state transition. A
crash after sealing but before the database commit replays the same finalization;
the artifact association and `succeeded` transition commit together. Short or
invalid selections cannot succeed. Cancellation persists in both control state
and a Rust marker; Rust checks cancellation between pages and before publishing.

`requestHash` remains SHA-256 of the exact UTF-8 HTTP body. `selectionSha256`
hashes UTF-8 `mosaicforge-selection-v1\n`, followed by each accepted decimal
index and `\n` in page order. The durable `documentConfigurationHash` hashes
Rust's compact sorted-key JSON object containing `schemaVersion: "1"`, the
count-one `job`, `requestedAcceptedCount` and `selectionSha256`. It commits to
the selected variants while remaining independent of block partitioning. It is
a distinct protocol from the legacy selected-PDF configuration hash. Java does
not reproduce Rust floating-point serialization to verify this hash; it checks
request identity, exact counts and typed integrity metadata instead.

Authenticated GET/HEAD downloads expose only sealed files of succeeded owned
jobs. They support a single byte range and If-Range. Each member has a quoted
SHA-256 ETag, exact Content-Length, MIME and Accept-Ranges. Rust validates sealed
integrity before serving; Java validates persisted metadata. Full-body streams
also verify SHA-256 and hold their final byte until completion, so a truncated or
corrupt upstream cannot appear complete. Java translates unsatisfiable ranges
into the shared JSON error with `Content-Range: bytes */total`; Rust's private
416 response has no body. The Next BFF forwards the public contract and streams
downloads without assembling a Blob in server memory.

Operational limits remain explicit: pilot target <=12, one compute worker,
bounded 4095 block references and 1 GiB artifact quota. The selection iterator
does not collect all accepted indices. The existing PDF writer still keeps
page/object offsets and has object/offset limits; million-page capacity and
memory bounds are not established by this change. Larger scale is deferred to
measured work in 3.6. The current local principal is server-configured; this does
not introduce individual user login or production multiuser isolation.
