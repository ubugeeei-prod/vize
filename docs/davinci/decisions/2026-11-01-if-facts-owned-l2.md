# Owned conditional key facts in L2

Related: #6838, #6839.

L2 owns `analysis::if_facts::{IfFacts, BranchKey, BranchKeyKind}` and the
receiver that extracts a key from a completed `IfBranch`. The receiver obtains
the carrier extent and single root from that same branch. It cannot be called
with a separate branch span or arbitrary list of ops.

The structural lowering constructs the real branch before calling this
receiver. A captured template-wrapper key retains its existing lazy precedence:
the inner carrier is untouched when the wrapper supplied the key. Otherwise,
only an Element, Component, or Slot whose span equals the branch span can supply
the fact. The earliest authored key still wins; static keys leave the attribute
surface, while dynamic bindings stay at their original index and keep their
retained expression.

DOM key emission and SSR key-surface exclusion import these L2 types directly.
Vapor retains its existing key-bearing control-flow refusal over the same
canonical rows. Input-specific wrapper capture, original page-order NodeId
publication, diagnostics, and provenance remain in the L1-to-L2 adapter.
Existing `lower`, `pass`, and `pass::vif` public names reexport the same types.

## Compatibility and remaining ownership

The exact fields, derives, owned heap values, and static lifetime are retained.
`IfFacts` remains 24 bytes and `BranchKey` remains 48 bytes on 64-bit targets;
`IfOp` and `IfBranch` keep their existing 32/48-byte layout guards. The per-file
storage inventory moves the measured schema and receiver sites to L2. Splitting
the imports adds one direct Vec and String path without changing the total
bound uses, allocation policy, or arena/heap boundary.

The transitional `Lowered.if_facts` SideTable, compatibility clone, and
legalization rekeying remain unchanged. Canonical `ArtifactParts` keeps its
existing four fields and complete Debug representation. Moving this legacy
side table into the sealed Artifact, admitting native source keys, and making
L3 the shared decision owner for every backend remain separate work. This
change introduces no native admission, synthetic source owner, or empty table.

## Preserved behavior

Existing complete facts, ordered diagnostics and provenance, Folio snapshots,
DOM/SSR output, 15 official key-mode controls, slot refusals, unsupported census,
and pass/walk budgets remain the oracles. New receiver boundary tests exercise
the real generic IR and retained JS expressions; a real lowering integration
law checks the canonical type through every old public alias. Generic IR tests
do not claim native source ownership.
