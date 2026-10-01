# Component definition hovers

For [#7319](https://github.com/ubugeeei-prod/vize/issues/7319), a component tag
hover shows the imported SFC definition's props, emits, slots and models with
their declared types. It uses the same contract analysis as an imported script
identifier, including unsaved child buffers and barrel/alias resolution.
TypeScript fenced code enables the editor's syntax colors.

Usage-side passed props, listeners, provided slots and missing-prop examples
are removed from this hover. An unresolved component does not fabricate a
definition contract. Single model contracts use object syntax so the complete
display remains valid TypeScript.

The authored Parent/Child fixtures under
`crates/vize_maestro/tests/fixtures/component-definition-hover/` cover the
declared contract, unsaved edits and an aliased kebab tag through a barrel.
Exact-head LSP tests and full merge-queue checks remain required.
