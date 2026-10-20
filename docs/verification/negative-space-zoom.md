# Zoom preserving original SVG negative space — 2026-10-20

The user approved preserving transparent gaps belonging to the artwork while
requiring geometric page coverage and proportional scaling when locked.

## Cause and change

The legacy coverage filter rejected high zoom because original gaps reached
corner tiles. With test.svg, seed 2026 and scale 1..4, accepted samples clustered
around 1.22..1.66. Nine fixed-scale probes at 2, 3 and 4 were rejected.

New rulesVersion 3 separates frame coverage from ink coverage. The transformed
source frame must cover the viewport, and at least 512 pixels must remain painted
in the 256×256 validation mask. Original transparent corners are allowed. The PDF
also checks the whole physical media rectangle, including bleed. Versions 1 and 2
retain their acceptance semantics and reproduction identity.

An end-to-end run exposed a response incorrectly labelled rulesVersion 1 after
executing version 3. A failing HTTP regression reproduced it; generation now
returns the version actually requested.

## Executed verification

- Rust: full CPU suite passed with 162 tests and 10 ignored before the final
  version-label correction. After that correction, formatting, strict Clippy,
  preview tests, constraints, determinism and PDF document tests passed again.
- Contracts: 16 tests passed. Web: 38 tests, format, lint and typecheck passed.
- Java: 11 tests passed, none skipped, with the separate real PostgreSQL test
  database; formatting and compilation passed.
- Docker production build and healthy startup passed. The projects persistence
  smoke read still passed after deployment.
- Real browser: original test.svg, seed 2026, 12 requested, scale 1..4, locked
  aspect ratio, rulesVersion 3. All 12 were accepted in 12 attempts. Each actual
  SVG affine transform had equal X/Y scale and orthogonal axes. Measured scale
  ranged from 1.2224946169 to 3.9662767621. Variant 3 displays the largest zoom.
- Downloaded PDF: 7,364,065 bytes, 12 pages, 100×100 mm, no image XObjects.
  Independent PDF inspection rendered all pages; first and last were visually
  checked alongside the browser preview.
- Legacy version 2 replay: 12,586,114 bytes, identical SHA-256 to its earlier
  output: `c2c466357fe37c17127163ce9c968103e99d5105fbeac8fe312686d0dafc03ba`.

New PDF SHA-256:
`a72650f0cf09ddafe668f8e71503f56688edb7edf982d257919673254efdb18f`.

Local ignored evidence: out/aspect/zoom-twelve.log, zoom-twelve-max.png,
zoom-twelve.pdf, zoom-twelve-inspection/, legacy-v2-replay.pdf and
negative-space-final-rust.log. Input artwork was not modified or committed.

## Limits

Acceptance is geometric and rule based; it does not certify aesthetic legibility.
This is a 12-variant measurement, not a million-page capacity or speed benchmark.
No CPU/GPU tolerance or acceleration was measured. Independent agent review was
unavailable because the workspace agent credit allocation was exhausted; the
checks above were executed by the primary agent.
