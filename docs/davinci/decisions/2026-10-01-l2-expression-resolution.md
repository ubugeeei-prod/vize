# Retained expression resolution

Paired issue: [#6838](https://github.com/ubugeeei-prod/vize/issues/6838).

`vize_l2::resolution::resolve_expression` walks the retained `JsExpr` AST.
Every AST byte range passes through the checked wrapper-coordinate bridge;
stored occurrences are decoded-relative. The result keeps the identical AST,
source and authored-coordinate capability. Nothing parses or serializes again.

The enclosing binder supplies explicit stable `BindingId` values through
`BindingLookup`. No missing name silently becomes a context binding. L2 records
semantic identifier names, exact byte spans, read/write/read-write roles,
object shorthand and constructor-callee context. Static property keys, string
contents and regex contents do not become references. An assignment to a member
reads its object/key; it does not assign the object binding itself.

The supported family covers primitive literals, template/tagged-template
expressions, unary/binary/logical/conditional/sequence expressions, members,
optional chains/calls, arrays and object values/spreads/computed keys, ordinary
calls, constructors, dynamic imports and simple assignment/update targets.
Every recursive branch must finish before a table is returned. Scoped
functions/classes, destructuring targets, TS erasure, JSX, direct eval,
delete and context-dependent syntax are explicit unsupported outcomes.
Depth 64 and 4,096 visited nodes bound this retained-tree traversal separately
from parser admission. Full grammar and complete file binding remain unfinished.

`ResolutionTable` has private fields and no occurrence-list constructor. An
empty successful list follows actual complete supported-expression analysis.
Missing bindings, invalid spans, unsupported nodes or traversal limits return
one checked error, without publishing partial facts or losing original source.
The owned occurrence vector moves directly from the resolver into its table;
two reviewed analysis storage rows record the actual new allocation sites.

Seven actual retained-AST laws pass with pinned OXC and L0 dependencies: exact
ordered references, assignment roles, Unicode/escaped names, empty/missing
facts, unsupported-family rejection, chains/templates/new/imports and bounded
elision/depth traversal. Direct
rustc also compiles the whole L2 library in `no_std` mode. Narrow direct Clippy
checks pass with the workspace panic/indexing policy. Exact-head Actions,
allocation checks and all unchanged instruction ceilings remain required before
merge. The regenerated legacy-consumer inventory has a documented name-level
false positive in the prerequisite artifact check (`scopes.get` mistaken for a
legacy producer named `get`); it is not an L2 normal/build dependency.

This supplies an expression provider. Canonical script declarations, aliases,
scope-aware file binding, complete native L1 lowering, pass migration and
product adoption remain unfinished. No production route or budget changes;
#6838 remains open.

## Actual inputs and outputs

The input is the retained `JsExpr` and a `BindingLookup` for its enclosing
scope. Native expressions additionally retain the real checked authored/decode
coordinate capability. The lookup supplies already-established binding IDs;
it is not a file binder, context-name generator or declaration registry.
The output is one complete supported-family `ResolutionTable` retaining
that exact AST/source/coordinate capability, or a typed error without partial
facts. Ordered occurrences carry decoded byte spans and usage/shorthand/new
callee roles. No reparsing, serialized packet or framework policy is added.

The canonical artifact's current `ScopeFacts` describe template introduction
sites: a `ScopeTag`, names and authored/synthesized origins. They do not own
script declarations, approved global/context inputs, complete destructuring
bindings or normalized slot-region visibility. `BindingLookup` receives only
a name, so selecting arbitrary IDs or scope tags through it does not prove
those declarations are visible at a particular artifact expression node.

TODO: the actual file binder must retain declaration origins and lexical
visibility, derive stable IDs from one artifact owner, and produce both its
expression resolutions and downstream access facts from the same registry.
It must reject foreign owners and unsupported scopes. Native `v-for`/slot
lowering and the declaration-bearing script handoff are prerequisites where
their current facts are incomplete. Interning unknown names into caller-owned
context IDs would not implement this missing provider; ContextOnly remains an
explicit caller/test policy until genuine declarations and language admission
prove it. This registry is unfinished in the current slice.
