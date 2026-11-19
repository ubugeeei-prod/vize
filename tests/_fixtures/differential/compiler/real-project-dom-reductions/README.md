# Automatically reduced real-project DOM fixtures

The Real Project Matrix creates a draft pull request here when the canonical
legacy-vs-Level DOM comparison finds a difference. Each fingerprint directory
contains only a minimized `witness.vue.txt` and a source-bound
`provenance.json`; full third-party source files must never be committed here.

`level_dom_corpus::promoted_real_project_dom_fixtures_agree` runs every
promoted witness in pull requests and the merge queue. A generated pull request
therefore stays red until the product difference is fixed.
