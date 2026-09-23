# P2-11 Installment 119 - HTML Reentry Close Casing

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5826](https://github.com/ubugeeei-prod/vize/pull/5826), merged
> 2026-09-06 as `263cb45bf`.

This installment keeps the SFC production selector's source scan aligned with
HTML close-tag casing after foreign-object re-entry. HTML tags close
case-insensitively, but SVG and MathML tags stay case-sensitive; the selector
now uses that namespace rule before deciding whether a later dynamic SVG node
still belongs to the foreign subtree.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_sfc_namespace_selector.rs` compares
an `<svg><foreignObject><div>...</DIV></foreignObject>...` section compile
against the compatibility lane and asserts the selected compile records one
`davinci.s2_dom.files` counter. The implementation change is limited to
`compile::sfc::selector::pop_closed_tag`.

This installment does not tick P2-11. The full production-lane switch remains
open because opaque custom-element predicates, unsupported option shapes and
the explicit legacy flag still require the compatibility path.
