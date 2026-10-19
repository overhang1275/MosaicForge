# Phase 2.6 verification — 2026-10-19

The local CPU editor was checked end to end without changing generation, contracts, validation thresholds or export limits. The current runtime remains available on `http://127.0.0.1:3000`; Rust is private inside the container.

## Checklist

- [x] ~~Real browser import, regeneration after a seed change, immutable saved-series PDF and CLI parity.~~
- [x] ~~Invalid input, count limit, geometric rejection, browser cancellation and incomplete-PDF protection.~~
- [x] ~~Bounded local CPU profiling with explicit measurement limits.~~
- [x] ~~Web and Rust checks, standalone transport smoke and independent review.~~

## Reproduce

Run from the repository root with the local runtime already built and healthy. Playwright CLI and Edge are optional development tools, not runtime dependencies. These checks are executable scripts with assertions; they are not a configured Playwright Test suite.

```powershell
docker compose -f deploy/compose/local.yml up -d --no-build --wait
New-Item -ItemType Directory -Force out/phase26 | Out-Null
./scripts/playwright.ps1 -s=phase26 open http://127.0.0.1:3000/ --browser=msedge
./scripts/playwright.ps1 -s=phase26 run-code --filename=scripts/editor-e2e.browser.js > out/phase26/browser.log
node scripts/editor-parity.mjs
node scripts/editor-profile.mjs
docker build --progress plain -f deploy/docker/Dockerfile.verify .
./scripts/playwright.ps1 -s=phase26 close
```

Put the CLI and Docker executable on PATH, or set `DOCKER_BIN` for the profiler. Parity requires a previously built native `services/engine/target/debug/mosaicforge` executable (`.exe` on Windows); `MOSAICFORGE_CLI` can select another build. The Docker verification image builds its own Linux CLI. Playwright CLI can report a script error while exiting successfully: inspect its `### Result` / `### Error` output; the parity script rejects an error log. Output files stay under ignored `out/phase26/`.

For structural PDF inspection, use the existing optional Python QA environment with pypdf, pypdfium2 and Pillow:

```powershell
python docs/qa/inspect-vector-pdf.py out/phase26/editor.pdf out/phase26/pdf-inspection 3
python docs/qa/inspect-vector-pdf.py out/phase26/profile-12.pdf out/phase26/profile-inspection 12
```

## Observed behavior

The public [SVG fixture](../fixtures/editor-seed.svg) was imported through the real BFF and Rust engine. Invalid SVG received HTTP 400 with `invalid_template`. Selecting a file did not send its content before explicit import. Three previews were generated at seed 2026 and regenerated at 2027. Editing the control to 2028 afterward still exported the saved seed-2027 series as exactly one PDF download.

Count 13 failed locally before a request. Rotation 45° with scale 1 produced an actual HTTP 200 geometric rejection, with no invented preview and no enabled PDF action. These assertions distinguish validation/rejection from a server failure.

For cancellation, only delivery of a real accepted preview response was held by a browser route. Cancel triggered an actual failed/aborted browser request and displayed the cancellation state. Releasing the response did not publish a preview or enable a partial PDF. This is controlled timing, not an assertion that Rust computation stops immediately. Separate controller tests cover a transport that ignores abort and returns obsolete results.

There were zero uncaught browser exceptions. Expected HTTP 400/413 validation responses are not reported as unexpected engine failures. The browser PDF, BFF PNG and BFF SVG were compared byte for byte with the native CLI for their exact saved settings:

| Format       |  Bytes | SHA-256                                                            |
| ------------ | -----: | ------------------------------------------------------------------ |
| PDF, 3 pages | 95,464 | `e8189b87c8ab5f68f42e9a56af171c3bf227eec6759f2d18486414ca25be5176` |
| PNG, index 0 | 17,868 | `af912a9d96996bdbb3f13297e7daad2210c098a1d6f6a7d09bfecd247b5e33f5` |
| SVG, index 0 |  1,249 | `73b0757d087428cb386450107dbe3a21f7d21720e4b0927e6c36768f121b3450` |

The three-page browser PDF and twelve-page profile PDF passed page-count and 100 × 100 mm checks, contained path operators and zero Image XObjects, and rendered all pages with PDFium. All twelve public-fixture renders were reviewed in a contact sheet.

The user's original local SVG was also retested, unchanged: nine accepted identity previews followed by PDF HTTP 413 in 11,079 ms, zero downloads, a visible recovery message and zero uncaught exceptions. This is successful limit handling, **not** a successful nine-page export of that artwork. The private input and its screenshots are excluded from this report's assets.

## Local measurements

These measurements use a restarted production container, CPU release Rust and production Next.js, x86_64 WSL2. Docker reported 12 logical CPUs and 3,943,485,440 bytes of available VM memory. Runtime image ID: `sha256:cd7a503dbb5d481ef265ec5e34aa97844b25df92665a27ae6128b9b6837ed34e`.

Input: the 1,060-byte public fixture, seed 2026, indices starting at 0, editor default transformation ranges, PNG previews at 512 × 512 and one vector PDF at 100 × 100 mm without bleed. Every candidate in these particular runs was accepted. Warmup was one inspect and one preview; PDF had no dedicated warmup. Counts ran in fixed order 1, 6, 12, three repetitions each, sequentially with no other verification workload in progress.

Times below are medians in milliseconds. Each boundary includes HTTP, repeated source import, computation, serialization and response consumption. Preview time is the sum of sequential requests. Total is client-observed elapsed time through receiving the PDF, excluding the subsequent local file write. Medians of columns need not sum to the total median.

| Variants | Inspect | All previews |    PDF | Total median | Total observed range | PDF bytes |
| -------: | ------: | -----------: | -----: | -----------: | -------------------: | --------: |
|        1 |   16.72 |        23.36 |  34.26 |        75.04 |          71.75–76.90 |    31,062 |
|        6 |   21.11 |       170.32 |  91.98 |       274.40 |        267.33–281.99 |   192,406 |
|       12 |   20.87 |       257.41 | 134.64 |       429.52 |        335.57–441.25 |   388,413 |

PDF hashes were identical across the three repetitions at each count. Twelve-page PDF SHA-256: `c973c80f9da29cd215c3c17e69b8887868219c9b97eb68eb70103336481887ee`.

Memory observations are ranges in MiB, not measured peaks or budgets:

| Variants | Engine RSS samples | Web RSS samples | Container cgroup samples | Samples across 3 runs |
| -------: | -----------------: | --------------: | -----------------------: | --------------------: |
|        1 |          6.79–7.10 |   102.75–118.34 |             90.81–106.38 |                     6 |
|        6 |          7.10–7.93 |   118.34–130.03 |            102.41–118.50 |                     9 |
|       12 |          7.00–7.93 |   129.15–145.04 |            116.93–132.65 |                     9 |

The profiler reads `/proc` RSS and cgroup `memory.current` through Docker exec, targeting 250 ms between probes plus command latency. Short operations can finish between samples; count 1 has only before/after samples. The cgroup includes cache, shared charges and the probe process. RSS can include shared pages and must not be summed or directly equated to cgroup usage. These observations cannot prove peak memory, absence of leaks, concurrency capacity or million-variant performance. No renderer/encoder/I/O substage instrumentation was added and no CUDA speedup is claimed. Raw final measurements are in [phase-2.6-measurements.json](phase-2.6-measurements.json).

## Checks and environment limitations

- Web formatting, ESLint, type generation/typecheck, all 26 tests and standalone production build passed.
- Rust formatting, Clippy with warnings denied and CPU tests passed in Linux Docker: 151 passed, zero failed, 10 explicitly ignored local-artwork/measurement tests. The ignored tests were not counted as executed.
- The Docker verification image also passed real debug Rust + standalone Next HTTP/BFF smoke: occupancy 429, disconnected upload releases slot, chunked bodies over 32 MiB rejected, foreign Origin rejected, static assets served, u64 extremes preserved, CLI/HTTP/BFF SVG/PNG byte parity and actual rejection without an artifact.
- Browser/profile used the existing release runtime; the transport smoke used newly built debug binaries in a separate build container with ephemeral internal ports. It did not replace or expose another server.
- Native Windows cargo/rustfmt were blocked by Application Control. The native transport smoke also could not spawn its preview executable. Docker verification completed instead. The temporary Docker image initially lacked rustfmt/Clippy and its registry cache was incomplete; required tools and locked dependencies were then resolved successfully. Native CLI artifact parity still executed successfully.
- Independent review found weak test assertions; they were tightened and the real browser test passed again. No production code or published contract changed in this phase.

## Functional follow-up explicitly deferred

**Requested count currently means candidate count, not guaranteed accepted count.** A twelve-candidate series can still contain ten accepted and two rejected variants. The PDF remains blocked for that incomplete series. The UI's existing final status can still say the attempt series ended; that does not establish twelve valid outputs.

The user requires N valid outputs and explicitly chose to finish this verification phase before implementing replacement. The next functional change must search candidates deterministically, preserve the accepted candidates' real indices in a versioned PDF selection contract, bound attempts and report an explicit failure when N cannot be completed. It must never claim success with fewer than N accepted pages. This requirement is not implemented by phase 2.6.
