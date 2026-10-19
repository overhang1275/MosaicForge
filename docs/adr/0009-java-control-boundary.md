# 0009 — Introduce the Java control boundary incrementally

Date: 2026-10-19. Scope: approved phases 3.1 and 3.2.

## Decision and cost

The demonstrated local editor now targets N accepted variants and exports their explicit indices. Durable projects, permissions and asynchronous recovery belong to a control plane; they do not belong in Next.js or the generative Rust domain. Introduce Java 21, Spring Boot and Spring Security now, before those features, and make the existing editor exercise the new boundary end to end.

Phase 3.1 specifies jobs and lifecycle contracts. Phase 3.2 implements a small synchronous control API for the five existing inspect/preview/export operations. Future job paths are documented separately as unimplemented until durable storage/work execution exists. No fake in-memory job queue or misleading million-variant endpoint is added.

The cost is an extra HTTP call, another runtime, bounded JSON validation and deployment/debugging work. No throughput benefit is claimed. Engine operations and artifact bytes remain authoritative in Rust. A framework-free EnginePort represents that real boundary; no speculative persistence/cache/queue interfaces are introduced.

Contracts live under contracts/. Existing preview v1/v2 models and validators are reused. A generated Java operation registry and bundled offline schemas prevent handwritten wire-model duplication; a JSON envelope at this transparent boundary is validated against the source schemas rather than redefined as Java DTOs. Job TypeScript types are generated from new job schemas. OpenAPI separates implemented control operations from future jobs.

## Local deployment

Extend the existing CPU image with a Java runtime and a private API on loopback port 4772. Rust stays on private loopback 4771, and only web3000 is published on host loopback. This keeps existing fixed-destination/Host protections and avoids a premature service-network redesign. Separate Web→API and API→Rust tokens are generated per start, held only server-side and never logged. Future distributed deployment must revisit networks and credentials in another ADR.

The web BFF addresses Java by default; explicit direct-Rust launch remains a compatibility development mode. Java forwards only generated allowlisted paths. It authenticates before body reads, bounds request/response sizes and deadlines, validates requests/responses and correlation, and propagates W3C traceparent without logging SVG or tokens. Rust echoes valid context for E2E verification. Cancellation does not promise to interrupt an already executing CPU stage.

## Dependency choices

Spring Boot 4.1.1 supports Java21 according to its [system requirements](https://docs.spring.io/spring-boot/system-requirements.html). Pin networknt JSON Schema Validator3.0.8, the [Jackson3-compatible release line](https://github.com/networknt/json-schema-validator), and use embedded draft7 schemas without remote resolution. Maven Wrapper3.3.4 is provided by [Apache Maven](https://maven.apache.org/tools/wrapper/); pin Maven3.9.11. Record resolved dependency licenses with the implementation. Required runtime components remain open source; CUDA/Python are absent.

PostgreSQL/Valkey were selected in the broader roadmap but are not introduced before their actual persistence/queue/cache responsibilities in later submodules.
