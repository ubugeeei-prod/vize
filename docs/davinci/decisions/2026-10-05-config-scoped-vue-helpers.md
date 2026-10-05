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
branches from actual main 7f605de65444cf3a54120709fde3c48099dddce8 and does not
depend on that unmerged source. The original #7861 config anchor remains its
authority. Logo assets and the three delivered bug-fix receipts are preserved.

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

Fresh strict source Actions, authentic whole native/editor execution, independent
artifact verification, protected full Rust/all 104 instruction gates, actual
signed merge/issue closure and public release/consumer proof remain required.
No runtime execution, speed improvement or publication is claimed by this
source decision.
