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

The next source Check (37645589975, `d0d67f08`) passed strict compilation
but its enrollment guard treated the new vendor as first-party. Extend only
the existing exact parser exclusion to the exact formatter path; all 39
first-party members remain enrolled and `PENDING` stays empty.

The `ead56771` PR Check (37646533326, worker 112883018216) passed both
original-input public laws and the complete CLI law but rejected the added
nested logical control's layout. Prettier 3.9.6 independently confirms the
unchanged whole expected output. Retain a group around the flattened operand
body, without a second indent or parentheses pair, to isolate it from the
outer comment's forced break. All original fixtures and expectations remain
unchanged; add inner-line-comment and long-chain controls, and require fresh
source/protected Actions.

PR [#8193](https://github.com/ubugeeei-prod/vize/pull/8193)
actually merged at 2026-10-07 17:36:27Z as signed, GitHub-verified
`102535bc0cc60e2aa1df1efde9334d5abb2122a9`. Its exact original source head
`e9eea64a35a7dd6fa729e1e0ab8b91a3e5103509` passed PR Check
[37649396334](https://github.com/ubugeeei-prod/vize/actions/runs/37649396334).
The exact protected merge-group Check
[37656842645](https://github.com/ubugeeei-prod/vize/actions/runs/37656842645)
passed on that actual merge SHA: 25 successful jobs, 16 planned skips and no
failed or pending jobs. Check, Musea accessibility/capture, Nuxt 3 build, Nuxt scoped-style build
and n8n adoption queue workflows completed successfully. The actual child parent is
`102535bc0cc60e2aa1df1efde9334d5abb2122a9`, preserving the parent-to-child
delivery relation.

Retain the separate full source Check failure
[37649406473](https://github.com/ubugeeei-prod/vize/actions/runs/37649406473):
its headless Vim scenario exits 1 and its Element Plus slot oracle rejects
whole editor-revision results. They are separate failures; PR/protected
success does not turn this full source run green. Preserve both complete
raw job logs and hashes, 112888498650
`ea5a731a06c3f3cce8d5b4ede4f90dd5abda158cb797fb983a8e3b2d7b255277`, and 112888498718
`b3f933d91afed04e225c7830e14f4203e065cbe805e679019dedfc65b03588f9`.
All earlier source failures, original fixture bytes, independent expectations
and regression controls remain retained. Public installed acceptance remains
pending: audit fix ancestry against the actual next cut/source receipt and
replay complete originals through that registry-installed package. The retired
unpublished 0.436 cut and source-only observations do not close this issue.
