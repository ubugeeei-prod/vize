# Unopened component prop consumer regression

The complete Child/Parent fenced sources from #7824, including their file-label
comments, are preserved and pinned in `case.json`. The typed test config keeps
the reported `src/**/*.vue` ownership and uses strict Bundler resolution.

`lsp_unopened_component_props_cli.rs` initializes fresh real `vize lsp` processes
with every file on disk before initialize, opens only Child for the original
reproduction and compares complete references in both declaration modes and
complete ordered rename WorkspaceEdits from its declaration and interpolation.
Opening Parent is a separate control that queries the parent attribute as well.
Every actual edit is applied and compared against the whole expected source.

The extended complete-source cases cover LF/CRLF, astral UTF-16 prefixes,
unopened aliased consumers with static and bound props, a distinct Other.text,
a same-name parameter and non-code text, a tsconfig-excluded importer, and an
unsaved Parent buffer whose content must override disk. Full versioned initial
diagnostic publications stay empty; foreign and excluded file bytes remain
unchanged. Expected objects are independently authored from declaration and
occurrence identity. This current legacy regression grants no native or whole
historical LSP admission; #6883 stays open.
