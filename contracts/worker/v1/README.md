# CPU block protocol v1

Private `POST /internal/v1/blocks`, existing engine bearer token, strict Host and
no browser Origin. Enabled only with an explicit private journal directory.
JSON schemas are authoritative; TypeScript and offline Java schemas are generated
by `node scripts/contracts/durable-generate.mjs`; Rust types are generated at build.

The job generation has count=1 and SVG output. The separate candidate interval
starts at startIndex and contains candidateCount, at most 10000. Overflow is
invalid. Evaluation stops immediately at acceptedLimit; only the processed prefix
is returned. A u64MAX result has nextCandidateIndex=null. IDs are UUIDs, never paths.
Actual templateHash is validated by importing the supplied SVG in Rust.

Each accepted record has reason=null; each rejected record has a nonempty reason.
Indices are consecutive from startIndex. evaluatedCount equals records length and
acceptedCount+rejectedCount. Physical PDF validation uses the same geometry and
coverage policy as the CPU exporter. A transport error is not a rejected candidate.

requestHash is SHA-256 of the exact UTF-8 HTTP body, not a reserialized struct.
Clients send stable sorted-key compact JSON and reuse those exact bytes for each
retry. Same blockId with a different requestHash conflicts, including lexical
changes. Journal recordsSha256 is SHA-256 of the compact sorted-key JSON records
array. No floating-point field participates in this records digest. Both hashes
are lower-case hexadecimal. recordsId is an opaque UUID, never a filesystem path.

The service returns only after durable block publication. Duplicate identical
requests replay the same result after restart. Committed corrupt journals are
not silently replaced. Java must validate response identity, digest, prefix,
counters and next index before applying progress under its current lease token.
Blocks are not completed job artifacts and cannot be downloaded as finished PDFs.
