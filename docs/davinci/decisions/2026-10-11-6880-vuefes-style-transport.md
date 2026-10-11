# VueFes generated stylesheet transport (#6880)

[Compiler fix-history issue #6880](https://github.com/ubugeeei-prod/vize/issues/6880)
owns the complete pinned heading fixture. This record extends the
[SSR dependency decision](./2026-10-11-6880-vuefes-ssr-style-dependencies.md)
with two additional stylesheet transport corrections. The first source qualifier
proved that retaining SSR imports alone does not repair the application cascade.
The unchanged original VRT on `e23560400b7642e4925c98e6da513b628ce64709`
([run 38117641371](https://github.com/ubugeeei-prod/vize/actions/runs/38117641371))
failed all three dev-home retries at 0.1235105465271594 against the original
0.002 budget; 37 subsequent cases were not run.

## Two source-bound failures

The actual literal-H component response from the bounded original runtime
capture imports
`/_nuxt/components/heading/VFHeading.vue.__vize_style_0.css?vue=&type=style&index=0&scoped=data-v-98cff51b&lang=css&vize-file=…`.
The complete component module has SHA256
`0ac1737999e8bb5a84be141e41ddc9af1952df056a4ed3c66733937150d9d988`,
and the corresponding actual stylesheet module was served with HTTP 200.
Both remain in original runtime artifact 11693093826. The original stylesheet
order and all fourteen correct native heading scope attributes are retained in
the preceding decision; a missing client scope attribute is not the cause.

Nuxt's production `render:response` plugin calls
`sanitizeNuxtDevStylesheetLinks`. Its source is byte-identical on the original H,
fresh `f3c340a0df`, and `e23560400b7642e4925c98e6da513b628ce64709`:
SHA256 `fd7eb3597d144e25ae8d14b99a43c588f473bf18232ab8387acc59a29034d39f`.
Passing the complete generated link to that actual function removes it: its
allow-list accepts global assets and single-segment CSS filenames, but rejects
the nested generated SFC stylesheet path. The module installs this response
plugin when the compiler and default development stylesheet filtering are active.
This is a direct source-bound reproduction using the actual served URL, not a
claim that the original application's raw SSR HTML was captured.

The fresh source fixture separately retained a real native SSR style dependency
but recorded HTTP 404 for its browser stylesheet request. The complete URL starts
`/VFHeading.vue.__vize_style_0.css?direct&vue=&type=style&index=0&…`.
Vite's stylesheet middleware inserts `direct` before existing query parameters
for `Accept: text/css`. The native adapter's `normalizeStyleVirtualId` accepted
only a literal `?vue`, so this request kept the synthetic filename instead of
recovering its authored SFC from `vize-file`. The identical query with `vue`
first recovers the source. The whole 404 responses, headers and empty bodies are
retained in source artifact 11694287884, SHA256
`75a5082e87c8303d19160befa66193f7a1bfa9ecda1110826f1ae7e37c4d9c2c`.

## Essential correction and regression

The Nuxt filter accepts the canonical generated
`*.vue.__vize_style_<numeric index>.css` path, including nested directories.
Its existing null-byte, decoded traversal and duplicate-link guards still apply.
Unrelated nested CSS paths remain rejected. The Vite normalizer checks for an
actual query and the `vue` query key, regardless of parameter order; it preserves
the complete original query bytes and authored source path. Lookalike `vueish`
and `notvue` keys do not classify a request as Vue.

The small source change touches these two existing helpers. It does not add a
CSS reparse or compiler stage. Full HTML regressions preserve the actual heading
link's position after global CSS, its complete query and the entire body. They
cover custom asset bases, generated Unicode filenames, duplicate links,
traversal, null bytes, nonnumeric style indices and unrelated nested styles.
The actual fresh 404 URL is a separate Vite normalization regression.

The whole pinned runtime fixture keeps its original SFC and asset closure while
preserving the source's `components/heading/` hierarchy. Native transport uses
the actual Nuxt sanitizer on complete generated HTML. Stylesheet dependencies
are emitted with their original graph URLs; real browser requests let Vite
insert `direct` itself. Successful stylesheet responses must be HTTP 200 CSS.

The response observer retains each status, all headers, request type and redirect
origin, body or explicit body-null API error. Its tasks settle before and after
browser closure, so late failures cannot discard the packet through an unhandled
rejection. Complete DOM/CSS observations precede the response completeness gate.
That gate still fails on an unavailable body; redirects receive no exemption.
The CSS-order counterexample moves the complete already-injected global sheet
after heading CSS and waits for style invalidation. Original HTTP links and all
rules remain in the full observation.

Local source tests passed all 17 cases, including the existing HTML edge guards.
An isolated official-only control on exact Vue 3.6.0-alpha.2 and stock plugin-vue
retained full SSR, cold DOM and hydration, both with 20px/34px headings and ten
successful complete responses. An authored HTTP 302 control retained the explicit
body-null reason and followed 200 response without an unhandled rejection; its
strict completeness result remains false. These are local harness controls and
do not qualify the freshly built native producer.

## Delivery and remaining acceptance

This is the stylesheet transport child of the SSR dependency slice. The next
dependent compiler slice owns the separately proven eight duplicate native SSR
scope attributes. Whole native/official SSR strings remain compared without
normalization, and cold DOM and hydrated DOM retain their full comparisons.

The dedicated qualifier runs once on the composed Stack's literal head, reusing
the unchanged original 38-case VueFes VRT and one native build. Its original
order, screenshots, deadline and budgets remain the application acceptance
contract. Intermediate layers receive ordinary PR checks. Actual full VRT,
protected merge and released-consumer proof remain outstanding; local controls
and this decision do not close #6880 or establish release readiness.
