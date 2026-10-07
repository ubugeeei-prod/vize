These supplemental #7994 inputs preserve every original same-name fixture.
`Recursive.vue` resolves its own component from the authored file name. Its
`:id` binds that component's public `id` key to its own implicit prop value.
The intrinsic input has only a value role; Parent has an independent local.

Renaming the public prop to `field` must change its declaration, the recursive
directive to `:field="field"`, the intrinsic directive to `:id="field"`, and
Parent's directive to `:field="id"`. Parent's local declaration stays exact.
Inline, local alias, interface and withDefaults forms run with LF and CRLF,
from the public declaration, recursive argument and Parent argument. The
Parent local direction separately preserves the whole recursive component.

The same-file shadow carrier has its own `const id`. Public renaming changes
only the declaration and both component keys: recursive and Parent values
remain `id`, and the intrinsic directive stays shorthand. Renaming that local
changes its declaration and the two local values, preserving the public key
and the whole Parent. File identity or matching spelling cannot establish a
dual role.

The mandatory real stdio target compares every complete references and
WorkspaceEdit response, applies the actual returned edits, writes both files,
checks their complete disk bytes and requires empty diagnostics in both
updated versions. It collects every form/newline observation before comparing
the full result vector; one differing transaction does not discard the other
native observations. Source preparation is not runtime qualification.
