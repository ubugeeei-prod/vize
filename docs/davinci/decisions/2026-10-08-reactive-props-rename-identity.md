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

## Current source reproduction and correction

PR #8188 source `570de01bd80e297e3937ec05ac44619c2d6f0273` genuinely failed
Check 37642663121. Public declaration rename omitted the destructure key and
produced TS2339; local declaration rename additionally changed the public type
and Parent implicit value, producing TS2339, missing-required-prop and unknown
Parent value diagnostics. Failed Rust jobs 112869450696 and 112869450967 retain
complete references, WorkspaceEdits, applied/disk files and diagnostic arrays.
Only the first shorthand/LF session executed in each new wrapper before its
assertion; this supplies no acceptance for the remaining 22 sessions.

The correction keeps all exact reverse-mapped generated identities of a native
resolved authored declaration, so both exported component type and original
setup type reach the native property rename. Complete authored range roundtrips
and existing scope/mapping/batch gates still control every endpoint and edit.
Actual setup BindingIdentifier definitions suppress public endpoint discovery
for local queries; producer-owned component arguments retain the public role.
No posthoc name filter or source spelling search removes a wrong edit.

TypeScript native references at a shorthand binding declaration include its
source-property group even though rename there selects the local variable.
For reactive object destructure declarations only, the reference query therefore
uses a real generated IdentifierReference resolved to that exact local symbol.
Each materialized copy receives the same role selection; ordinary object and
import shorthand retain their prior route. Generated parser/value-resolution
failure after a proved reactive cursor returns an authoritative empty reference
answer. TS/TSX syntax and actual Canon projection controls accompany the native
24-session contract. Setup-role detection alone is not a malformed-parser
fail-closed claim; existing canonical failure/scope guards remain necessary.

Fresh exact-head source/native Actions must qualify the correction and every
unchanged authored vector before readiness, queue admission, merge or closure.

The derived LSP consumer inventory is regenerated with its unchanged generator,
adding only the ten source-import rows introduced by these helpers. The binding
role unit now compares the complete ordered authored range vector; the existing
assertion allowlist and all-target lint requirements remain unchanged. Source
head `0e8c56e874` stopped before the 24-session workers on those source hygiene
gates and a needless borrow in a new unit. It supplies no 24-session acceptance
credit; the corrected head must qualify afresh.
