# Template listener ownership and correctness hold

The whole implementation and all 70 authored controls from parent source
`a2c9f545bfe90ea44f70be78d38fc8b18a533a4f` are preserved on
`fix/patina-template-listeners-retained-ast` before the script-only split.
The 53 script controls belong to parent PR #9222; all 17 template controls and
their complete native/provider expectations belong to [Draft child #9251](https://github.com/ubugeeei-prod/vize/pull/9251). No fixture,
source packet, original report or expected diagnostic is erased.

The existing template walker is a preserved implementation under review, not
accepted production code. This child must stay Draft until it uses retained
expression owners, canonical authored ranges (including entity-encoded
attributes), authenticated loop/slot bindings including legacy slot-scope,
If conditions and guarded traversal. Synthetic expression/arrow reparsing is
not acceptable. Reuse the SliceA/#9245 ownership and binding primitives.

`template-ownership.json` lists every control, its owner and source hash, plus
complete packet hashes. `native.json` retains every original native expectation.
The parent may qualify only the explicit script projection; the child must run
the full template inventory and add the review's missing boundaries before it
can be marked ready. Issue #9217 remains open.
