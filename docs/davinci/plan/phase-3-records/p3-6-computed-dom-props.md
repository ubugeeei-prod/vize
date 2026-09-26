# P3-6 — Computed DOM prop names (2026-09-26)

Ordinary DOM `:[name]` keys retain their S2 expression AST in the checked
payload. Every static attribute and bound prop on such an element belongs to
ordered `setDynamicProps` source groups. Keys and values remain separate typed
nodes; emission does not synthesize or parse `{ [name]: value }` JavaScript.
Static class/style keys merge within each literal group. Computed keys retain
their authored position, and object spreads divide the groups in authored order.

The retained lowerer uses the same structured groups for this surface. Static
attributes leave the HTML template so a key rename restores earlier values.
Static/computed duplicate identities are distinct, including an expression whose
raw text happens to match an attribute name. Static `:key` still belongs to its
loop; computed keys are ordinary runtime props.

Eight source fixtures cover reference, indexed, call and concatenated names,
static/computed spelling collisions, class/style merges, object spreads, input
values and loop aliases in both prefix settings. A checked-payload mutation
snapshot changes the key, value and static attribute while generation receives
an unrelated source. Six floor fixtures require zero legacy walks and zero
expression reparses in both prefix settings. All seven allocation ceilings stay
unchanged.

Three code and full decoded map snapshot pairs pin equality with the retained
lane. This preserves existing template and render mapping units; it does not
add individual key/value anchors to the merged-prop call.

Five TS-33 tests compare twelve mounted scenarios with the pinned official
compiler and runtime: both static/computed authored orders across four name
forms, class/style collisions and restoration, merging across object sources,
browser-visible input value restoration, and loop aliases through keyed
reorder/removal. Three malformed HTML cases pin parser diagnostic code, message
and source location across both lanes. Exact traces include repeated
updates, removal, stable node identity and unmount. Retained code equality is
proved separately by the source and map fixtures.

Computed prop modifiers, named or object event listeners beside these groups,
model/content/show directives, once subtrees and computed props on built-in
components remain explicit legacy surfaces. Computed slot outlet props have
their own [contract](p3-6-slot-props.md). This bounded
contract does not close P3-6's complete surface or performance acceptance.

Contract: [P3-6](p3-6.md).
