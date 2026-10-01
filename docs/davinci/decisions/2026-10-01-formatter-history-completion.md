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
complete input/output hashes, shared full stderr/probe assets, actual process
states and explicit repetition counts. Each successful default output
is a fixed point; typed JSON/JSONC errors and the internal small-const control
remain distinct. Original helper bodies have explicit public SFC wrappers;
wrapper bytes never masquerade as an original whole-file witness.

The [audit](./2026-10-01-formatter-history-audit.md) reconciles all 56 original
fixes and three supplementary semantic fixes to exact cases, retained Rust
laws, engineering controls or explicit supersession proof. Its 300 API plans
include all 87 retained binary references. Seven unversioned packs, 286 plans,
passed two complete source-built local replays; the fourteen versioned plans
await Actions. Native handled, equivalent and paired counts remain zero.

Schema 2 replaces duplicated full outcome records with a 1,495-line semantic
audit and 472 total receipt lines. It keeps all 87 full SHA/subject records,
150 original requirement maps, 250 source identities, 502 named witnesses,
118 independent Rust laws and 24 controls. Each of the 31 other commits has
an explicit source/case/law/control classification; a non-fix subject is never
a blanket waiver. All 300 input/options/expected/error assets and normalized
original/current witness declarations remain unchanged.
Git whitespace metadata preserves authored CR/LF and EOF spaces as fixture
bytes, using the same exclusions as the retained binary references.

The full pre-dedup committed evidence is preserved by
`provenance/formatter-history-full-evidence-20261001` at
`331f64feb9c6b9c0058789fcdedc94daec474e8c` and the archive SHA in the audit.
The frozen source-built observer, raw build logs and seven repeated replay
pairs remain separate historical evidence. Compact metadata hashes do not
reinterpret the old executed manifest hashes. Exact-head Actions produce full
raw outcomes under `target/differential/formatter-api/`, uploaded by the shared
differential evidence action. Normal tests check the complete source/asset/
process/probe hash chain and reject missing obligations or invented credit.

Five historical CLI scenarios compare check, dry-run, write and recheck status,
complete streams, unchanged check/dry files and full canonical writes. Expected
streams are explicitly repository-authored; actual CLI execution awaits Actions.

## Remaining validation

Publication replays only the five owned formatter commits onto actual main,
with direct observer-provider ancestry for every consumer. Historical capture
source pins remain independent provenance.

The later #7382 / #7258 import-sorting feature is inherited from actual main.
It preserves the old public API defaults but changes two retained Rust test
owner files. A bounded source transition pins their original/current full
hashes and eleven original raw function signature/body hashes, including
literals, assertions, comments and whitespace. Independent original/current
Git blob comparison verified those constants before publication; current
whole-owner and function checks require no historical Git objects. All eleven
positive controls pass in a scratch directory without `.git`; unknown owners,
functions, main-source and body drift are rejected. The original audit and
corpus remain unchanged; this grants no execution or output credit. New import-sorting
API/config/CLI corpus obligations remain unfinished outside the original pin,
and old frozen captures provide no coverage for that feature.

The [retained-law source contract](../../../tests/differential/formatter-history-current-witness.ts)
records both complete source-owner SHA256 pins and all eleven original raw
function hashes. Their authorities are original
`cc87bb5960ea9e49e82672205df919de58bb4b24` and inherited main
`d97d352940efe8ac95372b1288a51f0ede05afd1`; an independent second extraction
confirmed complete raw signature/body byte equality for both owners. Known
owner original SHA/revision checks precede the generic unchanged-owner path,
so substituting the inherited SHA cannot bypass the original authority pin.

Fresh exact-head Actions must execute all 300 API plans, actual historical CLI
verdicts and retained Rust/helper laws. Full protected merge-group validation
and actual merge are required before closing #6882. Static registration, local
capture, auto-merge or a queue entry cannot close this gate or admit a native
formatter route.
