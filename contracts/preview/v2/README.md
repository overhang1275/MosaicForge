# Selected PDF export v2

Additive local endpoint: `POST /v2/export-pdf-selection`; BFF: `POST /api/export-pdf-selection`. V1 endpoints and schemas retain their original semantics.

The version-2 envelope adds `variantIndices` to the existing PDF job. These are canonical decimal u64 strings, strictly increasing and unique, exactly matching generation.count (1–12) and beginning at generation.startIndex. Gaps are allowed. The job retains the original template, seed, rules, preview viewport and physical PDF layout. The response echoes the exact selection; the BFF rejects a different selection even if its counts are valid.

All selected variants must be accepted and pass physical PDF coverage. A rejection, write failure or size overflow prevents publication of the entire artifact. Existing limits remain 5 MiB SVG, 32 MiB request, 16 MiB serialized response, 512 px preview bounds and 12 selected pages.

The response reuses the document summary type from PDF v1 within its explicit v2 envelope. Here startIndex denotes the first selected index and count/pageCount denote the selection length, not the length of a consecutive range. `documentConfigurationHash` hashes sorted-key canonical JSON `{schemaVersion:"2",job,variantIndices}`. Generation configuration hashes retain their v1 meaning; `(seed,index)` randomness is unchanged.

`selection-cases.json` is shared by Node and Rust tests. Background/physical coverage can still reject an otherwise previewable job; the caller must retain an immutable selection and never publish a partial PDF.
