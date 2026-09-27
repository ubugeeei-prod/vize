# Static-class public autofix regression corpus

Issue: [#6920](https://github.com/ubugeeei-prod/vize/issues/6920), discovered
while preparing [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).

`cases.json` contains nine actual authored template/SFC inputs. The real Rust
integration test `crates/vize_patina/tests/static_class_fix.rs` executes every
input using the public Incremental preset and explicitly enabled builtin rule.
Expected UTF8 byte spans and complete applied source are authored in the corpus,
independently from the implementation. Six genuine offers must preserve all
other bytes and re-lint without a diagnostic. Existing static class, dynamic
expression and empty expression are actual no-fix controls.

Coverage includes short/long bindings, outer single/double quotes, template
literals, self-closing tags, adjacent attributes and Japanese/emoji/CRLF SFC
prefixes. Inputs live in JSON and add no accidental `.vue` walker members.
The default full Rust workspace suite executes this corpus integration test;
shared #6891 registration and native comparisons are still separate work.

`before-fix-history-cases.json` preserves the exact initial 13-case JSON pack,
including its original bad static-class case identity. Its SHA256 is the
`caseFileSha256` in the original history capture receipt. The unchanged
`lint_history__static-class-utf8-fix-control.snap` records the original offered
bad edit, malformed applied bytes and three parser errors. Neither archive is
an executable valid-output oracle or an eligible additional native case.

The current history runner replaces that one case identity with
`static-class-utf8-fix-corrected`, using its own full current-output snapshot.
It still executes 13 cases; the original bad bytes and receipt stay immutable.
No whole-history, unsupported adapter or native acceptance credit is inferred.
