# Compile waiver ledger

Charter #23 holds VDOM and SSR to corpus byte-identity of emitted output; a
change to emitted bytes needs a reviewed waiver that documents why the new
output is correct. Each entry names the shape, the old and new output, the
rendered effect, and the evidence. An entry closes when the maintainer's
veto window on its PR passes and the corpus baseline is re-recorded with the
new bytes; the ledger must be empty at each phase exit.

Rendered evidence runs the real Vue 3.5 server renderer over three modules
per fixture — `@vue/compiler-ssr` 3.5's output, the aligned vize output, and
the pinned pre-fix vize output — and requires the aligned HTML to equal
upstream exactly (attribute order inside a tag normalized) while the pre-fix
module threw or rendered different HTML
(`crates/vize_atelier_ssr/tests/vue_ssr_render.rs`, fixtures in
`tests/vue_ssr_render/fixtures.rs`, harness
`tests/tooling/support/ssr-vue-render-diff.mjs`). The S4 plan lane emits the
same bytes (`s4/differential_tests/vue_align_fixtures.rs`, four option sets).

## Open: SSR Vue 3.5 alignment (P3-8, `fix(ssr)!`)

Approved by the orchestrator on the maintainer's behalf on 2026-09-22 and
cleared for merge by the maintainer.

| id       | shape                                      | pre-fix output (effect)                                                      | aligned output                                                                 | render fixture                  |
| -------- | ------------------------------------------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ------------------------------- |
| W-SSR-1  | custom directive on an element             | `_ssrGetDirectiveProps(_ctx, _directives, "x")` (`ReferenceError` at render) | merged `_ssrGetDirectiveProps(_ctx, _resolveDirective("x"), value, arg, mods)` | `directive-value-arg-modifiers` |
| W-SSR-2  | custom directive on an empty element       | same crash; directive content ignored                                        | `_temp` owns `textContent` / `innerHTML`                                       | `directive-owns-empty-content`  |
| W-SSR-3  | custom directive on a `<textarea>`         | same crash                                                                   | `_temp` owns the `value` content, `"textarea"` tag argument                    | `directive-owns-textarea-value` |
| W-SSR-4  | custom directive on the fallthrough root   | same crash                                                                   | directive props merge after `_attrs`                                           | `directive-on-fallthrough-root` |
| W-SSR-5  | directive bound in `<script setup>`        | same crash                                                                   | `$setup["vX"]` (inline: `vX` / `_unref(vX)`)                                   | `directive-from-setup-binding`  |
| W-SSR-6  | `v-once` / `v-cloak` / `v-memo` on element | treated as custom directives (crash)                                         | dropped, as upstream                                                           | `builtin-once-cloak-memo`       |
| W-SSR-7  | inline `.camel` bind                       | kebab attribute name (`view-box`)                                            | camelized name (`viewBox`)                                                     | `inline-camel-bind`             |
| W-SSR-8  | inline dynamic-key bind `:[k]`             | key dropped (`_ssrRenderAttrs(value)` spread the value)                      | merged props object `{ [_ctx.k \|\| ""]: v }`                                  | `inline-dynamic-key-bind`       |
| W-SSR-9  | object spread on a non-root element        | spread rendered beside inline attributes (duplicate attributes)              | one merged props object, source order                                          | `inline-spread-collision`       |
| W-SSR-10 | object spread order (root or merged)       | spreads first, wrapped in `_normalizeProps(_guardReactiveProps())`           | source order, unwrapped; later props win as upstream                           | `root-spread-source-order`      |
| W-SSR-11 | `v-show` on the fallthrough root           | style merged before `_attrs` (parent style order reversed)                   | `{ style }` merged after `_attrs`                                              | `root-v-show-after-attrs`       |
| W-SSR-12 | `.prop` / `.attr` on a merged element      | client `.` / `^` prefixes reach `_ssrRenderAttrs` (attribute lost)           | plain attribute name, as upstream `transformBind` `inSSR`                      | `merged-prop-attr-binds`        |
| W-SSR-13 | merged `<textarea>` (spread / root)        | `value` rendered as an attribute beside the authored text                    | `_temp` owns `value` as content, `"textarea"` tag argument                     | `merged-textarea-from-spread`   |
| W-SSR-14 | `<slot :[k]>` outlet dynamic key           | `[k \|\| ""]` unprefixed (`ReferenceError`)                                  | `[_ctx.k \|\| ""]`, the local inside a scope                                   | `outlet-dynamic-key`            |
| W-SSR-15 | component / outlet `:[k]` inside a scope   | `_ctx.k` for a `v-for` alias or slot param (wrong key)                       | the scope-local `k`                                                            | `component-dynamic-key-in-loop` |
| W-SSR-16 | `<input v-bind v-model>` without a type    | `value` attribute; a spread `type="checkbox"` never checked                  | `_ssrGetDynamicModelProps(existing, model)`                                    | `input-spread-model`            |

Unchanged, recorded so review can see the remaining distance to upstream
bytes (render-identical or pre-existing, not part of this waiver): merged
`class` / `style` keys stay last in their object (attribute order only);
`_ssrGetDynamicModelProps` repeats the existing-props expression instead of
upstream's `_temp` sequence; `_resolveDirective` is inlined like
`_resolveComponent` instead of hoisted; under binding metadata a `v-bind` /
`v-on` dynamic key stays `_ctx.<key>` where upstream prefixes it like a value
(`$setup.key`; the render context proxies setup state and props, so the
HTML matches); a dynamic key or directive argument naming a JavaScript
global (`:[Math]`) is spelled `_ctx.Math` where upstream leaves it bare.
