# Checked native construction (#6838)

The arbitrary-parts `Artifact::try_new` constructor validates an existing tree.
Applying that constructor after a fused L1-to-L2 producer would add another full
tree stage. A restricted `Builder` instead checks source ranges and creates
canonical nodes during the producer's own construction walk.

The first factories create elements, components, static attributes, text,
comments and retained-expression interpolations. They mint page-preorder ids
before child construction, check UTF-8 and immediate source ownership before
allocating a node, and preserve producer-order provenance, including failed
decisions with no node. There is no arbitrary region or side-table insertion.
Scope-introducing operations, bindings and control flow need their actual
checked factory contracts before they are added.

Child callbacks receive a borrowed `RegionBuilder` with a private owner, rather
than a mutable owned `Builder`. Safe replacement of the entire builder therefore
cannot evade its checks. Source or expression rejection consumes no node id and
keeps already supported nodes. A callback that unwinds leaves pending frames;
`finish` refuses a seal and returns every partial owner/child/provenance record.
Only pending owner frames are closed to retain those nodes; no tree is walked.
If an outer callback catches a nested unwind, its constructor checks the actual
top owner id before closing anything and returns that pending owner's error.
Both owners and their original partial children and provenance remain intact.

On successful construction, `finish` directly creates the immutable Artifact
from the same accounting. It performs no expression parse, serialization,
validation walk or second node-count derivation. The native L1 consumer will
carry explicit typed holes and diagnostics beside supported partial artifacts;
this factory provider itself neither calls a parser nor switches a product.

Six new factory laws cover exact page order, atomic rejection, child/attribute
ownership, failed provenance and direct or nested caught callback unwinding.
The independent arbitrary-parts checker agrees with the factory result in test
code only. The nested-unwind regression fails the original factory, which
incorrectly returns outer success, and passes the checked pending-owner guard.
The reviewed storage rows describe the construction stack, its arena children,
and these test-only observers; scanner policy and every budget stay unchanged.
Full Actions, protected queue checks and actual native-producer measurements
remain required. Scope resolution, remaining native operations, all dialects
and product integration are unfinished; #6838 remains open.
