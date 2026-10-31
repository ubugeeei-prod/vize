# Logical workspace revision protocol

`input.json` contains the exact authored main, support, virtual source, original
configuration and four valid/error/repair/close phases compiled into the attached
Rust observer. Unicode line separators and CRLF are intentional. The Rust helper
records this input in a new `attached-observer-input.json` before the attached
workspace is spawned, and as `attached.observerInputDependencies` in the completed
packet on both source roles. The helper itself remains the sole Rust
`observerDependencies` item.

`source-authority.json` pins the eight production preimages/current sources and
the actual observer/helper consumer commit. The native workflow's historical
BEFORE source is distinct from the production preimage revision. Its four roles
are BEFORE/AFTER crossed with stable/beta. The unchanged first 12,150 observer
bytes retain the original core API comparator; only the later attached routine
consumes this input.

This is a protocol registration, separate from authored Batch fixtures and their
dialect counts. Source custody and pure refusal tests grant no native admission.
All four real DTO roles and complete packets still need their own Actions proof.
Only `input.json` is a copied runtime input; this authority record and README are
planner documentation, not compiled Rust dependencies. The historical 10731
protocol row, wire inputs and source-pairs record remain unchanged.
