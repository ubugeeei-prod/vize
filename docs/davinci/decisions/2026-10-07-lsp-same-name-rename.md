# Same-name template bindings during rename

Owning reports: #7994, with reactive-destructure consistency tracked in #7996.

## Decision

Use native symbol selection to discover edits. Parse the retained authored Vue
directives only to recover whole same-name `v-bind` spans and their existing
implicit expressions. A local variable rename expands `:label` to
`:label="title"`; a public prop rename expands it to `:heading="label"`.
Preserve the directive spelling, modifiers and the parser's camelized implicit
value for kebab arguments. Do not discover edits by spelling or sweep other
components. A binding declaration in a reactive destructure selects the local
role even when that one token also names a public prop.

Coalesce complete duplicate edits after expansion. Reject the complete
transaction if native aliases still project conflicting replacement text,
reversed coordinates or overlapping edits onto an authored file. Keep native
scope admission, document versions, annotations and authoritative refusal.
There is no additional native RPC or project source inventory.

## Validation and remaining work

The exact original #7994 report and Child/Parent source bytes are retained in
`tests/_fixtures/differential/lsp/same-name-rename/7994/`. Five Rust laws cover
local/property geometry, native/component directives, modifiers, reactive
binding role, CRLF/astral offsets, truncated aliases, boundary insertions and
plain/annotated conflicts. Four source-built stdio sessions require complete
original local-label/native-id edit vectors, applied whole source, unchanged
Child bytes and empty diagnostics after applying the edits.

At source preparation, Rust formatting and whitespace checks pass. Compilation,
source Actions, protected suites, merge and public package acceptance are
pending. The native prop direction, original multi-component/kebab control,
reactive-destructure declaration/attribute parity (#7996), event/slot/type-alias
linkage (#8011) and non-bundled runtime provenance (#8010) remain separate
qualification obligations. No issue-completion, latency or typecheck speed
claim follows from this preparation. #4075's standard tsgo Content Mapper lane
still requires upstream support; the existing Vize bridge variants remain
covered independently.
