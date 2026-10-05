# Bare template identifiers preserve typed LSP identity

Decision for [#7835](https://github.com/ubugeeei-prod/vize/issues/7835).

The ordinary template statement emitter searched the complete generated
`void (expression)` line for the authored expression. A bare identifier that
is a substring of `void` selected the keyword instead of the emitted binding.
Record the known expression offset plus the existing incomplete-expression
mapped bounds directly. Generated TypeScript bytes and pipeline stages stay
unchanged; the source mapping alone is corrected without an extra scan.

The corpus retains the complete original App.vue and strict Bundler config.
A generator law checks authored/generated ranges and reverse mapping. Fresh
real stdio `vize lsp` sessions assert complete typed hover ranges, references
with and without declarations, and complete ordered WorkspaceEdit objects
from declarations, bare interpolations and compound uses. Applying the actual
edits must match every resulting source byte. The seven broken names and all
18 reported unaffected controls run with LF/CRLF and an astral prefix;
parameter shadows and non-code text stay separate. Every unsaved rename and
restoration requires the complete empty versioned diagnostic publication.

Actions qualification, protected unchanged instruction ceilings, actual merge,
release and installed editor acceptance remain pending. This is a correction
of the current legacy product, not native-stage or complete fix-history credit;
#6883 remains open. This change claims no measured latency or 10x improvement.
