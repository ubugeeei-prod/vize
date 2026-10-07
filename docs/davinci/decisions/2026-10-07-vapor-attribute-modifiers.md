# Vapor static attribute force modifiers (#7888)

The reported original App loses `input[value]` because retained Vapor lowering
forgets `.attr`, and static attributes after an object lose their authored merge
position whenever a force modifier prevents native admission.

Keep forced attribute keys in the runtime's existing `^key` representation and
consume that prefix before selecting `setAttr`. `.prop` retains `setDOMProp` and
wins when both force modifiers occur. The public IR layout stays unchanged.
For a DOM element that mixes an object with static `.attr`/`.prop` bindings,
retain every attribute in the existing ordered merged-prop operation; forced
keys use the runtime's `^key`/`.key` vocabulary. The selected native emitter
admits these static modifiers and emits the same operations from its owned
operands. No retained-level fallback or new pipeline stage is introduced.

The whole original App and a reactive control are retained in
`tests/_fixtures/differential/compiler/vapor-attribute-modifiers/`. The latter
checks both source orders, property versus attribute writes, updates and DOM
identity. Existing Actions Rust workers execute development/production and
requested inline/separate SFC rows against both Vize and the locked official
Vue Vapor compiler/runtime, with independently authored DOM expectations and
raw failed-process capture. Native tests require actual admission and matching
retained operation bytes. The pinned runtime currently uses Vue 3.6.0-rc.9;
the reporter used rc.10, so later runtime qualification remains separate.

Computed modifier keys, component force modifiers, `.camel` mixed with objects,
whole native SFC migration, browsers, SSR/hydration and performance are outside
this static reported slice. Their unsupported admission routes remain explicit.
Source Actions, merge queue instruction ceilings, actual merge and installed
release verification remain pending. No historical green or fixture authoring
is treated as a successful runtime execution.

Initial source `3192942496` Check `37589873592` rejected identical forced-attribute
and SVG class/style Clippy branches before Rust execution. The correction joins
the conditions with the same single `setAttr` body; fixtures and runtime
expectations stay unchanged. Shared security advisories remain a separate gate.
