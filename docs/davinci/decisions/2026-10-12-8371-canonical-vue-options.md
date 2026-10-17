# Canonical Vue configuration (#8371)

The current public configuration policy is recorded in
[the October 17 decision](./2026-10-17-8371-vite-tsconfig-editor-config.md).
The dedicated-file and legacy-namespace behavior below is historical.

The requested public form is defineConfig({ vue: {} }) and the drop-in form
withVue({}, defineConfig(vpConfig)). Keep package names and the existing native
compiler/typeChecker/linter/formatter/languageServer/version/scopes vocabulary.
The options-first helper wraps the already normalized Vite+ export without
calling Vite+'s defineConfig again. Object, Promise, function and async exports
retain their plugins/hooks, consumer-specific fields and original environment;
no eager source evaluation or second plugin injection is permitted.
The previous config-first withVue alias changes to the requested options-first
signature in this minor migration; defineConfig/withVize retain the former call
shape. The whole former alias test is archived and only that identity assertion
changes; its other cases remain literal.

One private Carton projector is embedded by the Rust loader and imported by the
JavaScript adapters. It accepts an already evaluated selected Vite source,
evaluates an existing legacy native callback once, and merges canonical leaves
over legacy leaves. Arrays replace, undefined is absent and explicit false is
retained. Paths keep the selected Vite root/config-directory origin. The helper
does not mix in an unselected dedicated file. Native CLI/editor dedicated file
discovery and the legacy unknown-key behavior remain unchanged. Canonical-only
validation derives section shapes from the existing native schema and rejects
unknown options clearly.

Canonical root sections reuse generated native TypeScript types. Canonical
entries expose only the actually admitted basePath/files/ignores/linter keys;
root-only sections/version in entries refuse rather than silently disappear.
TODO: additional scoped native sections and per-file editor scope effects.
The native 0.10 dialect is retained, including its existing host-compiler path;
no Vue 3 compilation claim is made for host-compiler compatibility modes.

Packed consumer checks reuse the original built packages and selected installed
peer: Vite+0.2.9, consumer Vite/core0.1.24 and the Vite+ direct core0.2.9 are
different identities, all recorded without upgrading them. They cover strict
public types, all four export shapes, whole native normalization, plugin/hook
identity, original task ownership and genuine compiler output. The source-built
CLI/editor comparison uses one same-run receipted executable and the same
authored file, complete JSON diagnostics, formatter output, raw RPC and lifecycle.
The full legacy source/corpus and original package test cases are retained;
source-only projector tests confer no compilation/performance/release credit.
Fresh exact-head source/native Actions, protected merge and a published consumer
tuple remain required. No full-project Matrix, typechecker speed, LSP RSS or
release issue closure is claimed by this API slice.

This record is paired with #8371. The bilingual reader guide is explicitly for
the next minor release, preserving the currently published configuration path.

The first #8768 package gate exposed a frozen-source conflict with the approved
alias assertion change. Its private input reader now authenticates all three
complete current input files and the whole historical config test, admitting
only the original family or that one declared assertion successor. The unchanged
input-only VM receives the historical body; all three original digests and nine
whole plans remain literal, with the original helper and package caller archived.
Missing, partial, mutated, nonregular and hybrid sources refuse. Source-only
controls confer no native formatter, package execution or performance credit.

The first exact-head Actions run passed the packed public type check but stopped
at a genuine double normalization of root entries. Task metadata now returns the
raw shared projection and the existing consumer normalizes it once. The server
merge law compares the whole result with the actual selected Vite+ merge function,
retaining its host metadata and the original port/host assertions. Nuxt's existing
constructor alone supplies a private context for its already resolved host Vue
compiler object. Only its source-backed isProduction/template/script/features
fields are excluded from native projection; raw canonical inputs, mixed fields,
unknown fields and spoofed public flags stay strict. An accessor's first value is
captured once, and original host objects, descriptors and hooks remain intact.
The CLI scope control now uses OtherView.vue so its authored component name obeys
the unchanged Essential preset. The original failing packets and whole source
bodies remain retained. Packed four-shape execution, all four build/map controls,
CLI/editor cases, Nuxt integration and original formatter history require fresh
Actions on the successor; prior source/native checks do not qualify these fixes.

The successor's warning gate exposed two unawaited test registrations, an
unbound getter identity read and a heterogeneous tuple inference in its source
reader. Explicit void registrations, whole descriptor equality and a string-key
iteration retain every assertion and physical refusal. The zero-warning gate
stays unchanged; the original failed job and complete before/after bodies remain
retained before fresh qualification.

The exact `01aef925` Actions consumer passed the strict packed declarations, all four object/promise/function/async settings, the native CLI/LSP comparison, and the candidate Nuxt build/SSR/hydration/navigation. Its new compiler observer incorrectly rejected Vize’s existing `vite:vue` compatibility shim by name. The declared test-only successor preserves that whole failed helper and three complete unchanged producers, requires the original throwing host object to be absent, and checks the single shim’s complete own-key/prototype/API-descriptor/version geometry without invoking its accessors. All four real compiler builds, authored source maps, whole-output comparisons, whitespace effects, and Vite Plus injection checks remain literal; their actual after-fix execution is still required.

The same source’s prefetch comparison failed in its mixed prior arm: the exact historical Nuxt `4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf` constructor called the current Vite adapter without its private Nuxt context. An explicitly adapted historical transport now adds only the context argument to that whole authenticated constructor before the existing package build. Its raw archive, complete before/after constructor, exact inverse, original prefetch algorithm and all original consumer assertions remain retained. This arm is labeled adapted historical compatibility; it does not qualify the raw historical product bytes. Missing, unknown, already adapted, linked, executable and directory sources refuse. The complete original prior failure is retained. Fresh source/native/protected qualification and all actual prefetch arms remain pending.

The exact `208ce969` zero-warning check retained one new tuple-inference warning in the historical adapter evidence writer. Explicit `{ name, body }` entries preserve the identical two output filenames and complete source bytes without a lint exemption. Four separate setup jobs failed with HTTP 504 before their checks executed; these are retained as unavailable executions, not product or security findings. The fresh exact-head qualification remains required.
