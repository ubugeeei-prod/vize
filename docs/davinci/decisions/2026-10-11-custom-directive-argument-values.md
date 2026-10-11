# Custom directive argument values (#6879)

The frozen Element Plus matrix retained three TS2464 false positives at
`20298a21ae54a5417955373b80130ae7c548a6bb`: Cascader's `contentRef` and
Select/Select V2's `popperRef` are DOM element arguments to ClickOutside.
Canon projected these custom directive arguments as object computed keys,
which imposed an unrelated JavaScript `PropertyKey` restriction. Vue's real
`DirectiveBinding<Value, Modifiers, Arg>` defaults `Arg` to `any`.

Classify dynamic custom arguments from `DirectiveNode.name` and the parser's
exact argument span in the existing directive-table AST walk. Keep these
expressions in the existing mapped `void` expression path, preserving unknown
names and member errors. Keep every original directive hook/value check and
the computed-key path for built-in directives. No prefix heuristic, diagnostic
filter, new pipeline stage, budget adjustment, or golden change is introduced.

The minimized complete native corpus retains both upstream directive
spellings and genuine Vue bindings/refs. It requires the whole vector for
legal arguments, unknown arguments, invalid members, wrong handler values,
and each repair. The existing spread and dynamic-name native controls remain
required. Full original source/projection/diagnostic packets are retained in
the existing `native-source-recipe*` immutable artifact namespace.

The hosted baseline uses the fixed actual main merge `006c564`: the three
modified preexisting source files are retained, temporarily restored to that
commit's exact blobs, and the new argument-metadata module is made absent.
The complete compiled-source manifest verifies all other production/Cargo/lock
bytes; this is labeled `006c564 production + candidate neutral test driver`,
not an exact whole baseline tree. Rebuild with the same native corpus and
dependency cache. Require the entire original three-row TS2464 vector, then restore the
candidate files, require the exact original tracked HEAD tree/source hash,
and run every clean/negative/repair vector. An exit trap
also restores the candidate on failure. Neither another target/builder nor
a generic tolerated baseline failure is introduced; both source identities,
whole files and runtime packets are retained. Run this before the original
native suite so the candidate's Canon build is reused by subsequent controls.

Baseline reproduction, exact-head Actions, full protected qualification,
actual merge, and release acceptance remain pending. The frozen matrix's
three original controls and full 224-case coverage/budgets remain unchanged;
a green minimized corpus does not grant full-project or release acceptance.
