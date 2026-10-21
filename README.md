# MosaicForge

Deterministic SVG variants with a Rust engine and a local web editor.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Stack](https://img.shields.io/badge/stack-Rust%20%2B%20Java%20%2B%20Next.js-orange)
![Platforms](https://img.shields.io/badge/platforms-Windows%20x64%20%7C%20Linux%20Docker-lightgrey)
![Releases](https://img.shields.io/badge/releases-source%20only-lightgrey)

<!-- TODO(owner): Confirm the tagline and add the GitHub repository URL, maintainer username, website, and repository-specific release badge before publication. -->

## About

MosaicForge generates reproducible visual variants from an SVG template. Rust handles transformations, validation, rasterization, and export. A Next.js/TypeScript editor provides previews and controls through a private Java control API in the CPU Docker deployment.

The primary web export is **one vector PDF containing the complete generated series**. Individual SVG and PNG downloads are also available. This is a development preview; no published binary releases or package-manager distribution are configured.

## Features

- Deterministic variants derived from a seed and global variant index.
- Translation, rotation, scaling, mirroring, palette changes, opacity, cropping, layer shuffling, and bounded distortion.
- Proportional scaling by default, with an option to unlink X/Y scales under **Transformaciones**.
- SVG resource limits, coverage validation, and explicit variant rejection.
- Vector multipage PDFs with configurable physical dimensions and bleed; the web defaults to 100 × 100 mm without bleed.
- CLI batch processing, incremental JSONL manifests, artifact hashes, and cooperative cancellation.
- Local web previews and CPU-only Docker deployment with production builds.
- A private Java control API with validated contracts, bounded transport and request tracing.
- PostgreSQL persistence for local projects and immutable SVG/configuration revisions through the private API.

Reproduction requires the same normalized template, configuration, seed, engine version, and backend. Seeds and indices are decimal `u64` strings in JSON to preserve precision.

## Example template

![Example SVG input](docs/fixtures/editor-seed.svg)

An example input template. Generated variants depend on the configured rules, seed, and index.

## Requirements

| Workflow          | Requirements                                                       |
| ----------------- | ------------------------------------------------------------------ |
| Docker            | Docker Engine with Linux containers, Docker Compose v2, Node.js 24 |
| Rust CLI          | Rust/Cargo 1.99 and a platform linker; MSVC build tools on Windows |
| Native web editor | Rust requirements above, Node.js 24.19–24.x, and pnpm 11.25.0      |

Native execution has been tested on Windows x64. The Docker build and local HTTP/PDF flow have been exercised in Linux containers. Other native platforms are unverified. CUDA and Python are not required for the application; optional PDF/profiling checks use Python tools.

## Installation

### Docker

From the repository root:

```sh
node scripts/local-compose.mjs up -d --build --wait
```

Open **http://127.0.0.1:3000**. Use this exact host because the proxy validates `Host` and `Origin`. The first build downloads dependencies; later builds reuse Docker caches.

The image runs web → Java → Rust with distinct server-only service credentials.
Only the web port is published; Java 4772 and Rust 4771 stay on container loopback.
See [the Java service](services/api/README.md) and [durable job API](contracts/openapi/jobs-v2.json).
The local launcher creates `state/local-db.env` once with a random database
password and stable local owner ID; it never prints or overwrites existing
credentials. Keep this ignored file and the PostgreSQL volume when updating.
It also creates `state/local-valkey.env` once for the separate internal queue
credential. Keep that file, the Valkey volume and the private engine journal volume.
PostgreSQL and Valkey have no host ports. Node24 is required for the helper; its equivalent
Compose command is `docker compose --env-file state/local-db.env --env-file state/local-valkey.env -f deploy/compose/local.yml`.
The durable pilot enables asynchronous submission and sealed PDF publication
through the BFF API. Durable editor controls remain the next phase.
Individual user login remains a later phase.

If port 3000 is occupied, choose another port without stopping unrelated services:

```sh
MOSAICFORGE_HTTP_PORT=3001 node scripts/local-compose.mjs up -d --build --wait
```

In PowerShell, set `$env:MOSAICFORGE_HTTP_PORT = '3001'` before running Compose. Keep the same value for subsequent commands. The fixed Compose project name means changing the port replaces the existing instance.

### Build from source

```sh
cargo build --manifest-path services/engine/Cargo.toml --locked -p mosaicforge-cli -p mosaicforge-preview
pnpm --dir apps/web install --frozen-lockfile --ignore-scripts
pnpm --dir apps/web build
node scripts/start-preview.mjs
```

This compatibility development launcher talks directly to Rust. The integrated Java path is exercised by Docker. The launcher requires these debug Rust binaries and the standalone web build. It generates a temporary private token in memory and starts both processes on loopback. Press Ctrl+C to stop them. Stop a native standalone server before rebuilding its output on Windows to avoid file-lock errors.

## Usage

### Web editor

1. Select an SVG and click **Importar plantilla**.
2. Set the seed, starting index, sample count, and transformation rules.
3. Generate previews and inspect accepted or rejected variants.
4. Click **Descargar PDF multipágina** to export the complete saved series.

The interface currently uses Spanish labels. PDF export requires every sample in the saved series to be accepted; an error never downloads a partial document. Editing controls afterward does not change that saved series. SVG/PNG downloads are secondary options.

The editor uses a binary PDF download instead of the legacy JSON/base64 transport.
Rust and Java prepare/verify the completed document on private temporary disk;
Next streams it with length and SHA256 checks. The current local disk quota is
1 GiB per PDF, with a 120-second request deadline and the existing 12-sample UI.
The current editor still receives a Blob before saving. The separate durable
API already supports native streamed artifact downloads, while its editor UI
and large-scale capacity validation remain future work. This does not establish
million-page capacity or reduce the intrinsic size of unique vector artwork.

The editor supports **1–12 samples per series** and previews up to **512 × 512 px**. The CLI supports **1–10,000 variants per job**. One million variants is a roadmap objective, not a verified current capability.

The web sample count targets accepted variants. Rejected candidates are replaced in deterministic index order, with up to ten attempts per requested variant. If the target cannot be completed, the series fails explicitly and cannot be exported as a complete PDF. The CLI v1 continues to count candidates.

### CLI

Build an optimized CLI:

```sh
cargo build --manifest-path services/engine/Cargo.toml --release --locked -p mosaicforge-cli
./services/engine/target/release/mosaicforge --help
./services/engine/target/release/mosaicforge inspect --template services/engine/tests/fixtures/basic.svg
```

On Windows, use `mosaicforge.exe`. `inspect` returns the **normalized** template hash. Copy the [generation example](services/engine/tests/fixtures/identity.json), replace `templateHash` with that value, and adjust its settings. Supply your SVG and configuration files:

```sh
./services/engine/target/release/mosaicforge generate --template template.svg --spec generation.json --out output-svg --batch-size 7 --threads 2
./services/engine/target/release/mosaicforge generate-pdf --template template.svg --spec pdf-job.json --out output-pdf --batch-size 7
```

The output directory must be new and its parent must exist; outputs are never replaced. Publication requires filesystem hard-link support. SVG/PNG generation supports one or two threads; PDF processes one page at a time.

The CLI writes `manifest.jsonl` and structured stdout records. Its PDF includes accepted variants in global index order, with rejections recorded in the manifest. Unlike the web's complete-series requirement, CLI jobs may contain rejections. Zero accepted variants produces no empty PDF.

## Configuration and API

| Variable                   | Purpose                                                 |
| -------------------------- | ------------------------------------------------------- |
| `MOSAICFORGE_HTTP_PORT`    | Docker host web port; default `3000`                    |
| `PORT`                     | Native launcher web port; default `3000`                |
| `MOSAICFORGE_ENGINE_PORT`  | Native Rust port; default `4771`                        |
| `MOSAICFORGE_ENGINE_URL`   | Private upstream; canonical `http://127.0.0.1:<port>`   |
| `MOSAICFORGE_WEB_ORIGIN`   | Exact browser origin, including port                    |
| `MOSAICFORGE_ENGINE_TOKEN` | Private shared token when managing processes separately |

The launchers configure internal addresses and the token automatically. Never expose the token through `NEXT_PUBLIC_*` variables or commit it.

The web proxy exposes `POST /api/inspect`, `/api/preview`, `/api/export`, and the legacy `/api/export-pdf`. Accepted-series export uses `/api/export-pdf-selection` and the private Rust `/v2/export-pdf-selection` route with explicit accepted indices. Shared schemas are the source of truth; generated types and validators are intentionally versioned.

The Docker pilot also exposes asynchronous jobs through `/api/projects` and
`/api/jobs`. A job snapshots an owned project revision, evaluates ordered bounded
blocks, then seals one complete vector PDF and its manifest. Status, cancellation
and native GET/HEAD downloads with byte ranges survive client disconnection.
The current pilot accepts up to12 pages with one worker; the editor's durable-job
controls and history are a separate next step. See [worker protocol](contracts/worker/v1/README.md)
and [artifact design](docs/adr/0013-sealed-vector-job-artifacts.md).

- [Preview/export contracts](contracts/preview/v1/README.md)
- [Generation schema](contracts/engine/v1/generation.schema.json)
- [PDF job schema](contracts/engine/pdf/v1/job.schema.json)
- [Native web setup](apps/web/README.md)
- [Supported SVG subset](docs/svg-admitido.md)

## Deployment and security

Docker packages Rust, Java and Next.js together, keeps the engine/API private,
and publishes only the web on host loopback. PostgreSQL stores projects and job
state; authenticated private Valkey notifications supplement database recovery.
Named volumes retain the database, queue and Rust journal/artifacts. Credentials
are generated once in ignored local state. The current server-configured local
principal does not provide individual login or production multiuser isolation.
This configuration is for local testing, not an Internet-facing service.

```sh
node scripts/local-compose.mjs ps
node scripts/local-compose.mjs logs --tail 100
node scripts/local-compose.mjs restart
node scripts/local-compose.mjs down
```

Rerun `up -d --build --wait` to update code. See [Docker deployment](docs/docker-local.md).

The importer rejects scripts, external resources, embedded images, fonts, and unsupported SVG features. Only a bounded subset of declarative class styles is normalized. Keep services on loopback and respect resource limits. Coverage checks do not guarantee arbitrary artwork is readable or aesthetically valid.

## Troubleshooting

| Problem                 | Check                                                                                                                      |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| SVG rejected            | Review the supported subset; convert text to paths and remove unsupported features.                                        |
| Port occupied           | Choose a free port and keep the browser origin and configuration consistent.                                               |
| PDF returns HTTP 413    | The document exceeds the local export size limit. Reduce the sample count or SVG complexity; no partial PDF is downloaded. |
| HTTP 429                | The local engine runs one operation at a time; retry after it finishes.                                                    |
| HTTP 504                | Normal proxy operations allow 30 seconds; vector PDF allows 120 seconds. Reduce series size or artwork complexity.         |
| PDF export fails        | All web samples must be accepted and physical page coverage must pass. JSON responses are capped at 16 MiB.                |
| Windows file-lock error | Stop the process serving the affected executable or standalone directory before rebuilding.                                |

PDF output is vector RGB. PDF/X, CMYK conversion and printer/RIP integration are
not implemented. Durable jobs rebuild an interrupted PDF from committed blocks;
they do not append to a truncated PDF. Cancellation is cooperative; a running
stage may finish before stopping.

## Development and testing

From `services/engine`:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

From the repository root:

```sh
pnpm --dir apps/web format
pnpm --dir apps/web lint
pnpm --dir apps/web typecheck
pnpm --dir apps/web test
pnpm --dir apps/web build
node scripts/preview-smoke.mjs
```

For manual browser checks on Windows, put Playwright CLI on `PATH` and run `./scripts/playwright.ps1 --help`. The [design audit](docs/design-audit/latest.md) records visual and keyboard checks.

The [phase 2.6 verification report](docs/verification/phase-2.6.md) includes repeatable browser/CLI parity checks, Docker verification commands, bounded CPU measurements and their limitations.

The [accepted-count verification](docs/verification/accepted-count.md) covers deterministic replacement, selected PDF export, bounded failure and browser decoding.

Run checks sequentially. The smoke test needs debug Rust binaries and the web build. Ignored tests require optional local artwork or baselines and are excluded from the default suite.

For contract changes, run the [contract tools](scripts/contracts/README.md), regenerate outputs, and verify shared Rust/Node fixtures. Keep `contracts/generated/` and lockfiles: the build and proxy depend on them.

## Contributing

Keep changes focused, test changed behavior, and document compatibility changes. Preserve determinism and CPU-only operation. Update shared schemas before consumers, keep local artifacts and credentials out of commits, and use original or appropriately licensed artwork.

## Roadmap

- Connect durable jobs, progress and paginated history to the editor.
- Measure and extend bounded processing and PDF limits for larger workloads.
- Add individual authentication and production multiuser authorization.
- Consider CUDA only where end-to-end measurements demonstrate a benefit.
- Consider optional Python/ML analysis only if rule-based validation is insufficient.

## FAQ

**Does the PDF embed preview images?** No. The exporter writes vector geometry. PNGs are used for previews and optional downloads.

**Can it generate one million variants today?** That scale is not implemented or verified. Current limits are listed above.

**Are GPU, cloud services, or a database required?** CPU and local storage are
sufficient for the CLI. The durable Docker pilot uses private PostgreSQL and
Valkey; no GPU or cloud service is required.

## License and maintainers

Distributed under the [MIT License](LICENSE). Dependency license inventories are recorded for the [engine](docs/dependencias-motor.md) and [web](docs/dependencias-http-web.json); selected Java dependency families are listed in the [API documentation](services/api/README.md#open-source-dependency-choices).

Maintained by the MosaicForge contributors. A project website and GitHub release URL are not configured yet.

<!-- TODO(owner): Set public repository metadata and review existing Git history before the first public push. Working-tree cleanup does not rewrite past commits. -->
