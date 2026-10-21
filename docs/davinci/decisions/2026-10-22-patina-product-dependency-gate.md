# Protect Patina dependency direction

Patina is now a protected product root in the mandatory level-dependency gate.
Its baseline records five direct legacy edges and six edges inherited through
Canon at `b7515d2e7ccc8589eb6cd8366a21c515c0656f51`.

The checked-in policy is a shrink-only migration ledger. A pull request may
remove a permission as Patina or Canon stops using a legacy crate, but it may
not add a new edge, change the pinned baseline, or transfer an entry to another
product root. The gate also walks transitive workspace dependencies, so a new
helper path cannot hide a legacy edge.

Product ledgers are validated independently and then composed for the graph
check. Identical final edges are merged by protected root, which lets a future
Canon ledger coexist with Patina's inherited paths without weakening either
root's ratchet.

This is the next bounded step for #6851 after Glyph. Maestro and Canon remain
outside the protected-root set until their own exact baselines are reviewed.
