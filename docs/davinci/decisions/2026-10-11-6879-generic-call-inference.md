# Keep authored generic inference in component calls (#6879)

Paired decision: [#6879 comment](https://github.com/ubugeeei-prod/vize/issues/6879#issuecomment-6107450742).

The original complete App check on fourth source `da43c7ccd2` reported one
PinInput story TS2322 at 20:11: `string[]` was rejected as `number[]`. Its
pinned child declares `Type extends PinInputType`, with no generic default,
and `[Type] extends ['number'] ? number[] : string[]`. The original story
supplies `ref<string[]>([])` with no number prop; the child's runtime props
default to text. The full Root, story, shared types and original failed vector
are retained at `tests/_fixtures/differential/typechecker/generic-call-inference/`,
with revision `c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2`, hashes and its MIT license.

The producer already distinguishes authored generics from module-alias
generics. Aliases get synthetic `= any` defaults so bare type references remain
legal; the public generic component function uses the authored declaration.
The modern component's returned props/emits helper contract nevertheless used
the alias declaration. When this conditional prop cannot infer `Type`, an
`any` default can select its number branch. The bounded correction borrows the
existing authored declaration for that contract. Alias declarations, the
existing public function emitter and the legacy Vue2 branch stay unchanged.
Genuine authored defaults such as `= any`, `= 'text'` and `= 'number'` remain
part of the props/emits call signature. The existing slot resolver normalizer
is unchanged; this slice does not repair its separate default normalization.
No cache, pipeline stage, parser invocation or broad type widening is added.

A TypeScript 6.0.3 discriminator already reproduced the conditional/default
distinction: the synthetic-any call rejected string arrays, the authored call
accepted them, an explicit number prop rejected string arrays, and genuinely
authored `= any` kept that rejection. That is language-level evidence, not
proof of the real Canon/native cause or a product acceptance result.

Required `vue_codegen::generic_component_tests::` laws generate the complete
authored SFC through the production document factory and preserve alias and
legacy contracts. The native law always qualifies a tracked minimal whole-SFC
cause through the real public project factory and returning native client.
It uses the normal installed test Vue provider, no ReKa checkout, and a declared
strict standalone config. All six cases run: positive vectors must be empty,
negative vectors contain only the genuine model TS2322, and prior-contract
replay must differ only by the false text-model row. This mandatory standard
Rust-shard control runs even when capture and upstream fixtures are absent.

The single composed App qualifier additionally sets
`VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE=1`; the same law then requires the tracked
pinned fixture in an isolated snapshot with its full Root/story/config/import
context. Either native or pinned-fixture requirement also rejects native
disabling. Missing hydration or provider authority fails that suite. Every
observed ancestor provider scope is linked in its original nearest-first
layout; providers are neither installed nor copied through symlinks. Actual
link targets, Vue manifests, config provider bodies and missing scopes are
recorded. The authored fixture stays untouched. Capture is optional for native
semantics: `VIZE_NATIVE_SYNTAX_CAPTURE_DIR` or the standard
`VIZE_TEST_FIX_HISTORY_CAPTURE_DIR` retains the whole packets; without either,
the same complete data remain in the assertion context. No standard cause
assertion is conditioned on fixture availability or capture.

The pinned native control explicitly selects the existing core `tsconfig.app.json`.
That declared identity is distinct from the unchanged original full App CLI,
which supplied no explicit config. The real public checker materializes the
project; the returning native client attests that generated config, full
normalized compiler options and Root/story program membership. Complete
generated code, source maps and authored bodies are retained as verified
content-addressed blobs, with full raw and mapped vectors for each case.

No previous whole native child signature was captured by the original failing
App run. The before control is therefore explicitly a prior-helper-contract
replay: only the old checker/props/emits generic declarations replace the new
declarations in the actual produced Root's unmapped final export tail. Its
authored prefix, aliases, public generic function, source maps, project paths,
config and provider identities remain the same. It is not presented as an
unavailable old full-generator execution. The after control uses the complete
current production project. All cases are retained before verdicts are asserted.

The original text case must have exactly one genuine before model TS2322,
and after must equal its full before vector minus only that row. Explicit
number with number arrays must remain accepted; explicit number with string
arrays must retain its real TS2322. Authored any retains that genuine rejection,
while authored text and authored number preserve their positive array cases.
No original snapshot, reported diagnostic, check, cap or deadline is waived.

Formatting, module layout and generated inventories are preparation checks.
Compilation, native semantics, the unchanged full 32-case App row and all 15
negative controls, source Actions, queue, merge and installed release remain
pending. The App owner publishes the true fifth child once the actual parent
foundation is authenticated and runs the single existing composed qualifier.
This private payload does not dispatch an extra fourth/fifth observer build.

The publisher replayed only the two sealed owned commits onto the authenticated
post-prefix foundation `fe4dd1ee670b61da0b1397a8e77f0734ddcfba25`.
The existing composed workflow includes the same `generic_component_tests::`
filter and requires `VIZE_TEST_REQUIRE_PIN_INPUT_FIXTURE=1`, so the full pinned
six-case suite runs alongside the mandatory portable six-case cause control.
All five control groups reuse the original App row's dependency build; their
complete packets and individual statuses remain retained after any failure.

## Actual first qualifier and bounded corrections

The single qualifier `38129574946` on exact `888a4898a86a0fccfda99490e532eb407fc79157`
failed. Original App output removed PinInput's false model TS2322, but added
TreeAsync's false toggle-handler TS2322 at 86:12. The other 926 complete file
objects stayed exact; unchanged aggregate counts alone were not acceptance.
Both generic pure laws passed, but the native suite failed before any native
case verdict: its guessed virtual-root `tsconfig.json` did not exist. The
actual project materializes a nested generated config. The correction consumes
`generated_tsconfig_path()` and `virtual_root()` through the bounded test-only
project accessor; it retains the actual selected config, options, roots and
whole program. No successful six-case native result is inferred from that run.
The historical full TS40 comparison differs only by the two synthetic helper
default removals; those snapshots remain unchanged pending native acceptance.

The complete original Tree Root, Item, story and index are now retained in
`tests/_fixtures/differential/typechecker/generic-event-spreads/`, with the same
ReKa pin, complete-body hashes and license. Root's typed items supply the slot's
`flattenItems`; the story passes `item.bind` into TreeItem, whose `value: T`
controls `ToggleEvent<T>`. Both existing event inference argument emitters
passed only `ComponentUsage.props`, omitting its authored `spread_props`.
With the genuine authored generic declaration, that missing argument selected
`Record<string, any>` instead of the actual TreeNode and rejected its valid
handler. Restoring synthetic `any` defaults would conceal this separate defect.

Only usages with spreads now reuse the existing props-object builder, already
shared by the mapped prop checker and slot resolver. It preserves source-order
last-binding precedence and the original ScopeChain's lexical/reserved binding
rewrites. The native handler checker and event-navigation resolver receive the
same authored object. No-spread branches keep their original prop loops and
emitted strings, checked against literal `bb52` producer source; a whole native
no-spread case separately checks unchanged vectors and generated bytes.
No source graph, parser stage, fallback type or cache is added. The existing
spread rewriter keeps its parser-free fast path and scoped reference checks. The inference mappings remain discarded as in the existing slot resolver;
the authored prop checker and handler retain diagnostic ownership.

The existing generic filter adds one independent Tree native law. Six tracked
minimal whole-SFC cases exercise the actual editor factory options and both
event argument emitters: the original typed spread, a genuinely wrong callback,
both explicit-value/spread orders, a no-spread case and a real slot-loop binding
that shadows a different same-name setup value.
A small test-only hook invokes the existing editor-document setter before
registration; production and public configuration remain unchanged. The required
pinned control additionally qualifies the original complete Tree modules in
ordinary Batch mode, whose original navigation option is false. This distinction
is recorded rather than claiming a Batch packet exercised editor navigation.

The before argument-omission control erases only unmapped synthetic event spread
entries with equal-width spaces. All authored bytes, maps, remaining generated
code, configuration, providers and native program are unchanged. Full before and
after programs/raw/mapped vectors are retained before strict verdicts: positive
cases lose only the false handler TS2322, wrong callbacks retain their real
TS2322, and every other row stays exact. This isolates the omitted argument and
is not presented as an unavailable old factory/native execution. The original
failed CLI vectors remain separate evidence. Capture stays optional for standard
native semantics; the pinned requirement remains mandatory in the existing
single composed App qualifier. Current-source compilation, all native verdicts,
full App output, snapshots, queue and release qualification remain pending.

## Directory barrel authority after the first composed run

Composed qualifier `38138961313` at exact `8170fd3e` retained all six portable
PinInput cases, but its six pinned cases failed the original-text before
assertion. The full raw and mapped packets reveal genuine TS2307 on the
story's authored `import '..'`; all pinned before/after vectors consequently
missed the required model TS2322. The Tree pinned control had the same missing
barrel. This is incomplete program authority, not successful generic inference.

The dependency resolver and its parser-free prefilter now admit the exact
directory specifiers `.` and `..`. Existing `./` and `../` resolution, module
candidate order, aliases and bare-package fast paths are preserved. A complete
SFC graph law requires both directory barrels and their re-exported Vue
dependency. A neighboring prefilter law keeps `.hidden`, `..hidden` and plain
published packages out of that relative path. The portable six-case corpus now
imports its own real `..` barrel, so its unchanged strict empty/singleton
diagnostic verdicts detect this graph defect without a hydrated Git fixture.

The prior assertion and its complete packet remain in immutable artifact
`11702332387`. No App diagnostic golden is changed by this correction. That
run separately retained whole App differences after authentic provider setup
and the npmX/Vue Fes preparation failures; their qualification remains open.
Exact-source Actions and the composed whole App/native oracle are required
before queue admission. No throughput or 10x-target claim follows this fix.

Existing unpublished worktree heads `065baa9448`, `bb52fbfffb` and `888a4898a8`
remain intact on their original local branches; delivery repairs use a fresh
`repair/canon-stack-delivery-20261011` worktree from the authenticated remote
`1a0d0cbb`. The merged prefix stays immutable. Descendants will be rebased onto
the actual merged main and pushed only against their rechecked remote leases.

## Projection snapshot audit

Source Check `38138943314` at exact `1a0d0cbb0070c861e514864e1727b62b1192087d`
retained three failing projection snapshots in its Rust-shard JUnit artifacts.
The two complete Canon outputs differ only in the final checker/props/emits
resolver call signatures: synthetic `= any` defaults disappear as required by
the authored contract. Aliases, public function, slots, raw props and all
preceding generated bytes remain unchanged. The TS-40 generic fixture's three
producer texts each lose exactly twelve bytes; only their text hashes and the
Canon identity import-map probe count/hash change. Both probe digests were
independently reproduced over every old/new byte offset. Source hash, mappings,
semantic links, diagnostics and authored hits remain exact.

Only these three projection expectations are refreshed from the actual JUnit
output. No authored App diagnostic golden or assertion changes. This records
the intended source contract and permits fresh source gates; it grants no native
correctness credit. The composed pinned-native and unchanged whole App acceptance,
protected queue, merge and installed release qualification remain pending.

The new source pair `38142630222` at `fb350226ed` refused before timing:
the original 400-provider guard did not recognize the reviewed generic
projection snapshot. `warm-type-backed-generic-host.ts` enumerates only this
slice's exact twenty previously unlisted production/test/projection paths.
The existing bare import rewriter path was already recognized. This list admits
the same paired build and measurements; it changes no original 400 input,
response oracle, workload, profile, timeout or latency/memory budget. Unknown
paths retain the original refusal. Passing source or this preparation guard
alone grants no performance result; the actual pair remains required.

## Explicit directory-module resolver qualification

Source pair `38142630260` at `fb350226ed` also refused before its CLI or timings:
the reviewed directory-module resolver did not match either immutable resolver
source in the original eight-case alias contract. A separate
`directory-module-resolver-successor.json` binds the exact new resolver bytes
to the existing selected semantics. The original successor manifest, historical
archive, eight authored cases, diagnostic vectors, file memberships and budgets
remain byte exact. The new source must pass those original native/CLI cases on
Actions; registering it grants no semantic or performance credit.

Four source-contract laws preserve both original resolver/collector axes,
compare every original membership for the explicit successor, and retain the
unknown-source/wrong-head refusal before any trap CLI can run. The original
manifest is never broadened to accept arbitrary source hashes. This is a bounded
qualification candidate for the authored `.`/`..` fix, with exact-head paired
execution and composed provider qualification still required.

## Complete native declaration output and declaration-barrel control

The full native-phase job `114479435486` in `38142630198` passed its preceding
real CLI/config/editor controls and then failed the actual generic declaration
emit snapshot. Comparing the entire emitted `Multi.vue.d.ts` proves exactly
twenty-four bytes change: two call helpers each drop two artificial `= any`
defaults. Every other declaration byte is identical, including the authored
`'a'` default in `Generic.vue.d.ts`, public callable and aliases. This fourth
projection snapshot acknowledges only that intended producer contract; it
changes no diagnostic golden. Its exact path joins the reviewed producer list.

The [whole App baseline audit](./2026-10-11-app-check-provider-baseline-audit.md)
preserves historical green runs and their missing-provider/config custody.
They cannot promote newly configured App vectors. A new authored four-case
direct/declaration-barrel control retains broken/repair public and native
vectors, a genuinely absent neighboring Vue module and the same diagnosing
client's actual `getSourceFileNames` response before its verdict. Its two
test-only receipt paths and fixture test path join the same bounded producer
qualification; the frozen 400 inputs/oracles/budgets stay unchanged. No
declaration-edge production fix or App baseline migration is yet qualified.

The bounded declaration-import law also runs before the existing CLI/native
phase on PR Actions, sharing its exact source, toolchain, profile and sticky
target. Its compiled test name must exist and actual native execution is
required; complete packets and pipeline statuses use a distinct retained
subdirectory. Every existing full App/native qualification remains mandatory.
