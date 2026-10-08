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

The initial `10cbbf6d` source Check (37651827449, job 112896841289)
rejected a missing generated preferred-L0 row for the new compact-string
public test helper. Regenerate that exact formatter inventory row; preserve
all production and original/reference bytes and require fresh Actions.

PR [#8199](https://github.com/ubugeeei-prod/vize/pull/8199)
actually merged at 2026-10-07 17:36:27Z as signed, GitHub-verified
`9d87650eee89b7f44b1a1305cbb092d4ece59f10`. Its exact original source head
`745f6b82f29ff53d19745cef726083b2978e4d60` passed PR Check
[37652663471](https://github.com/ubugeeei-prod/vize/actions/runs/37652663471).
The exact protected merge-group Check
[37656845042](https://github.com/ubugeeei-prod/vize/actions/runs/37656845042)
passed on that actual merge SHA: 25 successful jobs, 16 planned skips and no
failed or pending jobs. Check, Musea accessibility/capture, Nuxt 3 build, Nuxt scoped-style build
and n8n adoption queue workflows completed successfully. The actual child parent is
`102535bc0cc60e2aa1df1efde9334d5abb2122a9`, preserving the parent-to-child
delivery relation.

Retain the separate full source Check failure
[37652706945](https://github.com/ubugeeei-prod/vize/actions/runs/37652706945):
its headless Vim scenario exits 1 and its Element Plus slot oracle rejects
whole editor-revision results. They are separate failures; PR/protected
success does not turn this full source run green. Preserve both complete
raw job logs and hashes, 112899906279
`66aec2517b6c59fce96d785c2ffdfa0139eab65566deb4b552952266751b9c43`, and 112899906291
`106b2bb0dff2c307571aa4aa961279f299fefe7d36e9a45d1e7e29a16befdfb5`.
All earlier source failures, original fixture bytes, independent expectations
and regression controls remain retained. Public installed acceptance remains
pending: audit fix ancestry against the actual next cut/source receipt and
replay complete originals through that registry-installed package. The retired
unpublished 0.436 cut and source-only observations do not close this issue.
