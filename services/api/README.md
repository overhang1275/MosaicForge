# Local control API

Java 21 and Spring Boot. Phase 3.2 implements the synchronous operations in
[the generated OpenAPI](../../contracts/openapi/control-v1.json), plus loopback
`GET /health`. It calls Rust through the framework-free `EnginePort` boundary.
Phase3.3 adds PostgreSQL-backed local ownership, projects and immutable revisions
through [the projects OpenAPI](../../contracts/openapi/projects-v1.json).
The configured local owner is derived from server configuration, never request
fields. Phase3.4 adds opt-in durable jobs, finalization and streaming downloads.
Individual login remains later work.

## Build and run

From this directory, run `./mvnw verify` (PowerShell: `./mvnw.cmd verify`). This runs
contract and real HTTP integration tests, strict compilation and Spotless. Format
changes with `./mvnw spotless:apply` before verification. From the repository root,
regenerate contracts with `node scripts/contracts/control-generate.mjs`, or check
them with `--check`. Generated Java sources and bundled schemas live in contracts/.

The supported integrated launch is the CPU Docker Compose command in the root
README. The container generates distinct service tokens, starts Rust on private
127.0.0.1:4771, Java on private 127.0.0.1:4772 and publishes only web port 3000.

For a manual Java launch, set `MOSAICFORGE_API_TOKEN`,
`MOSAICFORGE_ENGINE_TOKEN` (32–256 printable ASCII characters each), and
`MOSAICFORGE_ENGINE_URL` (canonical HTTP 127.0.0.1 origin). Then run
`java -Xmx384m -jar target/mosaicforge-api.jar`. The API port defaults to 4772 and
can be set with `MOSAICFORGE_API_PORT`. Configure the web server with
`MOSAICFORGE_API_URL` and `MOSAICFORGE_API_TOKEN`. Never use NEXT_PUBLIC variables
for these credentials. Host must match the actual loopback API port; browser
Origin headers are rejected. `/health` carries no secrets and requires no token.

## Limits and behavior

One operation at a time; additional requests get 429 before reading their bodies.
Requests are capped at 32 MiB and SVG text at 5 MiB decoded UTF-8. Responses are
capped at 16 MiB while receiving for legacy JSON operations. The additive
`POST /v3/export-pdf-selection` accepts the v2 selection request and downloads
`application/pdf` with ASCII JSON metadata in `x-mosaicforge-result`. Java spools
the upstream body in bounded chunks to a private temporary file, capped at 1 GiB,
and checks its actual Content-Length, SHA256, PDF signature, complete page counts
and request/selection identity before publishing any PDF bytes. The temporary file
is deleted on success, transport failure, timeout and downstream disconnect.
The v1/v2 JSON endpoints retain their original limits and behavior.
Upload has a 30-second absolute deadline, engine
calls have a 30-second deadline (120 seconds for PDF); a client closing its request
does not promise to interrupt already executing Rust work. The interactive
contracts retain their existing bounds: one variant per call, up to twelve PDF
pages and 512-pixel preview/export dimensions. The binary PDF transport remains a
local synchronous operation; accepting a million in the future job schema is not
measured capacity.

Requests and replies are validated against bundled offline schemas and semantic
rules. The API preserves artifact bytes, request IDs and exact u64 strings. It
rejects malformed replies, mismatched identities or selections, redirects and
unknown upstream error/status combinations. Traceparent is propagated; this is
context propagation, not a full span collector. JSON logs include operation,
request ID, trace context, status and duration, never SVG bodies or credentials.

## Open-source dependency choices

| Component                                        | License                           |
| ------------------------------------------------ | --------------------------------- |
| Spring Boot / Spring Framework / Spring Security | Apache-2.0                        |
| networknt JSON Schema Validator / Jackson        | Apache-2.0                        |
| Apache Tomcat / Maven / Maven Wrapper            | Apache-2.0                        |
| SLF4J                                            | MIT                               |
| Logback                                          | EPL-1.0 or LGPL-2.1               |
| Eclipse Temurin / OpenJDK                        | GPL-2.0 with Classpath Exception  |
| PostgreSQL / pgJDBC42.7.13                       | PostgreSQL License / BSD-2-Clause |

This lists the selected runtime/tooling families, not an exhaustive transitive
license audit. Pinned direct versions are in pom.xml, the wrapper properties and
Dockerfile. No proprietary service, CUDA or Python is required.

## Local projects (3.3)

`GET /v1/me` returns the configured local owner. Create/list/read projects with
`POST/GET /v1/projects` and `GET /v1/projects/{id}`. Save revisions with
`POST /v1/projects/{id}/revisions` and read them with
`GET /v1/projects/{id}/revisions/{number}`. The versioned request/response shapes
are generated from `contracts/projects/v1`; regenerate with
`node scripts/contracts/projects-generate.mjs` and check with `--check`.

Revisions contain a validated selected-PDF snapshot (SVG, job, seed, ordered
indices). Rust inspection verifies the template identity before persistence.
`expectedRevision` prevents lost updates: stale writes return409. Ownership is
always a server-side filter; unknown/foreign projects return404. List pages are
bounded to100 entries. All SQL uses parameters and transactions; JDBC has
connection/socket/statement/lock deadlines. Uploads have an absolute30-second
deadline, a32MiB cap and two admission slots.

Enable through `MOSAICFORGE_PERSISTENCE_ENABLED=true`, `MOSAICFORGE_DB_URL`,
`MOSAICFORGE_DB_USER`, `MOSAICFORGE_DB_PASSWORD` and
`MOSAICFORGE_LOCAL_USER_ID`. The root local launcher configures these and a
durable PostgreSQL volume. DB credentials are withheld from Rust and web child
environments. Keep `state/local-db.env`; deleting it does not reset the database
password and prevents the prior local owner from being selected automatically.

Real DB tests require `MOSAICFORGE_TEST_DB_URL` pointing to a separate test
database and the DB user/password variables above. Without that variable the
PostgreSQL integration tests are explicitly skipped. `scripts/projects-smoke.mjs
create` exercises the mounted API and saves test project identity locally;
after API/DB restarts, run it with `read` to verify durable immutable data.
It never prints credentials. Project management UI and individual-user login
are not included in3.3.

## Durable jobs (3.4)

`JobStore` and `JdbcJobStore` provide authoritative transactions for immutable
revision submission, canonical idempotency, outbox delivery, claims, fenced
heartbeats, ordered block confirmation, finite attempts, cancellation and
reconciliation. Migration 2 is additive and preserves project migration 1.
Migration 3 adds a durable monotonic progress revision, initially zero. Committed
state/progress/lease updates increment it transactionally; duplicate or stale
operations do not. Reopening the store preserves it. Values stay within the
existing status contract's safe integer range.
The pilot accepts at most 12 accepted variants and concurrency 1. Unsupported
requests fail before insertion. Unsigned counters remain decimal strings.

Enable the complete runner with `MOSAICFORGE_JOBS_ENABLED=true`, persistence enabled,
`MOSAICFORGE_VALKEY_URI` and `MOSAICFORGE_VALKEY_PASSWORD`. The source default is off;
missing required dependencies prevent startup. PostgreSQL remains authoritative;
Valkey notifications are advisory. One coordinator polls at most10 database jobs
per cycle, consumes bounded notifications and reconciles cancelled jobs. A broker
outage does not prevent database polling. Shutdown closes the runner and broker
resources.

Migration4 adds an immutable finalization request, stable artifact collection UUID,
sealed metadata and durable cancellation delivery state. Finalization can claim
only after exactly the target has been accepted. Its60-second fenced lease renews
every15seconds; replacement leases replay the same request bytes and artifact ID.
At mostfive finalization attempts use bounded backoff. Committed block references
are read in100-row pages, ordered by candidate start, with at most4095 references.
No accepted-index array is assembled. Worker identity, exact wire hash, page count,
digests and positive size limits are checked before metadata and `succeeded` are
committed together. Cancellation and expired fences cannot publish a result;
terminal states are immutable. Public status maps internal finalization to
`running` and contains bounded counters/references, never SVG or record arrays.

The authenticated public endpoints are `POST /v2/jobs`, `GET /v2/jobs/{id}`,
`POST /v2/jobs/{id}/cancel` and `GET/HEAD /v2/jobs/{id}/artifacts/{pdf|manifest}`.
Submission requires an `Idempotency-Key` of16–128 visible ASCII characters, a
JSON body capped at2MiB and the shared absolute30-second upload deadline. A replay
returns the same job; changed requests return409. Unknown/foreign jobs return404.
Cancellation persists first and retries its private Rust marker when transport or
response validation fails. A succeeded job remains downloadable after a late
cancel request.

Downloads require owned succeeded metadata and stream in8192-byte chunks through
two admission slots, without a whole-file byte array or browser Blob. PDF and
NDJSON members share one collection UUID and have distinct typed routes. Strong
quoted SHA256 ETags, Content-Length and single-byte-range Content-Range values are
checked against persisted metadata; `Range`, `If-Range` and HEAD are supported.
A full download verifies SHA256 and holds its final byte until EOF/length checks
complete. Malformed/unsatisfiable ranges return416 with `Content-Range: bytes */N`
and a sanitized JSON error. Combined collection metadata is capped at1GiB. Rust
serves only sealed, verified collections; temporary output is never downloadable.

Worker429 during block generation delays eligibility by two seconds without
charging an execution failure; old fences cannot renew or confirm after deferral.
Crashed claims consume attempts. Block requests persist as exact canonical UTF-8
JSON bytes; replacement leases reuse the same block ID and bytes. Responses
validate schemas, identity, contiguous indices, acceptance reasons, counters,
stop point and record hash. The local principal and pilot limits remain one
server-configured owner, target<=12 and concurrency1; no individual login or
million-variant capacity is claimed.

`LettuceWorkNotifications` uses a capped advisory Valkey stream; polling reads at
most100 records and acknowledgment deletes the entry. PostgreSQL reconciliation
recovers lost/trimmed notifications. Operations have two-second deadlines and a
bounded client request queue. Lettuce is Boot-managed and MIT licensed, verified
from the resolved artifact POM and its official repository LICENSE. Its official
[connection documentation](https://redis.github.io/lettuce/user-guide/connecting-redis/)
describes client/connection lifecycle and timeouts.

Run real durable integration tests with a separate `MOSAICFORGE_TEST_DB_URL` and
`MOSAICFORGE_DB_USER`/`MOSAICFORGE_DB_PASSWORD`. Set
`MOSAICFORGE_TEST_VALKEY_URI=redis://valkey:6379` and provide
`MOSAICFORGE_VALKEY_PASSWORD` separately for queue tests. Use
`./mvnw -Dtest=JobStoreTest,JobRequestsTest,WorkNotificationsTest test`.
`DurableStackTest` additionally requires `MOSAICFORGE_TEST_ENGINE_URL` (loopback),
`MOSAICFORGE_ENGINE_TOKEN`, and `MOSAICFORGE_TEST_SVG` pointing to test.svg. It
inspects that SVG with real Rust, generates12 accepted candidates at seed2026,
reopens the Java store and verifies totals remain exact after duplicate notices.
Tests delete only their own owner-scoped database rows. Tests without their
required environment variables are explicitly skipped.

## Durable pilot API

Enable `MOSAICFORGE_JOBS_ENABLED=true` together with persistence and the private
`MOSAICFORGE_VALKEY_URI` / `MOSAICFORGE_VALKEY_PASSWORD`. Docker Compose enables
these automatically without publishing a database, queue or API port.
The bounded runner reconciles PostgreSQL independently of advisory notifications.

`POST /v2/jobs` requires a stable16–128 character Idempotency-Key and an owned
project revision. `GET /v2/jobs/{id}` reports bounded counters and artifact IDs;
`POST /v2/jobs/{id}/cancel` is idempotent. `GET/HEAD
/v2/jobs/{id}/artifacts/{pdf|manifest}` serves only complete succeeded artifacts,
with SHA-256 ETags, byte ranges and If-Range. Java streams rather than buffering
the full PDF. Fenced finalization claims retain stable wire bytes/artifact IDs
across restart; success and metadata association commit together.

Current limits remain12 accepted pages and one worker, under the server-configured
local principal. There is no individual login or million-page capacity claim.
See ADR0013 and `docs/verification/phase-3.4.4-6.md` for checks and limitations.
