# Adjacent component slot text (#7970)

The complete public [report](https://github.com/ubugeeei-prod/vize/issues/7970)
contains the original `cmp.mjs`, `App.vue`, `MyTitle.vue`, DOM observation and
reported child-node arrays. The compiler corpus preserves every fenced byte,
including EOF newlines, separately from authored controls and current execution.
The report concerns Vue 3.5.43 and Vize 0.432.0; it does not concern the Vue 1 dialect.

The retained implicit default-slot generator emitted each text or interpolation
child independently. Its explicit-slot helper merged only wholly textual bodies,
so mixed bodies had the same missing run grouping. The selected L1→L2 emitter
already owned complete compound parts, but expanded those parts into separate text
vnodes to reproduce the retained output. Plain element emission already grouped
text runs. The repair joins each adjacent slot text run into one text vnode through
existing expression and text emission. A named-slot template, preserved comment,
element or structural child remains a run boundary, including children filtered
out of the implicit slot. Scoped expression ownership, raw interpolation handling,
source links and dynamic TEXT flags retain their existing owners.

Existing child/expression functions move unchanged in a move-only commit before
the functional repair. No extra parse, level pass, serialization or fallback is
added. The selected compound path uses its existing owned parts rather than
rebuilding expressions. Legacy output changes are confined to this reported text
vnode structure; existing unrelated histories and instruction ceilings remain
unchanged. The SFC adapter still receives no whole-product native completion credit.

Two ordinary affected Rust laws require 32 complete selected/retained module
comparisons and mapped/unmapped equality, then 64 whole template runtime cells
and four original two-SFC cells. Both whitespace strategies, both comment modes,
named/scoped slots, boundaries, Unicode, two reactive updates, complete HTML and
recursive child-node vectors, diagnostics and unmount are checked against the
existing locked Vue 3.5.35 compiler/runtime. The reported Vue 3.5.43 primary
[text transform](https://github.com/vuejs/core/blob/v3.5.43/packages/compiler-core/src/transforms/transformText.ts)
confirms the run algorithm; installed execution of that reported version is
unexecuted. Runtime receipts retain full original/current modules, maps and source
revision in the normal Rust shard artifact; no additional campaign is introduced.

Current source Rust/runtime execution, protected full Rust and unchanged 104
instruction probes, actual signed merge/footer and released installed verification
remain pending. Pure corpus custody and configured syntax/format checks provide
no runtime acceptance. The critical dependency repair #8137 must be an actual
signed ancestor before source publication; no dependency fix is duplicated here.

Source Check 37588253079 at 20889e14 compiled the changed production libraries,
then rejected the new test's ungrouped array-method expressions inside `json!`;
the JS gate also rejected an explicit `undefined` parameter default. Local whole
module strings and an implicit optional parameter correct those two test-only
errors without changing production, inputs or assertions. This run executed no
new Rust laws or DOM cells. Its separate dependency audit newly reports
GHSA-6qxp-vccf-f47h in MCP SDK 1.30.0; the security owner must repair that actual
dependency, with no advisory waiver or duplicated dependency edit here.
