# Plain script ownership and annotated refs

This decision fixes public [#7934](https://github.com/ubugeeei-prod/vize/issues/7934),
reported by `ubugeeei` (GitHub account ID `71201308`), paired with
[the issue decision](https://github.com/ubugeeei-prod/vize/issues/7934#issuecomment-6017808436). The full original issue,
340-byte `src/config.ts`, 110-byte configuration and exact `vize lint src`
command are retained in
`tests/_fixtures/differential/linter/plain-script-ownership-7934/` with byte
counts and SHA-256 custody. The original module has top-level await, three
runtime exports and a `Ref<string | null>` binding initialized by `ref(null)`.
Both configured rules must produce complete empty diagnostic vectors there.

On actual signed main `97d5c26a153944e66439a3108746c19d7bf62220`, the export
rule inferred setup ownership from top-level await or macro-like identifiers.
The existing SFC dispatch already passes descriptor-derived `SfcScriptContext`;
use its `is_sfc && is_script_setup` identity instead. Standalone TS/JS and normal
SFC scripts allow exports, while genuine setup blocks retain runtime-export
diagnostics even without await or compiler macros. Existing type-only export
exceptions, messages, help, spans and default severity stay intact.

The typed-ref rule now recognizes an authored type annotation on an identifier
binding whose direct initializer is the ref call, including parentheses. The
visitor retains that exact call span only while visiting its declarator and
restores the previous state afterward. It does not exempt calls nested in
arrays, wrappers or functions, destructured bindings, assertions without a
binding annotation, or the next unannotated binding. Existing Vue import alias
and explicit type-argument handling remains unchanged. This syntax correction
does not add a native query, stage, dependency or instruction-budget change.

The independently authored differential corpus contains 33 complete service
and CLI JSON vectors: literal original/LF/CRLF/Unicode inputs; TS, JS, MTS, MJS,
CTS and CJS modules; normal/setup SFC ownership; annotation spellings and import
aliases; and runtime-export/untyped-ref negative controls. Diagnostic arrays,
counts, severity, ranges, labels, help and fix values are checked whole. The CLI
law executes the original command and configuration before replaying every
whole JSON vector, retains all process status/stdout/stderr bytes before
assertions, and verifies that source/config bytes are unchanged. Original unit
inputs and snapshot expectations are conserved; tests that represent setup
now supply the existing explicit SFC context instead of relying on a heuristic.

Qualification remains pending until exact-head hosted Actions and the protected
queue complete. Local source formatting and the existing Croquis-consumer and
consumer-migration census generators are checked before push. These two rules
are syntax rules; this change claims zero native handled cases, no native
migration or separate LSP provider acceptance. Actual merge and subsequent
installed/release verification belong to the existing delivery lane; historical
green runs or source inspection do not substitute for current execution.

The source commit carries exactly one terminal reporter Co-author trailer.
Keep the PR small and conventional, retain incoming canonical decision clauses,
and enable independent squash auto-merge only after its current checks pass.
If an actual candidate fails, remove that candidate from the queue and correct
the same PR before fresh qualification. The root agent remains the sole release
publisher for the next frequent release.

First head `7003bcbd420c4a89b66be58e0baf8069dc7c70f7` passed affected Clippy,
compiled test archive, native L1/Program/navigation, doctests and all four Rust
workers. Tooling shard 3 alone rejected a new path-attributed test module under
the unchanged ordinary-discovery law. Preserve the complete official failure
log (758,943 bytes; SHA-256
`86566c4f1f7e266818ffe3fadd59e9613c5af2e71fc76514d02f2ca23d6867f7`).
The [paired correction](https://github.com/ubugeeei-prod/vize/issues/7934#issuecomment-6018063689)
removes only that unnecessary attribute: ordinary `mod ownership_tests;` loads
the same file. Production logic, all test bodies, original inputs and 33 whole
vectors remain exact first-head bytes; local execution of the original two
layout laws passes. No gate/oracle waiver is added, and fresh successor source
and protected qualification remain required before actual delivery.
