# Single-root fallthrough (#8625)

Preserve Hebilicious's #8620 root selection and all focused single, keyed,
conditional, nested and sibling controls. Move its root collector to a small
module to keep generate.rs inside the existing 350-line source budget. Both
recursive function bodies and stack protection remain unchanged. The composed
slot and root fixes also move the existing scope lookup into context/scopes.rs
without changing its resolution order or body; context.rs stays below the same
source limit. Exact trailing source newlines are retained in JSON inputs and
decoded without trimming.

The historical capture-b11eb and its five expected files remain immutable.
The root-document-order fixture gets one explicit current successor: only the
first sibling component's root flag changes from true to false. An independent
whole-packet test authorizes precisely that literal change. The original history
integrity, all original options, all five live whole-output comparisons and all
Davinci unsupported coordinates remain enforced. The differential manifest pins
both the historical reference and its current successor with their hashes and
the defect link. No case is removed or marked inactive.

The reporter's original whole FallthroughOuter and FallthroughInner SFCs are
also registered as a runtime corpus. Explicit owner props must land on the
section and never on its nested span, with complete locked Vue VDOM parity and
unmount coverage. Typed-handler snapshots in the next Stack layer must inherit
the corrected sibling-root flag.

Exact Actions, protected merge and installed release acceptance remain pending
until their actual executions succeed.
