# P3-6 — Failure after acceptance (2026-09-26)

The production compile now terminates the native emission route for every
accepted artifact. A payload inconsistency detected while building the shared
IR returns an explicit S3 emission diagnostic with empty code/templates and no
source map. It increments `levels.s3_vapor.rejected`; the obsolete
`legacy.emission` reason is removed. Such a failure never reparses the source
or invokes the retained lowerer, matching the acceptance invariant above.

A production-helper regression damages two already accepted private payloads:
an out-of-range root address and an out-of-range descendant address. Both
source-map settings reject the compile, and a generation callback that panics
proves no partial IR reaches the shared generator. A positive control verifies
that a valid accepted payload reaches generation with its authored anchors,
template bytes and expression operands. The existing code snapshots, native
zero-walk/reparse floor and test-target Clippy remain required gates. P3-6's
remaining semantic and performance exits stay open.

Contract: [P3-6](p3-6.md).
