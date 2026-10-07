# Original three-component kebab rename report

`original-comment.md` preserves the complete original
[#7994 supplemental report](https://github.com/ubugeeei-prod/vize/issues/7994#issuecomment-5988117648),
including its historical failing responses. `Panel.vue.txt`, `Wrapper.vue.txt`
and `App.vue.txt` are the exact LF code-fence bytes from that report.

The independent repaired files are authored before native requests. The local
Wrapper cursor remains at native position `3:10`; the App Panel cursor remains
at `6:11`. References and edits use complete native UTF-16 source ranges,
including Panel's `v-if` and the entire shorthand directive for expansions.
Duplicate, zero-length, truncated or unrelated returned entries remain in the
whole response and fail the exact oracle.

`lsp_kebab_prop_rename_cli.rs` requires eight real stdio sessions: both reported
origins and independent App Wrapper public-prop and Wrapper heading controls,
each with LF and CRLF. A local rename preserves the public key; a public rename
preserves the independent local value. All three complete files, the heading
sites, the other component's same-spelled prop and the literal boolean values
are part of the expected transaction.

The shared Vue project fixture requires the workspace native TypeScript runtime
and frozen Vue dependency, enables cross-file type checking and has no runtime
skip path. Each session compares the complete references and WorkspaceEdit,
applies the actual returned edits, writes every file before version-2 changes,
checks the actual disk bytes and requires empty complete post-edit diagnostics
for all three files. Protected execution requires `VIZE_TEST_REQUIRE_TSGO=1`
with `VIZE_TEST_DISABLE_TSGO` unset. Preparation alone supplies no native,
protected-merge, issue-completion or release credit.

After preserving the entire actual version-2 observation, each same-process
session independently installs its authored repaired files, reads every disk
file and captures complete version-3 diagnostics. Those repair arrays must be
empty even when the actual transaction was wrong; they never replace or weaken
the actual references, edits, applied files or version-2 assertions.
