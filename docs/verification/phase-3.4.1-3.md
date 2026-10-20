# Phase 3.4.1–3 verification — 2026-10-20

Implemented the three submodules requested by the user: durable control state,
recoverable CPU blocks and advisory queue/lease coordination. These are internal
components. Public asynchronous submission, final PDF publication and automatic
scheduling remain disabled until 3.4.4/5; reaching the accepted target deliberately
leaves an internal job running without a published artifact.

## Completed checklist

- [x] ~~3.4.1: additive jobs/v2 and worker/v1 contracts, generated types/schemas,
      immutable owned revision snapshots, additive migrations 2/3, canonical idempotency and
      transactional outbox.~~
- [x] ~~3.4.2: authenticated opt-in Rust blocks, shared physical PDF validity,
      durable journal, identity/integrity checks, bounded resources and restart replay.~~
- [x] ~~3.4.3: fenced leases, finite claim attempts, heartbeats, ordered idempotent
      confirmation, recovery, cancellation guards and authenticated Valkey notifications.~~
- [x] ~~Production CPU image built and mounted; private volumes/credentials and
      existing projects checked; verified Git checkpoints saved.~~

## Executed checks

- Contracts: **18 tests passed**, including generator drift and legacy cases;
  strict generated TypeScript, ESLint and formatting passed. The initial new
  contract test failed because jobs/v2 did not exist, then passed after generation.
- Rust implementer executed the full CPU workspace suite, formatting, strict
  Clippy and an explicit preview build successfully. **Seven new block tests**
  cover replay/reopen, changed identity, corruption, quota/time limits, competing
  journal owners, physical bleed/negative space, HTTP guards and exact u64 bounds.
  Existing local-artwork fixture tests remain explicitly ignored. No numerical
  coverage percentage is claimed.
- Final Docker Maven `verify`: **18 tests, zero failures/errors/skips**, using a
  separate real PostgreSQL test database, authenticated Valkey and the optimized
  Rust service. Strict compilation and Spotless passed.
- Real original test.svg, seed 2026, rulesVersion 3, proportional scale 1..4:
  **12 accepted results** across bounded blocks. Recreating the Java store and
  coordinator, then processing duplicate notices, did not duplicate progress.
  The test validates stored block totals and leaves the job awaiting finalization.
- A real slow authenticated request held Rust's shared gate. Java HttpEngine then
  sent the original large SVG and received HTTP 429, rather than an upload error.
  The job remained queued with zero evaluated candidates and zero charged
  attempts; a controlled clock verified delayed eligibility.
- Local launcher: **two tests passed** for independent credentials, retaining
  valid existing files and refusing invalid files without overwriting them.
  Root script syntax/format checks and silent Compose validation passed.
- Production Docker build passed. Its isolated tests skip environment-dependent
  integrations; those tests were executed separately in the full run above.

## Actual process restarts

The mounted application produced a four-candidate block from original test.svg.
After restarting the application container, resending the exact bytes returned
the identical response and digests:

- Block: `2017c03c-9bf7-4018-889a-c4978e1981f9`.
- Request SHA-256:
  `7412418dfde625d2f29c6f3bf8386cf4bc2a6a597cedd68f892bab3e2f88a50f`.
- Records SHA-256:
  `41ebf0480a146d62921b0bad4b18c3a1c17554d25e49a75a8b5635a3a273de5e`.

A second block was interrupted by restarting the application while its request
was active and before its journal file existed. The client failed, not partially
succeeded. Resending the same request after restart produced all four consecutive
records with the expected digest. This exercised a real interrupted process, not
only reopening a repository object.

An authenticated probe stream entry survived an actual Valkey restart. Only that
probe key was deleted afterward. Existing project revision 1 still contained seed
18446744073709551615 and the project retained currentRevision 2 after deployment
and application restarts. No user volume or project was deleted.

Child-process environment checks found the queue password in Java only, and the
journal directory in Rust only. The journal directory was mode 0700. PostgreSQL
and Valkey have no published host ports; the web remains bound to 127.0.0.1:3000.
This is credential minimization inside the existing shared local container, not
OS isolation between Java, Rust, web and their launcher.

## Review, failures and corrections

Independent read-only review found one P2: an occupied worker could exhaust job
attempts without computing a block. HTTP 429 now durably defers eligibility for
two seconds, revokes the old fence and refunds the unexecuted attempt. Controlled
clock tests also check stale heartbeat/confirm/retry behavior and reconciliation
before/after the delay. Scoped re-review found no remaining P1/P2 in that change.

A final audit also caught the missing durable progress revision. Migration 3 adds
it without replacing rows; committed state/progress/lease updates increment it
transactionally within the status contract safe-integer range. Duplicate and
stale no-ops do not increment it. Root observed the missing-column test fail,
then the two real DB tests and final 18-test suite passed. The last revision
correction was manually reviewed by root; independent re-review was unavailable
after the workspace agent credits were exhausted.

The first full-stack run with a debug Rust binary under parallel build load
failed at its first confirmation; it is retained as failed evidence. The optimized
run and final complete verification passed. The first production build failed
Spotless after the last source edit; formatting was applied and the next build
passed. No claim is made that the failed debug run passed or that its precise
transport/timing cause was proven. Initial Java scaffolding also needed compilation
corrections; no full pre-implementation Java RED cycle is claimed.

## Reproduction and limits

Local ignored evidence is under out/durable/: java-full-stack.log (failed),
java-release-stack.log, java-final-verify.log, java-revision-final.log,
java-busy-large.log, revision-red.log, revision-green.log,
docker-build.log (failed), docker-build-final.log, docker-revision-final.log,
deploy.log, block reports,
committed-block-state.json, crash-report.json and valkey-restart-report.json.
Smoke state contains the original SVG and is intentionally not committed.

Use `node scripts/local-compose.mjs up -d --build --wait`. The helper creates a
separate ignored state/local-valkey.env once and preserves existing DB settings.
`node scripts/durable-block-smoke.mjs create` followed by an application restart
and `read` verifies durable replay. `prepare`, `node scripts/durable-crash-smoke.mjs`
and `read` exercise interruption before publication. The crash script intentionally
restarts only the MosaicForge application, so run it in the local test instance.
Java service README documents the environment-gated real stack tests.

Valkey image 8.1.10-alpine is digest-pinned in Compose. Boot resolved Lettuce
7.5.2.RELEASE; its packaged POM license is MIT. Primary sources are recorded in
ADR0012. No new proprietary dependency, Python or CUDA runtime was introduced.

The operational pilot remains **12 accepted, concurrency 1**. Rust journal defaults
are 256 MiB, 4096 directory entries including a publication temporary, at most
4 MiB per stored response and a cooperative 30-second computation budget checked
between candidates and before publication. A single CPU stage is not forcibly
preempted. Disk-full/power-loss chaos and million-page throughput were not tested.
The current PDF format/object limits still require 3.6 work. No final asynchronous
PDF, resumable download, SSE or million-page capability is claimed here.

Verified checkpoints: 11984ea contracts; 87fc13a Rust blocks; 197a365 Java
coordination; 7e8ca55 durable progress revisions. Deployment evidence/checklist is
saved with the final operations checkpoint.
