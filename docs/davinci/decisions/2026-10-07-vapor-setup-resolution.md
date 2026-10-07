# Vapor filename recursion and setup directives (#7884)

Vapor asset resolution already receives the SFC filename-derived component
name and script-setup binding metadata. Use those owned facts: a matching
component tag (including its kebab-case spelling) marks the existing runtime
resolution call as a possible self reference. Imported setup components still
win first. Only experimental `Self` changes the authored resolution name;
ordinary filename recursion keeps the original tag and its source-map anchor.

Custom directives first resolve their `v` + PascalCase setup binding from the
existing setup metadata. Other directives retain normal registry resolution.
Allocate the `resolveDirective` helper only for an actual registry lookup;
builtin directives keep their existing paths. This adds no parse, stage,
serialization or legacy-backed level shortcut.

The complete original recursive Tree, a same-file-name imported-component
precedence control, its Child and mixed local/global directives are retained in
`tests/_fixtures/differential/compiler/vapor-setup-resolution/`. Existing
Actions Rust workers execute development/production × requested inline/separate
whole SFC modules, require equal complete map-on/off result fields, and mount
both source-built output and the locked official compiler/runtime against
independent whole DOM expectations. Recursive root prop updates must preserve
surviving node identity, detach the removed branch and reintroduce it correctly;
diagnostics and unmount are part of every row. Raw inputs/process outcomes are
retained before assertion. The pinned Vue runtime is currently 3.6.0-rc.9 while
the report uses rc.10; the latter remains a separate runtime qualification.

Custom-directive native admission and whole native SFC migration, SSR/hydration,
browsers and performance are not completion claims from this compatibility
slice. Exact-source Actions, protected instruction ceilings, actual merge and
installed publication remain pending. Global directives and experimental Self
controls retain their existing resolution contracts.
