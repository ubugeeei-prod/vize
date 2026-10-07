# Recursive component shorthand rename roles

Issue: [#7994](https://github.com/ubugeeei-prod/vize/issues/7994).
This supplements the [same-name rename decision](./2026-10-07-lsp-same-name-rename.md).

A recursive component can pass its own implicit prop through a same-name
shorthand. In `Recursive.vue`, `<Recursive :id />` can select both the public
`id` key and its implicit `id` value when the public prop is renamed.
The complete expected transaction is `:field="field"`, alongside the public
declaration and intrinsic `:id="field"`. Parent's independent local is
preserved by `:field="id"`.

Same-file ownership does not establish that dual role. A setup `const id`
shadows the implicit prop value in the recursive component. Public renaming
must then produce `:field="id"`, preserve the local declaration and retain the
intrinsic shorthand. Renaming that local must preserve the public key and the
whole Parent. Native expression provenance, independently of the native key
endpoint, is required before changing both roles at one authored token.
Contextual object keys are not value expressions merely because they lack a
component-navigation link. No source spelling sweep or URI-only inference is
admitted.

The new supplemental corpus adds four declaration forms with LF and CRLF,
three public query origins and the independent Parent local direction, plus
same-file shadow public/local controls: 40 mandatory real stdio sessions.
Every wrapper retains its complete observation vector before comparison:
initial diagnostics, full references and WorkspaceEdit, actual edits applied
to both complete files, disk bytes and both versioned post-edit diagnostic
notifications. Original same-name, alias and mixed-role fixtures and all
existing mapping/refusal/native scope controls remain unchanged.

The test-first source `b105a1f0f72931cafd6d0df005b331ba6ff57d92`
reproduced the defect on Check `37644258658`: all 24 public-origin sessions
returned complete references but expanded the recursive occurrence to
`:field="id"`, leaving a native TS2339 diagnostic after applying the actual
edits. The 16 Parent-local and same-file shadow sessions passed. Source
workers set `VIZE_TEST_DISABLE_TSGO=1` for opt-out suites; these stdio fixtures
have no skip branch and require the discovered native runtime in `corsaPath`.
This is actual source reproduction, not protected `REQUIRE_TSGO=1` credit.

The correction retains the value role of scope-checked native edits before
authored projection and deduplication. Only edits at definition-verified
property arguments are considered. The existing generated-document AST must
positively identify an expression reference or member access; contextual and
type keys remain key-only. The parser result is reused once per generated
document for the complete rename transaction. Unresolved provenance refuses
the transaction. The existing shorthand rewrite changes both roles only
when native edits selected both, preserving modifiers, annotations, versions,
source geometry, all mapping/refusal guards and final coherent-edit checks.
No new native query or source-spelling selection is introduced. Fresh source
Actions and protected performance gates must qualify the correction.

TODO: qualify the unchanged whole 40-session contract and original controls
on the correction's fresh exact source; attend protected execution, signed
actual merge and the next release. Historical binaries and predecessor green
runs provide no current execution credit. Performance budgets, canonical
source length and all gates remain unchanged.
