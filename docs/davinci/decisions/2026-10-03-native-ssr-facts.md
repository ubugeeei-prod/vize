# Native SSR eligibility in the shared decision walk

Tracking: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839), [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The SSR target needs actual server semantics rather than DOM classifications.
`build_decisions` adds its private SSR collector to the existing canonical
enter/binding/leave walk only for `TargetPolicy::Ssr`. Ordered parts retain the
original node references. No tree conversion, extra stage, parse, serialization
or second numbering walk is added. Neutral decisions and every DOM path remain
unchanged.

The bounded vocabulary admits ordinary lowercase native HTML, decoded text and
safe comments. It decides the void-element closing rule, refuses void children,
preserves authored root order, selects the single non-comment native root for
attribute fallthrough and marks multiple authored roots as a hydration fragment.
Native-element event listeners and cloak are omitted under the existing SSR
policy. Every other binding, expression/control/component/slot, non-HTML
namespace, raw-text/pre/template surface, duplicate/unsafe/special attribute
and unsafe comment is retained as an exact node/span typed refusal. Class,
style, value and reserved props remain refused pending actual normalization
semantics. L4 owns complete escaping and runtime spelling.

`NativeSsrFileAnalysis` has private fields and derives its only artifact from
one genuinely complete immutable `FileArtifact`; interrupted or erroneous files
fail before walking. Its retained owner cannot be dropped or replaced. This is
file admission, not original SFC custody or Vue runtime exposure. Bare native
`NativeAnalysis` remains a distinct sealed artifact boundary for the lower
canonical provider. No public SSR input accepts independently paired Region,
source, DecisionTables, scope, name list or legacy/Croquis facts.

Original-pointer/order/cardinality, target isolation, root/void/comment refusal
and incomplete-file laws validate this provider. Hosted exact-head checks and
the protected merge queue remain necessary through actual merge. No product
route, compiler-history #6880 gate or instruction ceiling changes. Full SSR
expressions, controls, class/style/model/slot/dialect semantics, original native
SFC entry, whole-product maps/performance and parity remain unfinished.
