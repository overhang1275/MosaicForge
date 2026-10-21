# Phase 3.4.4–6 verification — 2026-10-20

User authorization: "haz los 3". This completes the existing durable-job design
through artifact publication and API integration. Editor controls/SSE/history
belong to3.5 and are not part of these three submodules.

## Checklist

- [x] ~~Add finalization/cancellation/manifest contracts and generated consumers.~~
- [x] ~~Implement bounded Rust selection, one vector PDF and atomically sealed manifest.~~
- [x] ~~Implement fenced Java finalization, API lifecycle and streamed downloads.~~
- [x] ~~Implement thin Next projects/jobs BFF and native range downloads.~~
- [x] ~~Independent review; fix temporary recovery and HEAD error regressions.~~
- [x] ~~Final production build and real12-page test.svg export.~~
- [x] ~~Actual restart during finalization and after seal before DB association.~~
- [x] ~~Independent vector/PDF render verification.~~
- [x] ~~Final documentation and verified Git checkpoints.~~

## Executed source checks

- Contracts:21/21 Node tests, generator drift checks, ESLint, TypeScript and
  Prettier passed. Checkpoint `f730644`.
- Web:53/53 Node tests, format/lint/typecheck passed. They exercise original-body
  forwarding, Origin/Host/Fetch-Metadata checks, stable idempotency/trace,
  streaming200/206, SHA/length/truncation errors, range/If-Range, cancellation and
  non-success HEAD responses. Final production build is recorded below.
- Rust: initial full CPU suite177 passed,7 existing optional/manual tests ignored;
  formatting/strict Clippy passed. Two subsequent review regressions first failed
  for the intended reasons, then strict format/Clippy and the complete preview
  suite40/40 passed, including8 finalization tests. No CUDA required.
- Java: first focused real PostgreSQL/Valkey suite4 passed,0 skipped. The full
  current-source `mvn verify` against separate real PostgreSQL, authenticated
  Valkey and the optimized Rust binary extracted from the production image
  passed20/20 with0 failures/errors/skips. Spotless and strict compilation passed.
  A preceding full run had2 optional engine tests skipped; the final run enabled
  and passed those tests. Latest malformed-worker-reply regressions also passed.
- Docker release build passed Java verification and Next production build;
  build-only integration tests without service credentials were explicitly
  skipped there, then executed in the real-service verifier above.
- Local launcher tests2/2 passed. Source SVG hash remained unchanged.

## Production recovery evidence

The deployed application image is
`sha256:faa5ea81c3047b214f58189af48af2f99e81f6245c0df90087d1985f7331a313`.
Application, PostgreSQL and Valkey are healthy. Only
`127.0.0.1:3000->3000/tcp` is published;5432/6379 have no host binding.
Existing credentials/volumes were preserved. The existing3.3 project smoke
still reads its two revisions. Credential-presence inspection confirms only Java
receives the database password.

Both jobs used original `test.svg`, seed2026, rulesVersion3, locked aspect ratio,
scale1..4, rotation0..360, target12 and candidate budget120, blocks of4:

1. `cb3519dc-8fba-416b-9c22-8262950e5964`: the in-app browser read `running`,
   revision5 and8 accepted. That temporary tab was closed. The fault probe then
   stopped Rust after the job's own temporary PDF existed, before sealing.
   Restart with a one-second shutdown deadline recovered to `succeeded`,
   revision11 and exactly12 accepted. A fresh browser tab confirmed the completed
   status. No browser tab was needed to keep processing alive.
2. `a018d93b-7902-4332-a080-ba429c4c7542`: the probe stopped Java during its
   own finalization while Rust continued and sealed the collection. Restart
   recovered that orphan through the stable artifact ID/request and committed
   success at revision9. The final PDF was byte-identical to the first job.
3. A separate queued job was cancelled through the public BFF. It reported
   `cancelled`,0 accepted and artifact=null; artifact GET returned404. Concurrent
   cancellation during actual finalization is additionally covered by Rust tests.

Submissions were made by the reproducible Node HTTP client through the public
BFF, with same-origin POST guards and idempotent replay. Browser checks covered
status, disconnect and reconnect; no durable submission UI is claimed in3.4.
Agent-created tabs were closed; the user's editor tab was preserved.

GET/HEAD, a16-byte range206 and an unsatisfiable range416 passed through all
three processes. PDF length/hash, manifest identity/order/completion and selection
hash were independently checked by the client. Native range/If-Range, malformed
replies and truncation cases also passed component HTTP tests.

The one output PDF has12 pages,7,610,962 bytes, SHA-256
`60625d169fa0051a5bfb475b5a9424ba2030211f8897d83c4ee73783c28c87c2`.
Selected indices are0–11. The document configuration hash is
`d697de4fb9064a128c60d9a0e2f2af0d3fd1956833a64872afdc043d7bbeedbf`;
selection SHA-256 is
`ee907aaf04013e990bfec7b85e0624d287f21fdb528424a46d1cdeb5b085d8ba`.
Manifests differ between jobs because they include their job/artifact IDs.

`docs/qa/inspect-vector-pdf.py` independently parsed all pages with pypdf and
rendered all12 with PDFium. Every MediaBox measures100×100mm within0.001mm;
there are0 Image XObjects and actual vector path operators. Representative
first/last renders were visually inspected: varied scale/rotation and original
negative space, without anisotropic stretching. This does not certify arbitrary
artwork readability or a printer/RIP.

## Review and failed attempts

Independent review found two P2 issues: cancelled interrupted temporary
collections could occupy the aggregate quota indefinitely, and the BFF parsed
bodyless HEAD errors as JSON, converting valid errors to502. Rust now scavenges
only validated private owned temporary collections at startup under its exclusive
journal lock. The BFF preserves allowed HEAD error statuses without a body.
Rust also rejects committed suffixes that skip the job's original interval start.

Initial finalization contract test failed because the contract did not exist,
then passed after generation. A first suffix regression had an invalid numeric
u64 fixture; it was corrected to a string before observing the intended failure.
An intentionally interrupted duplicate Rust suite is not counted as passing.
The first native web build's final process result was unavailable after its agent
ended, so it is not claimed as a successful build. The Docker build is the
production completion check. An intermediate Docker rebuild failed Spotless
because it captured Java edits before formatting; the final release rebuild
passed. Windows Playwright CLI access was denied; browser
checks use the in-app browser when possible. No security settings were changed.

## Limits and compatibility

Pilot: <=12 accepted pages, one compute worker,4095 committed block references,
aggregate1GiB artifact quota. Selection does not collect all indices in memory;
the PDF writer still retains page/object offsets and existing object/byte caps.
No million-page, throughput, disk-full or power-loss guarantee is claimed.
Quota tests use bounded writers/sparse files, not an actual exhausted disk.
Cancellation is cooperative between pages and before atomic publication.
Already sealed cancelled collections are inaccessible but need future retention
policy/garbage collection; startup cleanup targets abandoned partial collections.

The output is one vector RGB PDF,100×100mm without bleed for the real fixture.
No PDF/X, CMYK, RIP/Fiery, GPU, ML or individual user authentication is added.
The local server-owned principal is not production multiuser isolation.
Existing legacy PDF/export routes and schemas are retained. The durable document
identity differs from legacy selected-PDF hashing as specified in ADR0013.

Evidence directory (ignored local data): `out/finalize/`; Rust logs under
`services/engine/out/finalize/`. No credentials or source SVG are published in
the report. Original `test.svg` SHA-256 remains
`ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8`.

Key evidence files: `docker-build-release.log`, `java-all-real.log`,
`java-all-real-reports/`, `restart-partial.json`, `restart-sealed.json`,
`partial-recovered-report.json`, `sealed-recovered-report.json`,
`cancel-report.json`, `job.pdf`, `manifest.ndjson`, `inspection/inspection.json`.
Reproduction scripts: `scripts/durable-job-smoke.mjs` and
`scripts/durable-restart-probe.mjs`. The latter deliberately stops a process and
requires an immediate application restart afterwards.

Checkpoints: `f730644` contracts; `3b9e087` Rust; `b97962f` Java/BFF.
Final operational/documentation checkpoint follows these verified modules.
