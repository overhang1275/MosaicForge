# Binary PDF download verification — 2026-10-19

The local editor now uses the additive `/v3/export-pdf-selection` binary
transport. Existing v1/v2 JSON endpoints retain their behavior. Rust generates
a complete PDF on private temporary disk; Java receives it on disk and validates
metadata, length, signature and SHA256 before publication. Next streams and
holds the final byte until its own integrity check passes. The browser still
uses a Blob before saving; this is not a native/resumable large-artifact download.

## Executed

- Contracts:13 tests passed, generated outputs checked, strict typecheck/lint.
- Web:38 tests passed, including17MiB transport and a real HTTP corrupt-body
  regression. Format, lint and strict types passed; production build passed.
- Java:9 tests passed, strict compilation and Spotless. Real17MiB transfer,
  malformed/mismatched metadata, indices, length, digest, advertised quota,
  cancellation cleanup and legacy port compatibility were exercised.
- Rust CPU:160 tests passed,10 existing local/visual fixtures ignored;
  format and strict all-target clippy passed. New endpoint, byte parity,
  quota failure, stream completion/disconnect cleanup and gate lifecycle tested.
- Independent review found an HTTP Content-Length/integrity race. Regression
  failed before the final-byte fix and passed afterward. No unresolved P1/P2
  findings remained in the local12-preview scope.

## Original artwork

Browser test used the unchanged local `test.svg`, seed2026, indices0..11,
rules2 with proportional scaling enabled and default1..1.8 scale ranges.
All12 PNG previews decoded and all12 exported SVG transform matrices had equal
orthogonal scales. One PDF downloaded with HTTP200 and12586114 bytes, exceeding
the old12517376-byte PDF buffer limit. Evidence is local under
`out/aspect/repro-twelve*`; artwork and output files are not committed.
Independent PDF inspection found12 pages at100×100mm, no Image XObjects and
rendered all12 pages. First/last renders were visually inspected. SHA256:
`c2c466357fe37c17127163ce9c968103e99d5105fbeac8fe312686d0dafc03ba`.

## Limits retained and unverified scale

The local path still permits12 accepted previews, has a120-second upstream
deadline and a1GiB temporary PDF quota. Generation is not immediately interrupted
when its HTTP caller disconnects; the blocking task retains its concurrency
permit and cleans up when it finishes. Full120-second timeout waiting, actual
1GiB transfer, disk-full behavior and Windows ACL behavior were not tested.

Streaming fixes service memory/transfer duplication; it does not make unique
vector pages small. The existing writer already applies Flate compression,
uses classic xref, caps bytes at9999999999 and objects at1000000. The measured
nine-page9731218-byte sample extrapolates to about1.08TB per million pages if
its average stays constant; that is an estimate, not capacity validation.
Durable jobs, native/resumable artifact downloads, writer structure and
measured1000→10000→100000→1000000 scaling remain required before announcing
one million. The single vector PDF requirement remains in place.
