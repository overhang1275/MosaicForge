# Local control API

Java 21 and Spring Boot. Phase 3.2 implements the synchronous operations in
[the generated OpenAPI](../../contracts/openapi/control-v1.json), plus loopback
`GET /health`. It calls Rust through the framework-free `EnginePort` boundary.
Phase3.3 adds PostgreSQL-backed local ownership, projects and immutable revisions
through [the projects OpenAPI](../../contracts/openapi/projects-v1.json).
The configured local owner is derived from server configuration, never request
fields. Individual login, asynchronous jobs and queues remain later work.

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
two PostgreSQL integration tests are explicitly skipped. `scripts/projects-smoke.mjs
create` exercises the mounted API and saves test project identity locally;
after API/DB restarts, run it with `read` to verify durable immutable data.
It never prints credentials. Project management UI and individual-user login
are not included in3.3.
