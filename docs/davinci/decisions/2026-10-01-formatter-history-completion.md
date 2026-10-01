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

## Complete literal witnesses

Register 118 additional complete literal/finite-loop references from the
existing Rust witnesses, preserving original inputs, explicit options and
source-expression proof. Fourteen supplementary Vue-version plans retain all
three whole-SFC profiles, four hyphenated/default selectors and seven Vue 2
payload/event/malformed-pipe arms. Their expected bytes are authored assertions
and include files; they are not described as fresh captures. The 118 default
plans passed two local source-built replays. Versioned execution awaits Actions.

## Complete observations and denominator reconciliation

Eighty further plans retain three actual source-bound capture receipts with
repeated raw input, output, stderr and status. Each successful default output
is a fixed point; typed JSON/JSONC errors and the internal small-const control
remain distinct. Original helper bodies have explicit public SFC wrappers;
wrapper bytes never masquerade as an original whole-file witness.

The [audit](./2026-10-01-formatter-history-audit.md) reconciles all 56 original
fixes and three supplementary semantic fixes to exact cases, retained Rust
laws, engineering controls or explicit supersession proof. Its 300 API plans
include all 87 retained binary references. Seven unversioned packs, 286 plans,
passed two complete source-built local replays; the fourteen versioned plans
await Actions. Native handled, equivalent and paired counts remain zero.

Five historical CLI scenarios compare check, dry-run, write and recheck status,
complete streams, unchanged check/dry files and full canonical writes. Expected
streams are explicitly repository-authored; actual CLI execution awaits Actions.

## Remaining validation

Fresh exact-head Actions must execute all 300 API plans, actual historical CLI
verdicts and retained Rust/helper laws. Full protected merge-group validation
and actual merge are required before closing #6882. Static registration, local
capture, auto-merge or a queue entry cannot close this gate or admit a native
formatter route.
