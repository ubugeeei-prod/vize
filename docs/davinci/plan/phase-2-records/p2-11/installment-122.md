# P2-11 Installment 122 - Custom Element Predicate Selector

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5854](https://github.com/ubugeeei-prod/vize/pull/5854), merged
> 2026-09-06 as `ae0721dba`.

This installment removes the last custom-element selector fallback from the S2
DOM production surface. Static predicate matchers now project into
`DomEmitOptions`, and S2 custom-element classification accepts either a
matching declarative pattern or the static predicate result before deciding
whether a tag emits as a native element or a component.

The same selector hardening lets parser-recovered SFC section compiles keep
using S2 after the compatibility parser records diagnostics for non-void HTML
self-closing tags. The diagnostic still comes from the compatibility parser;
the recovered render bytes and section boundaries come from the S2 production
lane.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_custom_element_selector.rs` compares
template and SFC section compiles for declarative patterns and static
predicates against compatibility output, then asserts the selected compile
records one `davinci.s2_dom.files` counter.

`crates/vize_atelier_dom/tests/davinci_s2_sfc_self_closing_selector.rs` pins
HTML roots plus SVG and MathML HTML re-entry cases that keep compatibility
diagnostics while emitting through S2. The stage-option unit tests assert
opaque static predicates are S2-supported, and
`tests/tooling/davinci-dom-production-boundary.test.ts` records
`custom_element_predicate` as part of the audited S2 DOM emit option surface.

## Remaining blocker

P2-11 remains open because the explicit `VIZE_DAVINCI_DOM=legacy` selector is
still a phase-live compatibility path. The full production-lane switch needs a
reviewed flag-deletion PR.
