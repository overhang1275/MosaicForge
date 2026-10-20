# Rules version 3: preserve original negative space

Version 3 keeps the explicit `preserveAspectRatio` flag and scale semantics of
version 2. Its validity policy separates artwork ink from the source frame:
transparent gaps that belong to the SVG are allowed. At least 512 painted pixels
must remain in the 256×256 validation mask; an empty/near-empty crop is rejected.
The source viewBox must still geometrically cover the generated viewport after
rotation, translation and scaling. Proportional scaling remains exact when locked.

Physical PDF validation applies the same nonempty-artwork policy and additionally
maps the entire media rectangle, including bleed, back through the two centered
cover mappings and the inverse generation transform. Every corner must stay in
the original viewBox; uncovered bleed is rejected. The point-in-frame comparison
uses a 1e-9 relative numerical margin in source units, not a measured pixel-error
or GPU tolerance.

Versions 1/2 sampling, acceptance thresholds, request semantics and replays remain
unchanged. Version 3 is a new reproducibility identity; it does not change the
counter PRNG or guarantee aesthetic legibility for every accepted crop.
