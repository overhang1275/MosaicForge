# Rules version 2: proportional scale

Approved 2026-10-19. The GenerationSpec envelope remains schemaVersion1 and engineVersion0.1.0; transformation behavior is explicitly selected by rulesVersion. Existing rulesVersion1 documents retain their exact sampling and serialized configuration identity.

Rules1 forbids preserveAspectRatio. Rules2 requires rules.preserveAspectRatio as an explicit boolean. These cross-field semantic requirements are enforced by Rust, Java and Node validators in addition to JSON Schema validation. A locked request requires identical finite positive scaleX/scaleY ranges; conflicting ranges are rejected, not silently replaced by the backend.

For rules2 with preserveAspectRatio=true, sample the existing scale-X counter (rule4) once for each (seed,index,attempt) and apply that factor to both axes. Scale-Y counter5 remains reserved. Translation, rotation and other rule counters are unchanged. Candidate fitting applies the same coverage correction factor to both scales. With the flag false, preserve independent sampling using counters4/5.

Include the flag and rulesVersion2 in configuration identity and report rulesVersion2 in accepted/rejected VariantResult. Replays must retain these fields for SVG, PNG and PDF. Legacy documents omit the optional field entirely; a deserialization default must never add a false field to rules1 serialization or change its hash.

The editor starts locked. Linking copies the current X range to Y. Editing either scale range while locked updates both. Unlocking retains the current values and allows independent edits. Existing saved series keep their own immutable settings after the control changes.
