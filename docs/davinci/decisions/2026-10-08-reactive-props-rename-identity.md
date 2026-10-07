# Reactive props destructure rename identity

Owning report: #7996; recursive component shorthand remains a separate #7994
qualification. Start from signed actual main `486c390d81643c16b865754239a987cfe4c9861e`;
the following signed `eed471b4` adds no rename/reference/fixture changes.

A public prop and the binding created by reactive destructuring have separate
rename roles. Public rename from either the Child type key or Parent attribute
must update the type key, destructure property key and matching attributes,
preserving the Child local alias and Parent local value. Local rename from
either the binding declaration or template value must update only that local
identity; same-name destructures expand to an alias, preserving the public key.

The complete original report and reported Child/Parent source are frozen under
`tests/_fixtures/differential/lsp/reactive-props-rename/7996`. The new actual CLI
stdio target `lsp_reactive_props_rename_cli` retains 24 separate sessions: four
query directions, shorthand/default/explicit-alias bindings and LF/CRLF. It
compares entire references and WorkspaceEdits, applies every returned edit to
complete files, then requires both resulting diagnostic arrays to be empty.
Existing original same-name, mixed-role and alias/external-edit controls remain
unchanged. This initial source records qualification intent; actual current-head
Actions must determine the remaining route defect and qualify any correction.

No old binary, predecessor green run, static concern or partially exercised
session supplies current-source acceptance. Native Stack/queue delivery, actual
signed merge and release remain separate responsibilities owned by the root.
The issue stays open until all reported directions receive genuine qualification.
