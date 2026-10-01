# Formatter history completion

Tracked in [#6882](https://github.com/ubugeeei-prod/vize/issues/6882).
This gate precedes replacement of the legacy formatter; it does not authorize
an L1 formatter route or count legacy execution as native support.

## Pinned audit scope

The original issue's denominator is reproducible at
`9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`:
`git log --no-merges <revision> -- crates/vize_glyph` contains 87 commits,
56 with conventional `fix` subjects. The unrestricted log includes one extra
merge commit. Cross-product, safety and performance requirements remain in
the audit; a subject alone does not establish a formatter-output witness.
Supplementary behavioral changes are tracked separately from this denominator.

## Observer provider

Extend the existing source-built public API observer, rather than adding a
production formatter route. User overrides are checked against real Serde
options: unknown names, wrong types, duplicate flags and internal runtime
flags in user JSON are rejected. The full effective options are observed
through a separate JSON probe for every case, including default fields and
the actual internal single-pass flag.

Public JSON and JSONC join the existing script, SFC, template and style APIs.
An explicit Vue selector invokes the existing public versioned template and SFC
APIs for Vue 2, 2.7 and 3. Its actual runtime value joins the options probe;
unselected cases retain the existing default API and unchanged probe format.
An expected-error observation requires an actual typed formatter error, writes
its complete pretty Debug bytes and variant stream, and exits unsuccessfully.
Successful output cannot satisfy an error fixture. Default success cases keep
three real formatting passes and exact byte comparisons; errors and historical
internal single-pass observations remain separate one-call contracts.

The frozen observer's source, Cargo artifact, toolchain, options and raw build
logs stay bound by the existing receipt. Each result retains complete raw
input/stdout/stderr bytes, status and hashes. Missing rows, option drift,
incorrect error status, broken pass chains and invented native credit fail.
Native handled, equivalent and paired counts remain zero.

## Prepared output registration

Register 82 additional API plans so that the existing six plans and this pack
cover all 87 retained `history_*.snap.txt` references exactly once. Complete
CSS error bytes and an unsuccessful typed error remain separate from 81
successful outputs. Fifteen cases retain explicit user overrides, including
false values, line endings, indentation and print widths. Original input
bindings and additional branch counterparts are distinguished in each case.
Two source-built replays passed locally; exact-head Actions remain pending.

## Remaining work

Register and actually execute every prepared byte fixture, reconcile every
original fix to its complete public witness or explicit control/supersession
proof, capture the remaining incomplete observations from unchanged product
source, and compare actual CLI check verdicts and streams. Full Actions and
the protected merge-queue corpus must pass before closing #6882. A pinned
inventory, static hashes or an unmerged local capture cannot close this gate.
