# SSR fragment root CSS variables

Issue: [#7892](https://github.com/ubugeeei-prod/vize/issues/7892).

The reporter's complete `App.vue` is retained in the compiler differential
corpus, including its final LF. Its two root paragraphs do not inherit
`_attrs`. The previous SSR prelude merges `_cssVars` into `_attrs` only, so
neither paragraph receives the variables before hydration.

[Vue 3.6.0-rc.10's original SSR transform](https://raw.githubusercontent.com/vuejs/core/v3.6.0-rc.10/packages/compiler-ssr/src/transforms/ssrInjectCssVars.ts)
injects a generated object bind on physical root elements and components,
including conditional branches. Ordinary descendants and root `v-for`
bodies are excluded. Root Suspense injects into its direct or slot children.
The [original CSS variable implementation](https://raw.githubusercontent.com/vuejs/core/v3.6.0-rc.10/packages/compiler-sfc/src/style/cssVars.ts)
uses `:--` keys in SSR to prevent a parent's inherited variables overriding
the component's own values.

The correction preserves the current single-root `_attrs` merge and its
output bytes. Roots that cannot inherit those attrs receive `_cssVars` in
their existing props composition, before moved `v-show` and custom
directive props. Both the retained walker and the already-admitted L4
string-plan emitter carry root eligibility through existing emission.
The retained metadata additionally identifies conditional-template and
Suspense-slot carriers; ordinary descendants and loop bodies clear it.
No authored node or span is modified, and generated props have no authored
map links. Neither route reparses input or adds a level, serialization,
admission refusal or per-root storage allocation. These metadata costs
are not a measured performance claim.

Published source `96d4956653572ef9be83c4136a2eb810d1ffdcf2` failed Check
[37260572300](https://github.com/ubugeeei-prod/vize/actions/runs/37260572300):
the generated-bind allocator calls produced two E0277 errors, before runtime
execution; two top-level Node test registrations also violated the promise
lint. That source's retained generated binds additionally changed the
public direct-template API's returned `RootNode`. The successor removes
that mutation rather than merely correcting the allocator reference, uses
explicit `void test` registrations, and compares every returned AST field
against the unchanged original-template compile, with both comment modes.
The original 141-byte SFC remains the input; no assertion or input is removed.
The root/fallthrough and Suspense helper moves precede the functional change
in a move-only commit to keep existing over-limit files from growing.

The original SFC and nine independently authored controls retain complete
public client, SSR and explicit Vapor-to-standard-SSR fallback results.
Controls cover single roots, nested descendants, conditional elements and
fragments, authored style/spread/show props, component roots, root loops,
Suspense slot roots and absent CSS variables. The two template emitters
also compare whole code, preamble, maps and diagnostics under the existing
`legacy-differential` feature. Existing fixtures and ceilings are unchanged.

The dedicated `vue-ssr-css-vars-oracle` test alias pins Vue, compiler-SFC,
compiler-SSR and server-renderer to the reported **3.6.0-rc.10**. All prior
lock entries and importer resolutions are preserved. The existing scoped
SSR composite action captures source-built whole modules, executes their
unchanged bytes under the production server renderer, and uses actual
production Chromium for client rendering and hydration against independent
official compiler output. CSS-variable names use the same original
filename identifier; HTML/property keys are not normalized. Complete CSS
bytes remain captured, but compiler CSS whitespace equality is not claimed.
The browser checks colors before JavaScript, retained node identities,
runtime diagnostics and complete unmount. Only ImportDeclaration source
literals are linked to the pinned browser runtime; authored compiler
expressions and statements remain unchanged.

TODO: require fresh exact-source Actions, actual runtime captures, full Rust
and unchanged 104 protected instruction ceilings, signed actual merge and
literal reporter credit. Preparation alone does not establish any of those
results. The complete compiler history and product migration under #6880
remain unfinished. This fix adds no new native eligibility and grants no
whole-product native credit; Vapor SSR remains the explicitly diagnosed
standard SSR fallback. Publication belongs to the release owner.
