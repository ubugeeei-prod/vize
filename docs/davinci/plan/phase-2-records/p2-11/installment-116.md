# P2-11 Installment 116 - Custom Element Patterns

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5806](https://github.com/ubugeeei-prod/vize/pull/5806), merged
> 2026-09-06 as `799a6d54b`.

This installment lets declarative custom-element patterns enter S2 DOM
production selection. Pattern-backed matchers can now be projected into
`DomEmitOptions`, so matching non-native tags lower as ordinary DOM elements
on S2 and still produce the same render bytes and section boundaries as the
compatibility lane.

Opaque predicate matchers remain outside the S2 production selector. The
boundary stays explicit: a string or wildcard pattern is serializable
production surface for S2, while a callback predicate is executable host logic
and therefore keeps the compile on the compatibility path until a later
reviewable contract exists.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_custom_element_selector.rs` compares
template and SFC section compiles for an `ion-*` matcher against compatibility
output and asserts the selected compile records one `davinci.s2_dom.files`
counter. The same file pins the callback-predicate case as compatibility-only.

`crates/vize_s1_to_s2/tests/emit_comp/basic_and_control_flow.rs` covers a
matching `ion-*` tag emitted as an element rather than a component, and
`tests/tooling/davinci-dom-production-boundary.test.ts` records
`custom_element_patterns` as part of the S2 DOM emit option surface.

This installment does not tick P2-11. The full production-lane switch remains
open because opaque custom-element predicates, unsupported option shapes and
the explicit legacy flag still require the compatibility path.
