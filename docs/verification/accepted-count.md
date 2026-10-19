# Accepted output target — 2026-10-19

The web target now counts accepted outputs. Rejected candidates are replaced in ascending global-index order, preserving `(seed,index)` determinism. The gallery counts accepted outputs and reports attempts/discards separately. An accepted PNG must decode in the browser before it can occupy a slot.

Search is bounded at ten attempts per requested output (up to 120 in the local editor). Rules that cannot supply N valid outputs, index exhaustion, cancellation or transport/decode failures leave an explicitly incomplete series; a complete-series PDF stays disabled. The CLI v1 still counts candidates.

PDF selection is additive [v2](../../contracts/preview/v2/README.md). V1 schemas and endpoints are retained. The new BFF/engine endpoint carries all accepted indices, validates exact response association and emits a single vector PDF. Editing controls after completion preserves the saved seed, rules and selection. Size limits and physical coverage can still prevent export; no partial document is downloaded.

## Verification checklist

- [x] ~~Observed the old failure: requested 12 with two rejections produced only 10 accepted.~~
- [x] ~~Controller regression: twelve accepted at indices 0,1,3,4,5,6,8,9,10,11,12,13 after fourteen attempts.~~
- [x] ~~Bounded failure, u64 exhaustion, stale response/cancellation and snapshot tests.~~
- [x] ~~Undecodable PNG cannot count as an accepted preview.~~
- [x] ~~Shared contract cases reject duplicates, descending order, leading zeros, overflow and wrong count/start.~~
- [x] ~~CPU selection preserves dense page order, exact u64 indices, deterministic PDF bytes and unchanged v1 geometry bytes; selection changes configuration hash.~~
- [x] ~~BFF selection association, real HTTP selection/rejection and no partial PDF.~~
- [x] ~~Complete suites, production build and deployed browser/PDF verification.~~

The RED results were observed before implementation for accepted count (10 versus 12), bounded retries (1 versus 10), contract acceptance, missing selected CPU function, BFF forwarding (400 versus 200) and browser decode failure. The corresponding focused checks passed after implementation.

To repeat browser verification after building and starting Docker, run from repository root:

```powershell
New-Item -ItemType Directory -Force out/accepted | Out-Null
./scripts/playwright.ps1 -s=accepted open http://127.0.0.1:3000/ --browser=msedge
./scripts/playwright.ps1 -s=accepted run-code --filename=scripts/accepted-count-e2e.browser.js > out/accepted/browser.log
node scripts/accepted-count-verify.mjs
python docs/qa/inspect-vector-pdf.py out/accepted/twelve.pdf out/accepted/pdf-inspection 12
./scripts/playwright.ps1 -s=accepted close
```

Inspect Playwright CLI's `### Result` / `### Error`, since its exit code alone does not establish success. The test uses the public fixture with scale range 1–1.3 to force real geometric rejections, then requires twelve decoded images, exact selected indices, a twelve-page PDF and byte-identical re-export after editing controls. A second scenario with impossible framing must stop at ten attempts with no export.

No million-variant capacity or new acceleration is claimed. No new required library, language or service was introduced. Rust checks use Linux Docker because native Windows tool execution was blocked during phase 2.6.

## Executed results

Final checks passed: 33 web tests, formatting, ESLint, typecheck and production Next build; seven Node contract tests with reproducible generated output; Rust formatting, Clippy with warnings denied and 153 CPU tests, with ten explicitly ignored local artwork/measurement tests. The Rust verification was run with two concurrent build jobs after stopping an initial competing build; ignored tests are not counted as executed. Independent code/QA review reported no blockers.

Deployed real browser result: seed 2026, public fixture and scale range 1–1.3 completed twelve accepted variants in thirteen attempts. Index 3 was rejected; the PDF selection was `0,1,2,4,5,6,7,8,9,10,11,12`. All twelve images decoded. The first QA range 0.7–1.8 produced no rejection and therefore did not satisfy the replacement scenario; it was tightened for the test without changing engine validation. The gallery's lazy loading was removed for these twelve bounded thumbnails; mobile verification scrolls to and decodes every image before capture.

The downloaded PDF is 442163 bytes, SHA-256 `da1bd88380a67aba18a3284b305b1b7ed90b774f19a784fadd42c258f16d2b32`. Re-export after editing the seed control was identical byte for byte. Equality is checked on the actual nonempty downloaded files, not Playwright download-response bodies, which can be empty. Structural inspection confirmed twelve pages of 100 × 100 mm, path operators and zero Image XObjects; PDFium rendered all twelve. Pages 11 and 12 were visually inspected. Impossible framing stopped at ten real rejected attempts with an explicit incomplete-series error and disabled PDF.

Desktop and mobile screenshots were reviewed. Runtime image ID `sha256:014dd5157a50cf361aedbae18c2412f24ed6e838ad2011b3f393e37d30e86e6b` is healthy with only `127.0.0.1:3000` published. No v1 published schema, license or CI configuration changed.

The unchanged original local SVG was also tested with the editor defaults: fourteen attempts produced twelve decoded accepted previews, replacing rejected indices 3 and 7. The selected indices were 0,1,2,4,5,6,8,9,10,11,12,13. One PDF downloaded successfully (HTTP 200): 12266910 bytes, SHA-256 ad133e869db5226645aa7586e3a978dc6751dd6065fc213d068c3f6dcf6f6b92. Inspection confirmed twelve 100 × 100 mm vector pages, zero Image XObjects and twelve renders; pages 1 and 12 were visually reviewed. The original and its images remain private local test data. This output is close to the existing local size cap; larger documents still fail explicitly rather than download partially.

The updated previous editor E2E also passed, and native CLI comparisons retained identical PDF/SVG/PNG bytes for its consecutive-index fixture. This is additional compatibility evidence; CLI v1 does not expose arbitrary index selection.
