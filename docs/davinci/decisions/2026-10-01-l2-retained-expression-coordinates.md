# Retained expression coordinates (#6836, #6838)

Native L1 expressions use a private parser wrapper, and attribute decoding can
change source lengths. Passing their raw OXC offsets to the previous `JsExpr`
would select the wrong bytes. L2 now accepts the actual retained expression
through `JsExpr::from_retained_in`, with a checked neutral `JsCoordinates` view.
It neither parses nor walks the AST. The handed root pointer and all its child
references remain unchanged.

The bridge checks the authored UTF-8 range, the parser prefix, complete ordered
decoded/authored segment coverage and literal segment identity once. Empty
segments require exact authored text. Entity segments remain indivisible even
when their decoded and authored byte lengths happen to match. L1 owns HTML
decoding; L2 owns only this neutral correspondence vocabulary.

Every native consumer reads parser ranges through `ast_span_to_source` and
projects edits through `authored_span`. A wrapper-only range, UTF-8 cut, missing
map coverage or interior entity boundary is refused. `matches_authored_source`
checks the exact authored slice supplied to a writer without copying a file or
hashing it. Canonical artifact checks reject a native coordinate view attached
to different source bytes.

The ordinary `parse_in` load path retains identity coordinates and the same
admission and text behavior. `JsExpr` grows from 32 to 40 bytes on 64-bit targets;
the optional native payload is allocated only for a retained native expression.
This is a required source contract, not a measured performance improvement.
The existing exact-head instruction and allocation ceilings must pass without
being raised. A regression requires revisiting the representation.

Five coordinate laws and the unchanged canonical-provider laws pass against
the actual cached pinned OXC/L0 modules. Full workspace Clippy, target builds,
Actions, differential suites and protected queue validation remain required.
The actual L1 consuming handoff, native producer, identifier resolution and L4
writer are dependent work; this provider alone does not integrate a product.
Both roadmap issues remain open.

The current complete-expression guard refuses leading or trailing line comments
as `OutsideExpression`. L1 still retains their actual AST/comment observations;
admitting them requires a safe generated separator contract before this guard
can change. This bridge is not complete JS/TS grammar admission.
