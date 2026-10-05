# Existing SSR builtin ownership (#7891)

Paired decision: [#7891 comment](https://github.com/ubugeeei-prod/vize/issues/7891#issuecomment-5995871251).
Source preparation base: `d8cd6a208b9aea02150b140a4b81b87222128a51`.
Reporter: `ubugeeei`, public GitHub identity `71201308`.

The original complete App.vue renders an async Suspense default and a tagged
TransitionGroup. Existing SSR appended the `fallback` text after the resolved
content, then resolved TransitionGroup as an authored component. The intended
whole result is `<!--[--><p>done</p><ul><li>1</li></ul><!--]-->` without warnings.

Keep direct Suspense slot payloads in distinct writer functions, preserving
original expression/name locations and existing root CSS-variable handling.
The official server helper invokes only `default`; a missing default renders
`<!---->`. Non-slot children form the implicit default after the existing
slot-child normalization. No-slot and default-only inputs retain the original
emission path and its code/map bytes. Conditional and looped slot-definition
structures remain outside this bounded repair and require a separate history
slice; no broader slot-readiness claim follows.

TransitionGroup owns its authored wrapper. Select the original static `tag`
attribute or statically named `:tag` binding, map the wrapper to that original
value/expression, and exclude only that selected property from the existing
source-ordered merged HTML-props path. Other attrs, spreads, class/style binds,
fallthrough and scope handling keep their existing ownership. Its children use
the existing traversal with nested list/conditional markers disabled and
comment children filtered. A missing tag emits the outer fragment. Ordinary
components, Transition, Teleport and current native L4 routing are unchanged.

The exact reported App.vue, Async.vue and issue body are immutable corpus inputs
under `tests/_fixtures/differential/compiler/ssr-builtins-7891/`. A separate
manifest registers this whole-SFC runtime pack without inflating the existing
single-row CLI adapter's acceptance count. Nine newly authored controls cover
explicit/default-only/missing-default/other/dynamic slots, static/dynamic/no-tag
groups, list and false-conditional ranges, comments and merged attributes. The
attribute control uses a bound style object; static-style normalization is not
part of this repair. Every input and the independently authored full-HTML
reference has a SHA-256 pin checked by the executed helper.

The ordinary Rust tests compile all eleven complete sources through the existing
SFC SSR API. Mapped/plain complete results must differ only in the requested
map field; module code stays equal. The runtime test retains every full current
result, original source, official complete module/helper/map and available
decoded map coordinate graphs. The existing current SFC API returns no map for
the three scriptless sources and only script-provenance maps for the other
eight; neither is evidence of template mappings. Map validation checks complete
canonical mappings and valid
source/generated positions; it does not claim original-segment semantic
identity beyond those structural checks. Both real default-component graphs
render all ten cases to twenty strict whole-HTML references with no runtime
warnings/errors. Transport rewrites only parsed import declarations to the
actual Vue/server/Async/helper bindings, never compiler text or expected HTML.
The packet remains in the existing automatic nextest worker artifact directory.
Independent source review found two concrete blockers before publication: the
lowercase builtin name incorrectly selected the custom-element attrs namespace,
and assertions could discard failing runtime evidence. The group-only const
path now omits the tag argument like the official transform, preserving the
ordinary path. An appended lowercase camelCase-spread SFC adds two full strict
observations; the earlier ten source payloads and eighteen expectations remain
unchanged. Rust retains the complete input/results, raw stdout/stderr, original
exit and literal Git source/tree/parents after the existing compilation
assertions and before runtime process or semantic assertions. A stdin-write
failure still reaps the child and retains its original outcome/streams before
asserting the transport failure.
Node records diagnostics before awaiting rendering and emits partial
modules/maps and the failing HTML/diagnostics on errors,
flushes both streams and exits nonzero. No failed comparison becomes success.

Official primary transforms were inspected at reported Vue `3.6.0-rc.10`:

- [Suspense transform](https://raw.githubusercontent.com/vuejs/core/v3.6.0-rc.10/packages/compiler-ssr/src/transforms/ssrTransformSuspense.ts).
- [TransitionGroup transform](https://raw.githubusercontent.com/vuejs/core/v3.6.0-rc.10/packages/compiler-ssr/src/transforms/ssrTransformTransitionGroup.ts).
- [Default-only server helper](https://raw.githubusercontent.com/vuejs/core/v3.5.35/packages/server-renderer/src/helpers/ssrRenderSuspense.ts).

Hosted execution uses the repository's existing pinned Vue `3.6.0-rc.9` and
plugin-vue `6.0.7`; the helper asserts and retains their exact identities. This
is Node SSR evidence. It grants no browser/hydration, client-backend, native
L4/default, compiler-history closure or performance credit. #6880 and broader
native replacement/legacy deletion remain unfinished. All old corpus pins,
level dependencies/defaults and protected 100+4 instruction ceilings remain.

At source preparation, Rust/native execution is unqualified. Independent source
review, fresh exact-source automatic Actions, all original differential/runtime
suites and protected full suites/ceilings are still required. The first v0.433
finite release admission hold remains: prepare source/Draft off queue, keep
#7891 open until both original failures actually merge, and hand the signed
terminal delivery to the release owner for the next frequent release.

First source Check `37323222769`, Rust build job `111807581667`, stopped before
execution on two unchanged strict Clippy policies: UTF-8 string indexing in the
quoted slot-key writer and collection indexing in the selected group property.
The causal successor strips the known outer quotes and uses checked original
property access, preserving emitted bytes, locations, all eleven inputs,
references and comparisons. This failed source supplies no execution credit;
fresh successor Actions remain required.

Successor Check `37324046914` builds the Rust archive and passes workers 1/2/3,
but worker 4 job `111814772169` rejects the new observer's incorrect demand for
a scriptless SFC map before any SSR render. The retained whole current results
and partial official graphs remain failure evidence, not twenty observations.
At actual unchanged main `8a8521d6897bbe3fd0af0cbfaebd83f4fc933933`,
[`compile.rs`](https://github.com/ubugeeei-prod/vize/blob/8a8521d6897bbe3fd0af0cbfaebd83f4fc933933/crates/vize_atelier_sfc/src/compile.rs#L153)
selects the descriptor-bound scriptless path;
[`template_only.rs`](https://github.com/ubugeeei-prod/vize/blob/8a8521d6897bbe3fd0af0cbfaebd83f4fc933933/crates/vize_atelier_sfc/src/compile/template_only.rs#L154)
returns `map: None`. The SSR template result has no map field and the existing
module-map assembly carries script provenance, not template anchors.
The corrected observer requires exactly `FallbackOnly`, `OtherSlot` and
`DefaultOnly` to have `map: null`, joins that list to each complete original
parsed descriptor, and still requires all eight scripted maps. It retains the
full public map fields, all eleven official maps, available decoded graphs and
their explicit dispositions. All original inputs, generated code paths and
twenty independent full-HTML references remain unchanged. No comparison failure
or unavailable map is credited as template-map readiness; fresh source Actions
and protected acceptance remain required.
