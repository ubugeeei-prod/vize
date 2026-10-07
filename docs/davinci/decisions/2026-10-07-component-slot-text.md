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

Independent source review identified a legacy-only widening: raw interpolations
bypass `ToDisplayString`, so grouping two numeric raw values changes `1` and `2`
into `3`. Any selected raw child now retains the complete old per-child implicit
or mixed path. The already-combined explicit wholly-textual raw path keeps its
original spelling and sum; this PR does not correct that separate legacy behavior.
Five full authored Vue 1 sources, mapped/unmapped modules, complete DOM nodes/HTML,
numeric updates and diagnostics are required in the existing protected
`davinci_l2_transform_vue2` feature target. Its real Vue 3.5.35 runtime measures
retained coercion only, granting no Vue 1 runtime equivalence or native credit.
The ordinary 64-cell authored child now invokes both default and header outlets,
so named-slot and conditional updates are actually observable; all original SFCs,
sixteen original controls and authored references remain unchanged. These new
source/runtime qualifications are pending fresh Actions.

Actual `a8e55cac` Check 37590048820 passes the test build and the first whole
32-module law. Its remaining failures are retained: the module-layout gate
rejects a redundant path attribute, and one original complete-module pin encodes
the reported two-vnode bug. Ordinary same-name module discovery and the exact
one-vnode pin preserve every input and surrounding byte without relaxing gates.
The new DOM observer qualifies zero cells because it validates the core map
against assembled imports plus render code. The existing producer serializes
`ctx.out` only, then returns imports separately; the observer must carry both
original fields, assert exact module reassembly and validate the unchanged map
against its actual render owner. Full input frames are retained before validation
so early failure preserves all generated modules/maps/SFC results. No source-map
production change, segment removal or partial DOM acceptance follows.

Actual `0e5f0da` Check 37591913211 passes all 64 complete template DOM cells
and the 32 whole-module law; it qualifies zero of the four original SFC pairs.
The saved first failure retains both complete SFC results: template-only
`MyTitle.vue` exports `render` without a default export, while the test loader
incorrectly reads its missing default. The existing Vite `generateOutput` owns
that assembly and adds the real render-bearing component. The observer now calls
that unchanged production assembler for both original SFC outputs, retaining raw
seven-field results and complete assembled modules before real imports/mounts.
No source, fixture, output reference or comparison is replaced. Current corrected
execution remains unknown. The separate MCP advisory is actually repaired by
signed merge `706a5b7` (#8148); incorporate that genuine main once and require
fresh source checks, unchanged protected ceilings/full Rust and installed release
qualification rather than transferring these historical partial results.

Actual source `73020d28` Check 37594002631 passes the affected build, but shard 4 (job 112704858870, artifact 11469848149) fails before new DOM observations: the observer exits early and Rust reports BrokenPipe while writing its input. This grants zero current runtime acceptance; the old `0e5` 64/32 partial remains historical. The old write panic did not retain child stderr, so its precise loader error remains unobserved.

Source inspection finds that the direct Vite utils barrel eagerly imports native HMR/CSS re-exports, although the ordinary Rust archive intentionally contains no prepared JavaScript addon. Preserve the genuine production assembly instead of adding a stub or a native build campaign: move only the unchanged component-export assembly block into a pure internal module, and let the existing `generateOutputWithMap` delegate to it. All CSS/HMR decoration, imports, default handling, scoped output and map statements remain unchanged. The original styleless SFC observer calls this same assembly owner and retains raw and assembled modules; all whole stock-Vue comparisons stay mandatory.

Retain whole serialized input before process spawn and both child streams before asserting write/exit status; import the pure owner within the existing evidence try/finally after input receipt. This repairs authentic early-failure evidence loss, not a semantic assertion. Every original input/reference, production slot fix, previous failure and cap remains exact. Fresh source Actions, existing full Vite output tests, 64/4 runtime and 32-module laws, protected full Rust/104 plus five legacy raw controls, signed credited merge and installed publication remain required. No current native migration or reported-version runtime completion is claimed.

Exact source `a3b08318` completed Check 37596283269 successfully: all four required contexts, 16,511 Rust cases, whole64/4 DOM/SFC comparisons, Vite tests and the canonical corpus pass. Those execution receipts remain historical rather than transferring to a new source SHA.

A necessary live queue composition check finds only our `.gitattributes` EOF insertion conflicts with newly appended upstream byte-control rules. Relocate the exact three-line #7970 corpus block beside the existing compiler fixture rule; preserve its comment, pattern, every original/reference byte and all production/test code. All incoming queue rules must remain intact. This changes neither effective attributes nor acceptance/caps, and grants no merge or installed-public proof. Record the pure effective-attribute and clean-prefix tree checks, then rerun ordinary exact source Actions before matched admission. PR #8141 is Draft and offqueue during this scoped correction; unrelated #8143 inherited linter/canonical conflicts belong to its existing owner.
