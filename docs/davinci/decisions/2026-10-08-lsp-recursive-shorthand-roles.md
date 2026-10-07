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

This first source change prepares the fixtures and expected contracts only.
Production is unchanged. There is no exact-current native binary locally;
hosted source Actions must establish actual current responses before a
production correction receives qualification. Historical binaries and
predecessor green runs provide no current execution credit.

TODO: preserve actual initial native responses; review and implement the
smallest producer-authenticated role correction; qualify the unchanged whole
40-session contract and original controls on its fresh exact source; attend
protected execution, signed actual merge and the next release. Performance
budgets, canonical source length and all gates remain unchanged.
