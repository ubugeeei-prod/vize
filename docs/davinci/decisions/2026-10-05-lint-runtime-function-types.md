# Runtime function return annotations (#7913)

The reporter's `RunButton.vue` and `callbacks.ts` contain TypeScript function
signatures, not runtime arrow functions. The legacy byte search matched `) =>`
in type aliases and generic arguments. Select actual `Function` and
`ArrowFunctionExpression` nodes through the existing shared script AST dispatch.
`TSFunctionType`, constructor types, ambient declarations and overload signatures
have no runtime body and produce no finding.

Keep the current rule's parenthesized-arrow, callback, message, help and byte-span
policy. This change does not broaden the rule to bare-parameter arrows or method
syntax, and preserves the existing named-function snapshot. Those broader policy
changes remain outside #7913. There is no new parse pipeline or serialization;
AST rules reuse the one parse already owned by each script dispatch.

The checked-in originals include full issue-body and source SHA-256 identities.
Public API corpus laws compare complete findings and clean results, including
filenames, counts, messages, help, labels, fixes and authored UTF-8/CRLF ranges.
Type/constructor/generic positions, real runtime positives, same-name types,
annotated functions, comments, strings and callbacks are covered.

Local formatting and source review do not establish native execution. Require
exact-head hosted Actions, protected full Rust/fixture and unchanged instruction
count gates, actual signed merge, issue closure and published release evidence.
This is a legacy correction; it grants no Davinci native-stage or speed credit.

Source Check `37257997293` rejected the original corpus carrier because repository
Oxlint reports the intentionally unassigned reporter `handler`. Preserve every
input byte/hash and use `.fixture` carriers, with original public filenames still
passed to the actual linter. No warning budget/configuration is relaxed; fresh
source Actions must qualify the successor.
Hosted source Check `37258429823` rejected the stale Patina consumer surface
inventory. Regenerate the owned shard with the existing official generator;
the raw OXC AST dependencies are explicitly recorded as legacy source custody,
with no native level credit or inventory waiver. Fresh source qualification is
required at the successor head; the failed head stays historical and unqueued.

The remote bottom becomes conflicted after actual main advances to
`a2712e78968e9112c89cbc2111b108bd0e51959c`. Rebase onto that immutable main,
retain the reviewed production/corpus bytes and all current canonical decisions,
and regenerate the census without additional changes. Historical source checks
do not qualify this successor; fresh exact-head Actions and the native Stack
prefix/protected queue remain required before every actual merge and release.
