# Proportional scale lock

Date: 2026-10-19. Base: d4bf538. Engine/contract checkpoint: 48d908f.
User approved the control and versioned rule before implementation.

## Behavior

Under Transformaciones, **Mantener proporciones (X = Y)** starts checked.
Editing either X/Y minimum or maximum updates both axes. Unchecking allows
independent ranges. Checking again copies the current X range to Y. Zoom still
varies by seed/index; the engine uses one scale factor for both axes in each
locked variant. A checkbox with a clickable label provides native keyboard and
screen-reader behavior and uses the existing focus styles.

The editor explicitly requests rulesVersion2 with preserveAspectRatio. The
existing schemaVersion1 envelope and engineVersion0.1.0 are retained. Inspect
continues to advertise the baseline default rules1 for old clients. See the
[rules2 contract](../../contracts/engine/rules-v2.md). Rules1 forbids the new
field, preserves independent counters4/5, and omits the optional field during
serialization; old configuration hashes/replays are preserved. Rules2 requires
an explicit boolean, and locked requests require matching ranges. Rust, Java
and Node apply the same semantic checks. Generation replies must match the
requested rules version; a silent downgrade is rejected.

Saved SVG/PNG/PDF exports retain their generated snapshot even after the
checkbox or other controls are edited. No dependencies, CI, license, output
limits, infrastructure or GPU behavior changed.

## Verification checklist

- [x] ~~Observed RED before the new contract, UI state and Rust sample behavior; observed Java/Node downgrade correlation regressions.~~
- [x] ~~Twelve shared Node contract tests and reproducible generated contracts pass.~~
- [x] ~~Thirty-six web tests, formatting, ESLint and strict typing pass.~~
- [x] ~~Six Java tests, strict compilation and Spotless pass.~~
- [x] ~~Full CPU workspace: 155 passed, ten explicitly ignored local/profiling tests; Rust fmt and Clippy with denied warnings pass.~~
- [x] ~~Independent read-only review found no P1/P2 in linking, sampling, fitting, hashes and saved exports.~~
- [x] ~~Production Docker build and healthy launch, only host port3000.~~
- [x] ~~Browser control/keyboard, real generation/export and desktop/mobile screenshots reviewed.~~
- [x] ~~Original private test.svg and three-page vector PDF inspection; twelve-page cap documented below.~~
- [x] ~~SVG/PNG/PDF comparisons and legacy replay evidence.~~
- [x] ~~Verified changes saved in the final Git checkpoint.~~

Rust's new regression checks exact equality of X/Y bits across multiple indices,
including u64MAX, and attempts0..7; repeatability, variation between indices,
unlocked independent samples, incompatible-range rejection and omitted legacy
serialization. Existing frozen counter, configuration-hash, CPU/CLI and PDF
tests pass. Ten ignored tests were not counted as executed. Native Windows Rust
tools remain restricted, so these checks ran in Linux Docker.

The first Docker validation attempt was not executed because automatic approval
review reported workspace credits exhausted. A later user continuation permitted
retrying; the tool subsequently executed and the above checks completed. No
approval check was bypassed.

## Browser and artifact evidence

The production image `sha256:018b73c2fe37382c93031701f15a2eccc77582ae5bd941f638fe9e04ca50d5eb`
is healthy with only `127.0.0.1:3000` published. The repeatable assertion script
is `scripts/aspect-lock-e2e.browser.js`, used with Playwright CLI from repository
root. It checks the default lock, updates from both axes, keyboard unlock/relock,
three real accepted rules2 previews, the saved PDF snapshot after controls change,
and desktop1440×900/mobile390×844. Both viewports were opened and their screenshots
visually inspected. There were no uncaught browser exceptions or global horizontal
overflow. This is a CLI assertion script, not a configured Playwright Test suite.

The public fixture's vector PDF has three 100 × 100 mm pages, 99723 bytes,
SHA-256 `d9df7c731c1055d13ec56c70f88e903a537a2619ba5d4e0148d6319ef473e377`.
All pages rendered and contain no Image XObjects. A local adaptation of the
existing editor-parity script used the saved rules2 browser job and the freshly
built Linux CLI. HTTP/CLI bytes matched for PDF, PNG and SVG:

| Artifact | Bytes | SHA-256                                                          |
| -------- | ----: | ---------------------------------------------------------------- |
| PDF      | 99723 | d9df7c731c1055d13ec56c70f88e903a537a2619ba5d4e0148d6319ef473e377 |
| PNG      | 18896 | 029ff923f2dd061bcab7eb195ec2a05f6f1703f4c8f9c25a242167e8f598184c |
| SVG      |  1249 | d08be86b295593400c097839ebf018d0382d5184b1ff63d2a84219133198fc77 |

The original `scripts/editor-parity.mjs` also replayed the saved rules1 job
against the updated CLI and HTTP pipeline. Its PDF/PNG/SVG hashes remained
`e8189b87…`, `af912a9d…`, `73b0757d…`, byte-identical to the complete baseline hashes
in [phase3.1–3.2](phase-3.1-3.2.md). The native Windows CLI predates rules2, so
parity used the new Linux CLI and shared the running container's loopback network.

The private original `test.svg` passed the same five browser checks with three
accepted rules2 variants. Its three-page vector PDF is 3296325 bytes, SHA-256
`a3363079d3b23289739ec8c63748f27b4389923bd70805a34c59e610c4a8992e`.
All pages rendered with no Image XObjects; first/last renders were visually
inspected. The SVG itself was unchanged (SHA-256 `ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8`).

### Observed local export limit

A separate original-SVG run generated twelve decodable accepted previews with
the lock active (scale1.1..2.1), but its twelve-page PDF exceeded the existing
local export cap and returned HTTP413. The UI displayed the size-limit error
without downloading a partial PDF. The first browser probe waited only30 seconds
for a download and timed out; the assertion script now allows the same120-second
PDF wait as the transport. The actual failure was the size cap, not a broken
aspect lock. The later three-page run returned HTTP200. Payload limits were not
increased or bypassed; large-artifact transport remains later work.

Local evidence is under `out/aspect/`: browser.log, original-three.log,
original-limit.log, parity.log, legacy-parity.log, inspection JSON and screenshots.
PDF inspection command:
`python docs/qa/inspect-vector-pdf.py out/aspect/locked.pdf out/aspect/pdf-inspection 3`.

UI review uses the existing design direction and the applicable form/keyboard/
focus checks from [Vercel Web Interface Guidelines](https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md).
This is a focused review, not a full WCAG certification.

### User-reported regression: original SVG, seed 2026, nine variants

On 2026-10-19, the user's existing browser tab still ran the old editor:
it showed rules1, no proportional-scale checkbox, five rejected candidates
among nine cards, and a completed-series message. Reloading that tab loaded
the deployed rules2 editor. No production code changes were needed for this
reproduction.

- [x] ~~Reproduce with the original `test.svg`, seed2026, nine variants.~~
- [x] ~~Confirm nine accepted, decodable PNG previews in the refreshed editor.~~
- [x] ~~Export all nine SVGs and measure their geometric transforms.~~
  All nine had equal column norms (`scaleX == scaleY`) and zero column dot
  product, checked with an absolute threshold of `1e-10`.
- [x] ~~Download and independently inspect the complete vector PDF.~~
  HTTP200, nine rendered pages, 100×100mm, no Image XObjects.

The PDF contains 9731218 bytes, SHA-256
`83d5820e81038ed7e733255b681bd7d8360526961fe595f4bba899f0eb7c66db`.
Evidence: `out/aspect/repro-nine.log`, `repro-nine.png`, `repro-nine.pdf`,
and `repro-nine-inspection/inspection.json`.

The existing preview transport buffers PDFs and caps their size at
`12 * 1024 * 1024 - 64 * 1024` bytes (12517376 bytes), leaving room for JSON
metadata and base64 in the 16MiB response. This is an implementation limit,
not a PDF-format page limit. Streaming large exports remains outstanding.
This successful nine-candidate run did not exercise technical-request retries;
the current editor still stops on transport/decode errors and bounds rejected
candidates at ten times the requested count. It does not guarantee completion
for every possible configuration or service failure.
