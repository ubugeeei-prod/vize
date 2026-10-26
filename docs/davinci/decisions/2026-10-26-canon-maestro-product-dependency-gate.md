# Protect Canon and Maestro dependency direction

Issue: [#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

Canon and Maestro join Glyph and Patina as protected product roots in the
mandatory level-dependency gate, so all four products named by #6851 are now
gated. Both baselines were measured at
`d858ffb3bd9e0bfeae7b64bb3afd577426ff2dc1`.

Canon has six direct legacy edges: Armature, Atelier Core, Atelier JSX,
Atelier SFC, Croquis and Relief. Maestro has six direct edges (Armature,
Atelier JSX, Atelier SFC, Croquis, Croquis CF and Relief) and fifteen inherited
through Canon, Glyph, Patina, Incremental and Musea.

Each ledger is shrink-only, like Glyph's and Patina's. A pull request may drop a
permission when the edge disappears, but it may not add an edge, change the
baseline, or move an entry to another root. Edges shared between roots, such as
Canon's own edges that Patina and Maestro inherit, are merged per root when the
ledgers are composed, so each root keeps its own ratchet.

The gate is complete, but #6851 stays open: it closes when #6847-#6850 drain
these ledgers to empty.
