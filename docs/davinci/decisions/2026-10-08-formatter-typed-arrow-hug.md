# Typed last-argument arrows (#7868)

A type-reference return annotation on a last-argument arrow must not prevent
its object/array/block body from using the same call grouping as an untyped
arrow. Retain the complete 191-byte TypeScript reproduction from
[#7868](https://github.com/ubugeeei-prod/vize/issues/7868), including the Row
type and every template literal/property token.

Backport official OXC [fix #25044](https://github.com/oxc-project/oxc/commit/ca1ba71595d175da6b3b44b6100e09bfaa66f8e1):
remove only the obsolete 44-line return-type gate in the existing private
formatter's call argument grouping. Body-based admission remains unchanged.
The source depends on the separately licensed pinned formatter import in
[#8193](https://github.com/ubugeeei-prod/vize/pull/8193); keep a separate native
Stack child and do not expand that parent's scope. Supporting OXC/parser/AST
versions remain pinned, with no second parse or output-text rewrite.

The expected whole output follows the reporter's hugged layout and was
independently checked with Prettier 3.9.6 at Vize's default width 100. A bounded
pre-fix private formatter execution reproduces the original failure; the
44-line backport then matches the unchanged expected output through three
passes. Public Rust laws retain LF/CRLF, ordinary/setup SFC carriers, changed
flags, untyped behavior, generic/union/array return types, body comments,
scalar/empty bodies and TSX. The actual CLI checks before writing, performs
three writes, then checks again, with complete bytes/status/streams. Register
one additive SFC corpus case and preserve every prior source/reference byte.

Fresh exact-head source/full corpus and protected Actions must pass before
root-owned Stack-prefix queue admission and actual merge. Public installed
release verification remains separate until a release includes the fix.
