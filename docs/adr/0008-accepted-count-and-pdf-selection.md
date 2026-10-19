# 0008 — Accepted output count and explicit PDF selection

Date: 2026-10-19. Status: accepted for implementation by the user's continuation of the deferred requirement.

The web count must describe accepted variants, not candidate attempts. Rejected candidates do not consume a requested output slot. Candidates are evaluated sequentially at ascending global u64 indices, preserving existing `(seed, index)` determinism and engine/rules versions. Search is bounded at ten attempts per requested variant, at most 120 in this local editor. Cancellation, index exhaustion, invalid artifacts and transport failures never report a complete series.

PDF v1 describes a consecutive range. Replacing rejected candidates can produce gaps, so a new additive `/v2/export-pdf-selection` endpoint carries a version-2 envelope, the existing typed PDF job and `variantIndices` as canonical decimal u64 strings. The list must be strictly increasing, unique, have exactly `job.generation.count` items and begin at `job.generation.startIndex`. The local endpoint retains the 12-page and existing body/response limits. Existing v1 schemas and endpoints remain unchanged.

The version-2 response echoes the selection and reuses the typed document summary: count is the number of selected outputs and startIndex is the first selected index, not a claim that all intermediate indices were selected. The v2 document configuration hash covers the canonical JSON object `{schemaVersion:"2", job, variantIndices}` with sorted object keys and canonical decimal-string indices. Page order follows the selection. Geometry and the PDF byte writer are shared with v1; all selected variants must still be accepted and pass physical coverage. Any error discards the entire unpublished PDF.

The editor keeps immutable accepted snapshots. Editing controls after completion cannot alter the saved selection or seed used for PDF export. The gallery counts accepted outputs and reports attempted/discarded counts separately. Generation failure blocks a complete-series PDF even when some accepted previews remain available.

No new language, service or required dependency is introduced. CLI v1 keeps its existing candidate-range semantics; this change concerns the local web output target. A future large-job contract must explicitly choose accepted-target semantics and persist its selection rather than infer that N candidates are N valid outputs.
