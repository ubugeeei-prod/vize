# Governing config authority for Vue helpers

Issue [#7949](https://github.com/ubugeeei-prod/vize/issues/7949) supplies a pnpm
workspace with Vue 3.6.0-rc.10 installed only in `apps/web`. An original `router`
paths alias reaches a sibling TypeScript barrel and `Link.vue`. The widened
common source root makes root-level shared declarations resolve `import('vue')`
outside the app's dependency chain. `defineModel` becomes `never` and native
element attribute checks lose their actual Vue types.

The already merged #7825 config anchor preserves TypeScript option lookup. It
does not change the importing directory of `__vize_helpers.d.ts`. This change
places that declaration beside the same generated governing config. Its include
entry, cold ownership set, incremental candidates/writer/current-path test and
source-membership transition all consume the same path. The existing Vue/JSX
eligibility and O(1) source counter stay intact. No pipeline stage, root dependency
injection, helper type weakening or budget exception is added.

The existing Batch/session repair #7857 owns configured native project selection;
this slice changes VirtualProject helper ownership and config generation. It
initially branches from actual main 7f605de65444cf3a54120709fde3c48099dddce8 and
does not depend on that unmerged source. The original #7861 config anchor remains
its authority. Logo assets and the three delivered bug-fix receipts are preserved.

The differential corpus stores all nine original filesystem inputs. Every SFC
is retained as `.vue.txt` and copied byte-for-byte to a live `.vue`. The existing
frozen test dependency `vue-ssr-css-vars-oracle` supplies the exact genuine
3.6.0-rc.10 package without a new install or lockfile change. App-only and
root-present controls use the same package; production CLI processes remove
external Vue package overrides so an unrelated runtime install cannot mask the
root-absent case.

Required acceptance includes ten complete CLI file/diagnostic vectors: explicit,
default and sharded original runs with root Vue absent/present, plus explicit
and default valid-disabled and invalid-model controls. Four separately authored
native TypeScript 7.0.2 macro/native-element vectors use genuine Vue declarations.
Eight whole editor vectors retain the original App, callable Counter, rejected
Toggle and valid Toggle for each root layout. Original inputs, native and CLI
streams/statuses/arguments, generated config/helpers/four SFC projections/barrel,
editor Results and nine real Vue package manifests/declarations are captured.
The receipt binds their complete hashes to actual source and CLI/native binaries.

A nested cold/sharded/incremental law retains the existing membership law while
proving that the helper is created, included, kept unchanged and deleted at the
governing app path. Neither a stale root helper nor a generic-any helper can
substitute for this control.

The PR native-phase route requires both new CLI/native and editor targets,
unsets the backend-disable envelope and retains fail-fast receipt execution and
always-uploaded raw outputs. All old cargo qualifier argument vectors are
preserved; two existing command continuations are only joined to keep the
workflow below 350 lines. The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5992769531) records the same scope.

The initial private peer found that root-only control alias protection would not
cover the relocated app helper. The successor adds that actual relative path to
existing exact-control protection, retaining every root control and explicit
user alias priority. Absolute in-project targets use the same lexical boundary
as mirror expansion; unrelated absolute imports stay untouched. Nested wildcard,
exact, absolute and real authored-file controls exercise actual governing-config
flattening/materialization. Helper include separators are normalized only as
paths for this cross-platform alias match. Original diagnostic vectors are
unchanged. The official native process failure status is 1, independently checked
against retained 7.0.2 raw receipts; new runtime execution remains unqualified.
The [paired correction decision](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5992964278) records this guard, status and explicit lexical sort correction.

The [paired actual-main replay](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5993327886) starts from signed main 66a9b15639c828b17d5f655b3dd58f697799b3fa.
All 25 owned noncanonical/nonworkflow blobs retain exact source-reviewed bytes;
the incoming #8020 qualification step and all 16 complete existing cargo argv
remain. The sole conflict relocates only this record's canonical note onto the
existing type-checker history sentence, preserving every incoming clause and
the 350-line record. The combined native workflow remains 349 lines.

The [paired first source correction](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5993409758) retains native run 37301981773's e801 compile failure.
Two complete-empty stderr assertions now specify their existing byte-vector
type, resolving serde_json equality ambiguity without changing any expected
byte or diagnostic. The established generator adds only the two genuine new
test/dev inventory rows (nested Carton aliases and editor L0 aliases), retaining
every old row. Production, original sources and all runtime vectors remain
unchanged; no new runtime control executed before the compile failure.

The [paired fixture IO correction](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5993633513) retains current2d10's successful native qualifier
and its strict Rust rejection of 36 unchecked fixture utility accesses. Shared
IO/path/package-field operations now return Results with checked parents and
string fields; existing test callers fail on every Err. Optional capture absence
keeps its established contract, and the required workflow/receipt still requires
all cases. Compact runtime JSON, original inputs, genuine declaration sources,
all whole expected vectors, production and workflow bytes stay unchanged. Fresh
successor strict/native qualification is separate from the historical packet.

The [paired predicate correction](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5993949278) retains exact20145 native run37304532817's success
and strict builder111745874867's two panic-in-Result failures. The same five
Vue/package name/version and root-absence predicates now return Err from the
utility; existing test callers still fail on every Err. A mechanical inverse
restores the entire prior helper. Production, exact originals, whole oracles,
capture layout and workflow remain unchanged without lint exceptions. Fresh
successor source/native acceptance remains separate from that historical packet.

The [paired signed-main7c replay](https://github.com/ubugeeei-prod/vize/issues/7949#issuecomment-5994005204) retains the whole incoming P0 delivery ledger,
every corresponding canonical line, and all 28 owned noncanonical blobs from
the source-reviewed1dfd snapshot, including the unchanged349-line workflow.
The clean replay uses actual main rather than a queue candidate; original sources
and whole expected vectors remain exact. Fresh successor gates are required.

Fresh strict source Actions, authentic whole native/editor execution, independent
artifact verification, protected full Rust/all 104 instruction gates, actual
signed merge/issue closure and public release/consumer proof remain required.
No runtime execution, speed improvement or publication is claimed by this
source decision.
