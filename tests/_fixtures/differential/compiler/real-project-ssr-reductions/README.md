# Automatically reduced real-project SSR fixtures

The Real Project Matrix creates a draft pull request here when the canonical
production SSR comparison (selected lane against the pinned legacy walker)
finds a difference. Each fingerprint directory contains only a minimized
`witness.vue.txt` and a source-bound `provenance.json`; full third-party source
files must never be committed here.

`davinci_ssr_corpus::promoted_real_project_ssr_fixtures_agree` compiles every
promoted witness under its original corpus path in pull requests and the merge
queue. A generated pull request therefore stays red until the product
difference is fixed.
