# Selected original static event operands

Issues: [#6836](https://github.com/ubugeeei-prod/vize/issues/6836),
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6844](https://github.com/ubugeeei-prod/vize/issues/6844).

`NativeTemplateComponent::observe_attribute_handler` consumes an actual current
`NativeAttribute` projection from that original selected Component. The existing
Vue 3 directive provider selects only complete static `@name` / `v-on:name`
heads without modifiers. Dynamic arguments, object forms, modifiers, other
directives and incomplete values refuse before decoding or parsing. Original
recovery/extent and v-pre checks keep their existing error precedence.

The provider derives JS/TS exclusively from the original descriptor selection,
decodes the full value once and parses it once as real `Shape::HandlerBody`.
`NativeAttributeHandler` retains the original complete source/map, comments,
diagnostics, argument/value/name spans, actual body descendants and private
Origin membership. Reference expressions and inline or multiple statements stay
real original handler syntax; this syntax provider makes no runtime distinction
or identifier resolution claim. No Expr reparse or synthetic caller AST is used.

Moving the selected wrapper or parking/growing operand storage preserves the
private arena-backed original header origin. Short `admitted_for` joins only the
actual original selected owner/current attribute, its ordinal and grammar.
Sibling/nested headers and an independent equal-byte parse cannot confer that
origin. Local syntax/context/resource holes remain observable but expose no
admitted body. `into_syntax` explicitly drops attribute authority and keeps the
normally owned original syntax; the owner is neither Clone nor Copy.

Eight genuine selected-parser library laws cover both static head forms,
Unicode/entity/CRLF coordinates, multi-statement body identity, setup-only TS,
JS syntax observations, move/parking, foreign/sibling/nested custody,
unsupported no-parse forms, v-pre/recovered carriers and retained syntax/context
holes. Strict production Clippy and the private-origin rustdoc law pass locally.
Fresh exact-head Actions, the protected full/instruction queue and actual merge
remain required for both native Stack layers.

This is the actual L1 event dependency, not L2 legalization or product delivery.
The native Component pattern table and DOM target still refuse events. TODO:
same-owner File handler resolution with real local scopes and `$event`, attached
`ui.on` construction, owner-bound L3 event meaning and complete DOM runtime laws.
Modifier/dynamic/object event grammar and full dialects remain unimplemented.
No product route changes; #6836/#6838/#6844 and fix-history #6880 stay open.
