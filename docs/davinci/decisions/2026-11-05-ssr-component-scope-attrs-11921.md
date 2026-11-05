# SSR scope ownership stays outside component props

[Q16solver reported #11921](https://github.com/ubugeeei-prod/vize/issues/11921):
a scoped parent renders a component with `inheritAttrs: false`, which forwards
`$attrs` through a slot to another child. The parent scope ID reaches that child
in SSR and survives hydration, but is absent in a client mount. Its scoped CSS
therefore styles an element the parent does not own.

`ssrRenderComponent` creates a VNode in the current rendering scope. The runtime
already retains that scope independently of props. Explicitly adding the scope
ID to component props puts it into `$attrs`; forwarding it defeats the runtime's
component and slot boundaries. Dynamic component SSR already left this to the
runtime. Both static emitters now do the same. Physical elements still carry
their own scope attributes, and slotted scope IDs keep their separate path.

Three live snapshots change for this reason: static root, static nested, and
forwarded layout component calls lose the generated scope prop. The forwarded
layout also loses its now unused `mergeProps` import. Other props, slot output,
and physical element scope attributes stay byte-identical. Historical producer
and public acceptance evidence is preserved.

The source-bound test retains the original four SFCs and compares complete
public results, including source maps, between selected and retained SSR routes.
It executes the component graphs through pinned Vue 3.5.35 and compares the whole
server DOM with the official compiler. Chromium then compares complete hydrated
and client DOMs, warnings, original element identity, section attributes, and
the actual computed font style. Controls cover no forwarded attrs, dynamic root
components, and ordinary attribute inheritance. Reintroducing the old props
merge is a negative control: the section regains the parent's scope attribute.
The test loader supplies the same `__scopeId` host metadata as the Vite adapter.

The differential input retains the original App source under the adapter's
`Layout.vue` filename, its complete corrected SSR module, and unchanged CSS.
The published 0.691.0 CLI reproduces the unwanted nested `mergeProps` before
the fix. A separate reference-only self-test qualifies the new runtime oracle;
source-built acceptance comes from Actions.

After release, run the original report's `npm run check` in a fresh project
with `@vizejs/native` and `@vizejs/vite-plugin` pinned to the new version. The
section must carry only Panel's scope in server output, after hydration, and in
a client mount. App's scoped `.leaf` rule must match in none of those modes.
