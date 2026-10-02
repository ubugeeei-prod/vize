# Retaining original parameter observations

Tracked with [#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
This adapts the existing L1 parameter handoff to the authenticated embedding
provider. It is part of that provider's functional integration: the parent must
compile with the current Params and Dense carrier APIs before publication.

`NativeSyntax::into_slot_params(self)` consumes its original private embedding
owner. The parser-owned `into_parameters(self)` uses the arena retained by the
original input; neither method accepts a substitute allocator. It moves the
same selected `FormalParameters` header into that arena while retaining its
original parameter/rest buffers, children and complete ordinary observations.
No AST cloning, reparse, new visitor, serialization or pipeline stage is added.

`RetainedSlotParams` owns the resulting `ParametersObservation` alongside the
existing checked `EmbedSource`, wrapper coordinates, explicit `SourceType`,
local admission hole and local diagnostics. Comments and complete parser
diagnostics remain normal owned observations. Their original pointers and
corrected decoded/authored maps survive consumption, including a local hole.
Safety/budget refusal retains its source without manufacturing an observation.

`admitted_parameters()` returns the original stock capability borrowed briefly
from the retained owner, only when no local hole exists. Public raw AST nodes,
source text, flags and numeric windows cannot construct that capability. The
existing raw parameter/rest references can live as long as the original arena;
this does not extend the lifetime of the authenticated observation borrow.
The selected formals, original content/window, explicit profile and default
parser options remain attached to the genuine owner.

A wrong syntax shape returns the entire original `NativeSyntax` boxed before
mutation. A defensive wrong embedding goal restores the same original owner
before boxing. Program AST buffers, complete comments/diagnostics, clean-parse
capability and explicit profile remain intact. A normally owned Program header
may move with that box; its original arena statement buffer and descendants
are the relevant stable syntax pointers.

The existing Vue Dense provider consumes these parameters and its collection
expression without allocator arguments. The two original parser observations
remain distinct normal owners. The moved existing context visitor now refuses
actual semantic `arguments`, `eval`, `yield` and their escaped spellings as
runtime formals; Dense retains both part owners and the complete whole source
on that refusal. Erased TS/signature distinctions remain the shared provider's
policy. Its stock projection does not establish duplicate-name freedom:
generic `(item,item)` parameter syntax remains accepted, with the canonical
For constructor's existing binding enumeration responsible for duplicate
preflight.

Meaningful current-source laws cover original selected Formals and descendants,
full diagnostics/comments through holes, unchanged entity maps and authored
spans, explicit JS/TS Module profiles, ordinary owner Drop, intact wrong-shape
Expr/Program artifacts, and the actual Dense strict-formal counterexamples.
Scoped source compilation and laws are not fresh whole-workspace Actions.

The intrinsic joint Dense whole-head/part association, actual Component file
and attribute event identity, live private file declarations, broader language
early errors, native control execution, and target emission remain unfinished.
Equal copied source text cannot establish original-file authority. Genuine
foreign-source regressions remain held until that separate provider exists.
