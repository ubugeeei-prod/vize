# Conditional template text retains its hydration range

[Q16solver reported #11920](https://github.com/ubugeeei-prod/vize/issues/11920)
from [Esposter](https://github.com/Esposter/Esposter/blob/cfc73c9218aee89e1042d0cdc686acefded4105b/apps/web/app/components/Message/Model/Message/Emoji/Tag.vue#L23).
A conditional template with one text or interpolation child compiles to a client
Fragment, so SSR must write `<!--[-->` and `<!--]-->` around the selected branch.
Vue's `processIfBranch` in `@vue/compiler-ssr` applies this rule to non-element
children, while allowing a single loop to own its own range.

The retained template wrapper hid that single text child from the legacy SSR
shape check. The string-plan emitter separately treated a single text segment
as an element. Both emitters now retain text branch ranges. Single element and
loop branches keep their existing ownership.

The public 0.691.0 CLI reproduces the original SFC as ``_push(`hi`)`` without the
anchors. `tests/_fixtures/differential/compiler/ssr-template-text-fragments-11920`
retains that original SFC and the complete corrected SSR module. Its sole output
change is ``_push(`<!--[-->hi<!--]-->`)``, justified by Vue's client Fragment.

`nested_branch_fragments` adds text, interpolation, else, root, mixed content,
and single element controls on both SSR routes, with source map parity. The
existing scoped CSS Actions lane executes the complete outputs against official
Vue, checks exact HTML for both boolean inputs, and hydrates the six controls
with actual client SFC compiler outputs. It checks the entire resulting DOM,
original node identity, and an empty warning stream. Complete compiler and
runtime captures remain Actions artifacts.

After release, run the issue's original `npm run ssr` and `npm run hydrate` in a
fresh project with both `@vizejs/native` and `@vizejs/vite-plugin` pinned to the
new release. The expected SSR HTML is
`<div class="box"><!--[-->hi<!--]--></div>`. All four server/client combinations
with `@vitejs/plugin-vue` must hydrate without warnings and preserve that HTML.
Retain 0.691.0 as the failing published control; the single `<b>hi</b>` branch
remains the control that must not acquire an extra fragment.
