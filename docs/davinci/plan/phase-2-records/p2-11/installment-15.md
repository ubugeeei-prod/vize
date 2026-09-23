# P2-11 Installment 15 — foreign namespace elements (2026-08-24)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4785](https://github.com/ubugeeei-prod/vize/pull/4785) moved SVG and
MathML native elements out of the local `Unsupported` bucket. The S2 DOM
emitter now threads the current parent namespace while emitting children:
`<svg>` and `<math>` enter block-local foreign namespaces, same-namespace
descendants stay inline VNodes, and Vue integration points such as
`foreignObject` re-enter HTML for their children.

The durable witness is
`crates/vize_atelier_dom/tests/davinci_s2_dom_namespace.rs`: five
S2-vs-shipped DOM lane comparisons covering simple SVG, SVG
`foreignObject`, nested SVG dynamic props, same-namespace dynamic
descendants, and MathML. The comparison is byte-for-byte including helper
usage.

This installment deliberately did not claim P2-11 completion. At merge
time, template refs, `.native`, filters, bind modifiers, and local
slot/outlet guard-only shapes still remained.
