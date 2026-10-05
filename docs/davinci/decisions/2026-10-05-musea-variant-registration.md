# Original Art variant registration context

Paired decisions: [#7897](https://github.com/ubugeeei-prod/vize/issues/7897#issuecomment-5987633855)
and the registration part of [#7900](https://github.com/ubugeeei-prod/vize/issues/7900#issuecomment-5987634064).

The actual Art consumer passed a template-only context for each variant.
Retain the original physical filename, descriptor and fragment root. Reuse
an existing shared descriptor, parse each genuine variant exactly once,
and analyze original descriptor scripts once when semantic rules demand
it. Seed each original fragment's own Drawer from that summary; do not
manufacture an SFC/template, rename the file or expose another variant's
template scopes. Genuine script scopes provide imports and local setup
values without per-variant script parsing.

The original summary's defineArt macro supplies its actual target. An
actual component attribute belongs only to the descriptor custom block
containing that original fragment. Art file stems cannot grant implicit
self registration; ordinary Vue recursive references remain unchanged.
The existing component-source naming policy applies to those literal
paths; general runtime/module resolution is not claimed.

A genuine SourceRoot slice supplies template diagnostics' physical offset.
Ordinary owned template content retains the existing descriptor offset
fallback; script/SFC reports use their existing absolute domains. Keep
per-variant diagnostics, parse errors, directive/severity handling and the
configured callback route; no broad rule skip or missing-component warning
filter is used.

## Original corpus and acceptance

Retain all six original files from the issues in
`crates/vize_patina/tests/fixtures/musea-variant-bindings/`, with exact lengths,
SHA-256 hashes and URLs. Seven complete result laws cover original imports,
macro/attribute targets with unrelated physical filenames, missing imports,
false Art self references, distinct UTF-8/CRLF variant positions, independent
Art blocks, real local/type-only bindings and ordinary Vue registration.
The owned Patina consumer ledger is scanner-generated; no foreign rows,
ceilings or fixture captures are changed.

Exact-source hosted validation, protected acceptance, actual signed merge
and reporter/footer readback are pending. Original-source real mounts
remain a separate unfinished qualification. This first slice fixes the
registration report; #7900's setup-unused conclusion still needs all actual
variant reads and one physical script report. Its dependent child must use
a genuine native Stack if delivered before this parent actually merges.
No default/native/SDK/fix-history closure, compiler behavior change or
performance improvement follows from this source preparation.
