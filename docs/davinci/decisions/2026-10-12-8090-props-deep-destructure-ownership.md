# Compiler-owned prop destructuring depth

The bounded [#8090](https://github.com/ubugeeei-prod/vize/issues/8090#issuecomment-6117907123) decision
corrects `script/no-deep-destructure-in-props` before considering any further
preset enablement. It remains opt-in. Preparation starts from actual main
`edc003416dafd322aa6cd100dde488fb112a2abb`, including the actual setup-export
preset fix. The independently reviewed production source is
`33a66544ab5543b4eecbfb5f5dfd8bf1bce6297c`.

## Product boundary

The old rule visited declarations in arbitrary JS/TS and ordinary module
scripts, identified a macro by its last callee name and counted only objects.
That warns for local/foreign/member calls and misses nested array bindings.
The correction uses the existing shared AST and conservative macro binding
proof. Only the actual builtin rule receives the narrow capability; custom
rules with the same name keep ordinary default dispatch and source context.
The existing rule-major iterator and every other rule body remain unchanged.
No sibling parse, source stage, type query, public configuration or Canon API
is added. The preset registry, default depth of one, severity and diagnostics
remain unchanged.

The builtin caller proves a sole inline `<script setup>` using its existing
`runtime_props_eligible` descriptor authority. The rule inspects top-level,
nonambient variable declarations whose initializer is a compiler-owned bare
`defineProps` call, optionally inside a compiler-owned bare `withDefaults`.
Initializer parentheses and TypeScript wrappers follow the existing AST
unwrap operation. Member, optional and TypeScript-wrapped _callee_ expressions
remain outside macro recognition. Object-root binding spans remain whole and
arrays nested within object bindings now contribute depth. Defaults count the
binding on their left, not expressions on their right; aliases/rest remain
shallow unless their actual binding is nested.

This is a depth policy, **not a compiler-error proxy**. Vue 3.5.43 with
`propsDestructure: false`, and `withDefaults`, can compile the same nested
binding while the opt-in rule warns. No compiler-mode field exists in this
legacy lint context; the correction does not infer one. Root arrays are outside
the retained object-root contract. Sibling scripts/external source descriptors
cannot prove this bounded ownership and stay silent. The reused conservative
binding helper still treats nonambient type-only namespaces as possible runtime
bindings; that named false negative is not finished coverage. Parser recovery,
complete compiler/provider parity, default Matrix parity and broader #8090
remain unfinished.

## Independent authorities and complete packets

`tests/_fixtures/differential/lint/props-deep-destructure-8090` retains all eight
initial controls and their complete public/compiler/related-provider roles.
Official compiler-sfc 3.5.43 and eslint-plugin-vue 10.9.2 source/license bytes
are inert references. `vue/no-setup-props-reactivity-loss` is a related rule,
not an exact oracle for this policy. Its eight original whole refusals differ
only by physical `/tmp` versus `/private/tmp` file paths; both whole packets
and a path-only law remain archived. No messages, ranges or fixes are projected
away. The standalone compiler control is explicitly inapplicable.

The original 31 authored neighbors, their sources, original full failures and
all raw public-before process packets remain archived. A before-only optional
call authorship correction follows the original producer's `CallExpression`
check and actual OXC `ChainExpression` source: the optional call was already
silent. A separate two-field after-only Unicode correction follows the whole
CLI renderer source: columns count Unicode scalar values, so Japanese plus the
fox emoji gives columns 28/46, while byte API spans 60..78 remain unchanged.
Both corrections were frozen before candidate execution and preserve every
other field. Two independently authored TypeScript-wrapped-callee controls and
whole compiler packets prevent unsafe macro expansion; the compiler accepts
those ordinary runtime calls without transforming them.

The final 33 whole API/CLI cases have 21 old warnings and 10 proposed warnings.
Three additional explicit-off routes make 36 CLI/API packets. The old 31 public
runs are untouched, and a separate five-run before archive covers both extra
callees and explicit off. Genuine v0.443.0 binary SHA256
`9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254`
passes these independently frozen before packets; this grants no candidate or
successor release acceptance. Three custom-namesake full SFC engine packets
cover normal/setup/external context and prove compatibility of default dispatch.
The old unit test extraction is a separate move-only commit, with complete
original bodies/snapshots retained. Generated documentation changes only this
rule's implementation line and bounded English/Japanese explanation.

## Whole source-family composition

Original runtime manifest `ea908f5d` and the actual registry-only successor
`7a612fc1` remain literal, including every original before/after archive.
The new `de01ccf2` manifest authenticates all twelve incoming bodies before
admitting three changed carriers: `props_emits.rs`, `script.rs` and
`script_rules.rs`. Nine other bodies, including the registry, ownership
helper, existing typed/runtime props, engine and SFC consumers, stay exact.
Each changed whole body has immutable SHA256/Git blob/size and literal
before-to-after transforms. The separate rule producer is also whole-pinned.
Original iterator and generic dispatch bytes are asserted unchanged.

The two existing runtime/preset custody helpers admit only that complete family
when the three named carriers differ. Whole historical/current helper bodies
are frozen separately. Unknown bodies, changed archives and partial families
are refused; no source hash is rewritten or mapped into old performance
qualification. Original 79 runtime-default, 392 installed, 357 setup-emits,
443 reserved, 70 setup-export, old membership/order laws and measured-host
refusals remain unchanged. In particular, source-family acceptance does not
qualify the old real400 host cohort. No budget, fixture denominator, Matrix
baseline, native driver, query mode or pipeline stage changes.

## Delivery and TODO

Local source-only reference/refusal laws and named JS lint/format checks pass;
these are preparation evidence. Rust/API, the 36 raw whole CLI packets and
physical shared-builder source identity must pass on ordinary exact-head
Actions. Preserve raw stdout/stderr/status/config/argv/source and build custody
before assertions, including any first failure. Then require the full protected
queue suites, actual merge identity and fresh included official release with
installed verification. No local native builder or new manual Matrix campaign
is added. Root owns official release dispatch; failed446 and old443 cannot
qualify the successor. None of those runtime/merge/public gates is claimed by
this source preparation record.

Independent source peer reviewed all 107 prepared bodies and 2,452 preserved
incoming Git blobs, complete twelve-body transforms, three custody helpers and
whole oracle/consumer roles. Its immutable receipt SHA256 is
`e4c82e65b0bbcd77965ed868276f231a8993c8db456e54a2755a7cb5cb4f67cf`;
this is source review only. Candidate Rust/API/CLI and protected/public gates
still require their own execution evidence.

Genuine fresh-main replay uses `dd188515f8ec3c51662b4f4d69adcd39c86a0907` (source CLI retry custody included).
All 105 reviewed nonadministrative bodies are exact; only the companion review/comment
record and incoming canonical clauses differ. Incoming 350 rows and the original
NoExport suffix remain preserved, with only the owned paragraph 198 appended.
Fresh Actions qualify this actual descendant independently of all earlier greens.

Initial source `55b014e`, Check `38205153114`, stopped at the source inventory
plan before runtime consumers: the owned Patina shard was stale after the
production/test move. Full original log, whole original inventory and regenerated
inventory remain losslessly archived. The unchanged official generator
changes exactly three old rows to six actual source/test rows in that one shard;
all product/helper/oracle bytes stay literal. Its local check passes. Fresh
successor source and native execution remain required, with no skipped gate
or qualification transfer from the failed initial source.

The initial check-js job `114666888487` also refused the new companion's
Markdown formatting. Its full log and original whole decision body remain
archived; named formatting corrects only that documentation. No product, helper,
config or expected packet changes. This old failed/cancelled Check remains
historical evidence; every successor source/native gate runs independently.

Rust builders at `e79cfa1` then found genuine E0432: `registry.rs` did not
re-export the imported NoDeep name constant. Full unused-bindings, n8n and
Nuxt3 logs remain archived. The production correction `46c1f88` removes that
import and compares the actual typed capability's existing metadata name to
the registered entry. Custom default `None` and foreign identity both retain
generic dispatch; a separately authored whole foreign-capability engine packet
checks the latter. No registry, new API, preset, rule producer or semantic
expected packet changes.

The separate complete twelve-body import successor `39cf7ab7` preserves
original `de01ccf2`, changes only the whole `script_rules.rs` body through two
literal transformations, and keeps its other eleven bodies exact. Original
Deep helper/caller bodies remain whole archives. The current helper result
retains the original family explicitly plus authenticated `currentSuccessor`;
no original pin is rewritten. The source CLI consumer requires that repaired
successor before execution. New custody laws refuse unknown current/archive
bytes and the failed parent as this repair; complete historical source custody
stays explicitly historical, never execution/measurement qualification. Fresh
source/native, protected and installed gates still remain required.

Independent correction peer authenticated all 17 delta bodies, original whole
helper/caller copies and builder failure archives, twelve-body import succession
and unchanged semantic vectors/laws. Its source-only receipt SHA256 is
`edb2b55134185033c3a768b24d6836dd3b56839f25512901a3b6f4f2a4aa405b`;
9 independent new/retained source laws passed with no skips. Actual successor
Rust/API/CLI, source/native, protected merge and installed release stay pending.

The fresh `feeca40` builders then exposed E0599: the typed metadata method
requires the existing `ScriptRule` trait in scope. All three complete builder
logs remain compressed losslessly with their source/job identities. The narrow
production correction `ecb8c9b` adds that trait to the existing import only.
Its distinct whole twelve-body successor `9591f993` preserves the complete
`39cf7ab7` and `de01ccf2` families, changes only this import in `script_rules.rs`,
and keeps the other eleven current bodies and every semantic vector exact.
The old two import custody laws remain byte-exact. Authentication retains both
historical families and explicitly nests the current trait successor; the CLI
requires that latest family before executing. New mutation laws refuse every
unknown current/archive body and both rejected parents as current source.
Source-only laws pass with no skips; Rust/API/CLI, exact-head source/native,
protected merge and installed release still require fresh execution evidence.

Independent trait-scope peer authenticated all sixteen delta bodies and the
complete three failure logs, preserved old families/laws/whole packets, and ran
eleven focused source laws with no failures or skips. Its receipt SHA256 is
`771021667d623c7bf3e1a95b89271a47796ef9b23443753b5b9ca896db397ee3`.
This is source review, with compilation and runtime gates still pending.

The next genuine replay includes actual main `5c54b119` and its native library
filter source/workflow together. All 135 owned bodies are literal against the
retained trait-repair branch; the incoming canonical keeps 350 rows and only
the already-owned paragraph 198 append. Replay receipt SHA256 is
`73b83e209fd6d01b2b220ddfaa295c82823a9c33ed0cbb6d69439a48241afab7`.
This preserves source identity, with no old execution qualification transferred.
