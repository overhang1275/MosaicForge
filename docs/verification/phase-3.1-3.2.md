# Phases 3.1–3.2 — Java control boundary

Date: 2026-10-19. Base: 6566aa2. Contract checkpoint: 27aabf5.

## Scope

3.1 defines future job contracts: immutable accepted target, candidate budget,
bounded blocks, deterministic global order, idempotency, progress, cancellation,
and atomic publication of one complete vector PDF and selection manifest.
See [the job contract](../job-contract-v1.md) and [ADR 0009](../adr/0009-java-control-boundary.md).
The future job OpenAPI is explicitly unimplemented. One million is a contract
ceiling; no million-variant workload has been executed or claimed.

3.2 introduces Java 21/Spring Boot/Spring Security as the actual synchronous
web → Java → Rust boundary for the five existing operations. It preserves the
interactive limits and N-accepted/PDF-v2 behavior. Rust remains the CPU source of
truth; no generative logic was moved into Java or Next.js. No persistence, users,
queue, distributed workers, CUDA or Python runtime was added.

## Verification checklist

- [x] ~~New OpenAPI/JSON Schemas, generated TS/Java registry/offline schemas reproduce.~~
- [x] ~~Eleven Node contract tests pass: exact u64, bounds, incomplete success, exhausted budget and trace events.~~
- [x] ~~Five Java tests pass, including all shared v1 fixtures, v2 selection cases, strict JSON, trace context and real HTTP robustness checks.~~
- [x] ~~Java strict compilation and Spotless verification pass.~~
- [x] ~~Web formatting, ESLint, strict typing and 34 tests pass.~~
- [x] ~~Affected Rust preview crate passes format, Clippy with denied warnings and 17 tests.~~
- [x] ~~Independent review's slow-upload finding corrected; follow-up static review has no P1/P2 findings.~~
- [x] ~~Production Docker build and health for all three processes.~~
- [x] ~~Browser N-accepted/PDF replay, error/cancellation regressions and trace correlation.~~
- [x] ~~Vector PDF inspection and unchanged artifact byte comparisons.~~
- [x] ~~Verified changes saved in the final Git checkpoint.~~

Java real HTTP checks cover missing/wrong service credentials, browser Origin
rejection, invalid JSON/contract requests, unimplemented job routes, response
correlation, malformed and oversized upstream responses, occupied admission,
engine timeout, safe health and upload deadline with slot recovery. The upload
regression uses a chunked real socket which stops sending before the body ends;
the API returns 504 and serves the next valid request. The production deadline is
30 seconds; tests use a shorter configurable deadline. The first concurrent image
build exposed a test deadline too short under compilation load; verification was
repeated with more generous test-only timings.

The initial Rust trace test and strict-parser/upload regressions failed before
their implementations. The Windows Rust toolchain remains restricted; Rust
checks here executed in Linux Docker. The full unchanged CPU workspace suite was
not rerun for this HTTP-header-only engine change; the affected crate was.

## Deployment and limitations

The deployed image is `sha256:bcc386434fcb606b78ed92541afd4df799500f1610e0167e2c191437bc738990`.
Docker reports healthy; inspection confirms only `127.0.0.1:3000` is published.
Private Java health returned 200/up; an unauthenticated private operation returned 401. Thirteen browser candidate request IDs and trace contexts matched Java log
events. Rust's trace echo and Java's upstream forwarding were also tested.

The public fixture produced twelve accepted previews in thirteen attempts
(index 3 rejected). Its one twelve-page PDF is 442163 bytes, SHA-256
`da1bd88380a67aba18a3284b305b1b7ed90b774f19a784fadd42c258f16d2b32`;
the repeat download is byte-identical. All twelve 100 × 100 mm pages rendered,
with vector path operations and no Image XObjects. First/last renders were
visually inspected. The five accepted-count browser checks and six existing
editor/cancellation checks passed with zero uncaught browser exceptions.

CLI comparison through the new web/Java/Rust boundary retained identical bytes:

| Artifact       | Bytes | SHA-256                                                          |
| -------------- | ----: | ---------------------------------------------------------------- |
| Three-page PDF | 95464 | e8189b87c8ab5f68f42e9a56af171c3bf227eec6759f2d18486414ca25be5176 |
| PNG            | 17868 | af912a9d96996bdbb3f13297e7daad2210c098a1d6f6a7d09bfecd247b5e33f5 |
| SVG            |  1249 | 73b0757d087428cb386450107dbe3a21f7d21720e4b0927e6c36768f121b3450 |

The unchanged private original `test.svg` also produced twelve decoded accepted
previews after fourteen attempts (rejected indices 3 and 7). Its one twelve-page
vector PDF returned HTTP 200, weighs 12266910 bytes and is byte-identical to the
previous reference: SHA-256 `ad133e869db5226645aa7586e3a978dc6751dd6065fc213d068c3f6dcf6f6b92`.
All twelve pages rendered and contain no Image XObjects. The input file retained
SHA-256 `ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8`.
This verifies a real larger document, not general scale or peak memory.

Repeat the two browser scripts using the commands in the earlier accepted-count
and phase-2.6 verification reports, then run `node scripts/accepted-count-verify.mjs`
and `node scripts/editor-parity.mjs`. For trace matching, save `docker logs
mosaicforge-local-mosaicforge-1` to `out/phase31/runtime.log` and the JSON output of
`docker inspect --format '{{json .NetworkSettings.Ports}}'
mosaicforge-local-mosaicforge-1` to `out/phase31/ports.json`; run
`node scripts/control-trace-verify.mjs`. Inspect the twelve-page PDF with
`python docs/qa/inspect-vector-pdf.py out/accepted/twelve.pdf out/phase31/pdf-inspection 12`.

The local image contains three processes with separate generated server-only
Web→API and API→Rust tokens. Java/Rust bind private container loopback 4772/4771;
only web 3000 is published on host loopback. The launcher stops siblings when a
child exits. Health checks web, Java and the Rust listener. This is a local CPU
deployment, not a production multiuser security or distributed scheduling claim.

Request/response caps are 32/16 MiB. Java authenticates before consuming the body,
admits one operation, enforces a bounded nonblocking upload and bounded upstream
subscription, and validates generated schemas plus semantic/correlation rules.
Traceparent/request IDs propagate; JSON logs omit SVG contents and credentials.
This is context propagation, not complete distributed span instrumentation.
Already running Rust compute may continue after a caller or Java deadline ends;
Rust retains its own admission permit until work finishes.

Large jobs need durable state, streamed artifacts, quotas, recovery and measured
format/storage limits in 3.3–3.6. The current base64 preview transport and twelve-
page interactive PDF ceiling are retained. PDF/X, CMYK, Fiery and GPU throughput
are outside this verification.
