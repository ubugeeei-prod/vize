# Unary comment parentheses (#7929)

Retain the two complete TypeScript conditions reported in
[#7929](https://github.com/ubugeeei-prod/vize/issues/7929), including their
trailing own-line comments, without adding a second pair of parentheses.

The pinned OXC formatter independently adds comment-retaining unary grouping
and binary/logical operand grouping. Backport official OXC
[fix #25526](https://github.com/oxc-project/oxc/commit/7f350d166ff5b3d460c27c2eac7dd28123e7ba8c):
a printed-state-independent comment predicate lets unary grouping supply the
single pair and indentation. Other argument types retain their existing
parentheses rules. This also preserves leading/trailing block comments and
required nested operand groups.

Import the exact current formatter source and MIT license at OXC
`fc702c1fa9f0412d06ec6908b58cd395b826cf7f` in a separate source-only commit.
The following change integrates unpublished `vize_oxc_formatter`, retaining
its `oxc_formatter` library name, and changes four private formatter files.
Supporting formatter core, AST, allocator and parser revisions remain exact.
The existing Vize parser is the direct dependency. There is no output-text
rewrite, second parse or additional pipeline level. A revision upgrade was
rejected after a bounded parser check reported 62 AST API incompatibilities;
a broad OXC migration is separate work.

The complete `a.ts` (64 bytes) and `b.ts` (97 bytes), LF/CRLF variants and SFC
transport are committed in the existing differential formatter corpus.
Public Rust tests compare whole outputs through three passes, changed flags,
comment tokens, strings/regexp data and required inner parentheses. The actual
source-built CLI checks without writing, performs three writes, then checks
again, comparing complete bytes, exit status and streams. All prior corpus
cases and reference bytes remain intact.

Exact-head Actions, all existing formatter/history suites and unchanged
protected instruction ceilings must pass before actual merge. Installed
release verification remains separate until a release contains this change.

The initial source Check (37644756787, `6c0f3f3c`) exposed two retained
upstream `match_same_arms` expectations without their enabling lint. Restore
that exact lint in the vendored policy; preserve all implementation bodies
and fixtures, and require fresh successor Actions.
