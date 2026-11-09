# Automatically reduced real-project production DOM fixtures

The Real Project Matrix creates a draft pull request here when the canonical
production-reach comparison (a `dom_inline` or `dom_module` adapter compile,
or its Croquis projection, against the pinned legacy lane) finds a
difference. Each fingerprint directory contains only a minimized
`witness.vue.txt` and a source-bound `provenance.json` naming the shape and
the corpus-relative filename; full third-party source files must never be
committed here.

`davinci_production_reach::fixtures::promoted_real_project_reach_fixtures_agree`
compiles every promoted witness in pull requests and the merge queue. A
generated pull request therefore stays red until the product difference is
fixed.
