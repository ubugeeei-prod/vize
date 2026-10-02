# Component definition hovers

Full workspace validation also covers native hover tests. Their filesystem and
hover setup helper returns contextual errors; assertions remain in the actual
test functions, so the production panic lints do not need an exemption.

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
The packaged VS Code host compares all three hover locations with the same
complete TypeScript contract, including the tag and the object-shaped model.
The real-server Neovim scenario retains the same exact contract for its
import and script references.
The TypeScript Vue plugin also emits a valid intersection contract and includes
every typed model declaration. Its language-service test parses the displayed
contract as TypeScript to protect syntax highlighting and checks default and
named models together. Editor scenario policy checks require definition-side
slots and reject the retired usage card.
Runtime contract keys use TypeScript string-literal names when they are not
simple ASCII identifiers, including colon-bearing emits and hyphenated props.
Model names are always escaped string literals. The native runtime-declaration
fixture covers those names and a quoted model; the TypeScript plugin also shows
an untyped model as `unknown` and parses its escaped contract in the service test.
