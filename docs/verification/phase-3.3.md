# Phase3.3 verification — 2026-10-20

Implemented local users/projects and immutable SVG/job revisions in the private
Java API, with a framework-free persistence port and PostgreSQL JDBC adapter.
Versioned schemas, TypeScript types, offline Java schemas and OpenAPI precede
the implementation. This is an API foundation, without project-management UI,
individual login, jobs, queue or workers.

## Executed checks

- All15 contract tests passed; contract generator checks, ESLint and strict
  TypeScript checks passed.
- Docker Maven `spotless:apply verify` against real PostgreSQL completed:
  **11 tests passed, zero failures/errors/skips**. This includes existing PDF
  and gateway tests plus real project HTTP and JDBC integration tests.
- Migration repeated safely; ownership filters, bounded pagination, concurrent
  optimistic revision conflicts, rollback and exact u64 strings were exercised.
- Local launcher test passed: credentials/owner generated once, existing valid
  configuration retained, invalid existing files refused without replacement.
- Production Docker build passed. Its DB-dependent tests are skipped in that
  isolated build; they were executed separately above against `mosaicforge_test`.
- Mounted API smoke created a project, saved two revisions, rejected an injected
  owner, stale revision and counterfeit template hash, then read immutable data.
- PostgreSQL and the application were both restarted. A fresh API smoke read
  the same project with currentRevision2 and original seed
  `18446744073709551615` intact in revision1.
- Process environment checks found DB credentials in Java only, absent from
  Rust/web child processes. The three processes share a container/OS user;
  this is credential minimization, not OS isolation between services.
- Port inspection: web3000 bound to127.0.0.1; PostgreSQL5432 is not published.

Local evidence: `out/projects/full-verify.log`, `contracts.log`,
`docker-build.log`, `create-report.json`, `read-report.json` and `state.json`.
The smoke project is retained as test data. No volume or user data was deleted.

## Limits and review status

PostgreSQL image digest is pinned in Compose; pgJDBC42.7.13 resolves from the
Spring Boot BOM. Driver license was inspected in the packaged artifact
(BSD-2-Clause); PostgreSQL uses the PostgreSQL License. Credentials are stored
only in ignored `state/local-db.env` and runtime configuration, never this report.

Snapshots currently reuse the selected-PDF v2 interactive contract, including
its12-variant bound. This does not implement million-variant jobs. Upload and
JDBC time limits are configured; exhaustive slowloris, disk-full and sustained
load tests were not executed. Login/tenant identity must come from verified
principals before a multiuser deployment; the current owner is server-configured.

Initial local Maven test execution was blocked by sandbox access to its cache;
Docker tests subsequently passed. No successful pre-implementation RED cycle
is claimed for the persistence task. Independent project review was interrupted
by workspace agent credits before a final report; root reviewed the source and
completed the integration/restart checks, but independent3.3 review remains
outstanding. PDF independent review was completed in its separate checkpoint.
