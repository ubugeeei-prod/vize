# Format comment-bearing JSON config files

Issue: [#7865](https://github.com/ubugeeei-prod/vize/issues/7865).

The reported `.devcontainer/devcontainer.json`, `turbo.json` and
`trailing.json` fail default formatter discovery even though the same content
works under the known `tsconfig.json` name. File-name heuristics omit common
authored tool configurations.

Select the existing JSONC formatter for every `.json` and `.jsonc` CLI input.
Keep comments in authored order and accept trailing commas on input while
removing them in printed output. Ordinary strict JSON retains identical
output through the existing shared parser/printer. The public `format_json`
API remains strict: this correction changes CLI file selection policy only.
Malformed values, invalid numbers, invalid escapes and unterminated comments
remain errors and must not modify source files.

The sixteen-row legacy corpus retains all four original inputs, then adds
four previously unrecognised config names, ordinary JSON, empty collections,
array trailing commas and quoted comment-like scalar content. Four complete
error controls preserve the existing grammar. Rust checks all file selectors,
whole output, changed flags, fixed points and strict-public-API rejection.
The existing source-built tooling jobs also run check, three writes and
recheck on every original input, retaining complete raw process streams and
document bytes under `target/differential/`. Failed checks stay read-only.

Hosted source qualification, protected full suites, unchanged instruction
ceilings, actual merge and release are still required. No Davinci formatter
migration or measured performance improvement is credited.

The actual-main replay on `a2712e78968e9112c89cbc2111b108bd0e51959c` preserves every incoming decision, every original author/trailer and all owned production, corpus, runtime/helper and strict witness bytes. The earlier source failure/success receipts remain retained; fresh exact-head Actions and protected delivery are required before requeue.

The next actual-main replay on `8f667ea070bb90d57ac7125db35d791025f746e2` preserves the complete incoming protected prefix and every owned source/runtime/corpus/witness byte from `86bb0906da74f860548082cd90a2d6e808b4e2bb`. Original authors, reports and ceilings remain intact. Earlier qualified heads are retained as historical receipts; this replay requires its own exact-head source and protected reports before actual merge/release.

The genuine bf91 protected-tail composition conflicts only in the shared
canonical footer. The actual736 main replay retains every incoming decision
and all owned source/corpus bytes; this clause now occupies its relevant
distinct canonical location at350 lines. Fresh exact source and protected
delivery remain required; no earlier acceptance or publication is transferred.
Paired [placement decision](https://github.com/ubugeeei-prod/vize/issues/7865#issuecomment-5988292746).
