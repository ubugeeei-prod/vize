# Reactive props destructure rename

Original report: #7996. The report body is frozen verbatim; the Child and
Parent files retain the reported same-name destructure and component binding.
`lsp_reactive_props_rename_cli` exercises public declaration/parent-argument
and local declaration/template directions, with complete native references,
complete WorkspaceEdits, applied full files and post-edit diagnostics. LF and
CRLF, shorthand/default bindings and explicit aliases are separate sessions.
The existing same-name, mixed-role and external-edit refusal fixtures are
unchanged. Current-source Actions qualification is required before completion.

`supplemental/Default.vue.txt` and `supplemental/Aliased.vue.txt` are complete
authored derivatives, preserving the interpolation as a value read while
changing only the intended binding form. They are loaded directly, avoiding
script-pattern substitutions inside template interpolation braces.
