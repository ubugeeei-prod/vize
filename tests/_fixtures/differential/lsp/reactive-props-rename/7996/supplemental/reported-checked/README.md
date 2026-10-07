# Original checked boolean rename report

`original-comment.md` preserves the complete
[#7996 supplemental report](https://github.com/ubugeeei-prod/vize/issues/7996#issuecomment-5987980068),
including its historical response and configuration. `Child.vue.txt` and
`Parent.vue.txt` are its exact LF code-fence bytes. The original tsconfig and
Vize JSON code fences are retained separately without substituting them for
the current standard native test project configuration.

The independently authored public repaired files rename the prop to `active`
and expand the destructure to `{ active: checked }`, preserving the Child's
local ternary and Parent's literal `true`. The local repaired Child instead
uses `{ checked: active }` and changes the ternary, preserving the complete
Parent and public type key. The full reference/edit vectors distinguish these
identities without filtering any returned entry.

`lsp_checked_prop_rename_cli.rs` requires eight real stdio sessions: the public
type declaration, Parent argument, local binding and local template value,
each with LF and CRLF. The displayed historical edit coordinates are
human-readable; requests and complete expected ranges use the actual frozen
source's native zero-based UTF-16 positions.

The existing Vue fixture pins the workspace native TypeScript runtime and
frozen Vue dependency, uses a strict bundler project and enables cross-file
type checking. It has no runtime skip path. Each session compares the complete
references and WorkspaceEdit, applies actual returned edits, writes both files
before version-2 changes, checks actual disk bytes and requires empty complete
post-edit diagnostics for both files. Protected execution requires
`VIZE_TEST_REQUIRE_TSGO=1` with `VIZE_TEST_DISABLE_TSGO` unset. Preparation alone
supplies no native, protected-merge, issue-completion or release credit.

After preserving the entire actual version-2 observation, each same-process
session independently installs the authored repaired project, reads both disk
files and captures complete version-3 diagnostics. Those repair arrays must be
empty even when the actual transaction was wrong; they never replace or weaken
the actual references, edits, applied files or version-2 assertions.
