# P3-6 — Retained custom directive runtime correction (2026-09-26)

The retained component lowerer now preserves custom directives after component
creation instead of dropping their binding value, argument and modifiers. Both
elements and components use the published runtime's `withVaporDirectives`
helper. Values and arguments are getters; absent operands remain absent. Modifier
keys and literal arguments are escaped as JavaScript strings.

Seven mounted scenarios compare complete DOM and functional directive traces
with the pinned official compiler. They cover elements, ordinary and dynamic
components, static and computed arguments, multiple directives, absent values,
absent arguments, modifier-only bindings, and a hyphenated modifier. Reactive
post effects observe updates; replacing the dynamically selected component
cleans up the old directive scope and applies it to the new root. Every scenario
checks teardown. Exact generated-code and runtime snapshots pin the contract.
VDOM object lifecycle directives are a separate runtime contract.

The former #1161 component payload-loss fixture is now required to pass. The
coverage runner has zero known-failure exemptions and floors of 459 VDOM,
117 Vapor, 167 SFC and 743 total passing fixtures. Only the two custom directive
fixture expectations changed, reflecting the verified runtime helper/getters
and resolution order.

Custom directives still select the explicit retained route. This corrects its
runtime behavior; it does not claim native S3 custom-directive admission or
P3-6's full acceptance.

Contract: [P3-6](p3-6.md).
