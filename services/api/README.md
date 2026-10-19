# Local control API

Java 21 and Spring Boot. Phase 3.2 implements the five synchronous operations in
[the generated OpenAPI](../../contracts/openapi/control-v1.json), plus loopback
`GET /health`. It calls Rust through the framework-free `EnginePort` boundary.
There are no job, database, user or queue implementations yet; the separate
[job contract](../../docs/job-contract-v1.md) describes later work.

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
capped at 16 MiB while receiving. Upload has a 30-second absolute deadline, engine
calls have a 30-second deadline (120 seconds for PDF); a client closing its request
does not promise to interrupt already executing Rust work. The interactive
contracts retain their existing bounds: one variant per call, up to twelve PDF
pages and 512-pixel preview/export dimensions. Large job artifacts need a later
streamed transport; accepting a million in the future job schema is not measured
capacity.

Requests and replies are validated against bundled offline schemas and semantic
rules. The API preserves artifact bytes, request IDs and exact u64 strings. It
rejects malformed replies, mismatched identities or selections, redirects and
unknown upstream error/status combinations. Traceparent is propagated; this is
context propagation, not a full span collector. JSON logs include operation,
request ID, trace context, status and duration, never SVG bodies or credentials.

## Open-source dependency choices

| Component                                        | License                          |
| ------------------------------------------------ | -------------------------------- |
| Spring Boot / Spring Framework / Spring Security | Apache-2.0                       |
| networknt JSON Schema Validator / Jackson        | Apache-2.0                       |
| Apache Tomcat / Maven / Maven Wrapper            | Apache-2.0                       |
| SLF4J                                            | MIT                              |
| Logback                                          | EPL-1.0 or LGPL-2.1              |
| Eclipse Temurin / OpenJDK                        | GPL-2.0 with Classpath Exception |

This lists the selected runtime/tooling families, not an exhaustive transitive
license audit. Pinned direct versions are in pom.xml, the wrapper properties and
Dockerfile. No proprietary service, CUDA or Python is required.
