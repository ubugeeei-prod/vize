# Mixed public prop and same-name binding roles

Supplemental coverage for #7994 and the local type ownership in #8011. The
original report and its Child.vue/Parent.vue reproduction remain unchanged.
Each Child variant is installed as Child.vue beside the shared Parent.vue.

Renaming the public `id` prop to `field`, from either its Child declaration or
the Parent shorthand argument, has exactly three references and three edits:
the Child property key, the Child DOM shorthand value, and the Parent public
shorthand argument. The Child DOM `id` attribute and Parent local `id` variable
retain their names. The repaired directives are `<input :id="field" />` and
`<Child :field="id" />`.

Renaming the Parent local `id` variable to `field` has exactly two references
and two edits in Parent.vue. Its shorthand becomes `<Child :id="field" />`;
Child.vue remains unchanged. The withDefaults variant defaults independent
`tone` so the `id` reference and edit cardinality remains identical.

`lsp_mixed_prop_rename_cli.rs` requires the production stdio service, the
workspace native TypeScript runtime, and frozen real Vue declarations. All
four Child forms and all three query origins run with LF and CRLF. The test
compares complete native responses, applies their actual edits, writes both
files before versioned changes, and requires empty diagnostics for both files.
