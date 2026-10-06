# Preserve the declared TypeScript script extension

Issue: [#7965](https://github.com/ubugeeei-prod/vize/issues/7965).
The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6007450561)
records the same source publication and qualification boundaries.

The complete original issue, all four fenced inputs/command/reported output,
and verified reporter identity are retained with whole-byte hashes. The
reported Generic.vue and plain.ts are independently expected to remain
unchanged. The report names vize 0.432.0 and Node 26.8.1; those versions have
not been reproduced in this preparation.

The locked OXC revision fc702c1fa9f0412d06ec6908b58cd395b826cf7f defines
SourceType::ts() with extension=None. Its type-parameter printer forces a
single unconstrained generic-arrow comma unless extension is exactly Ts,
including unknown extensions. The existing SFC selector therefore parses the
right TypeScript grammar but prints explicit lang=ts with unknown-extension
policy. The formatter callback argument None is not an extension parameter.

Only explicit lang=ts now selects SourceType::from(FileExtension::Ts), retaining
with_module(true). The language, standard variant and module grammar are
unchanged. Explicit TSX/JSX, omitted/unknown languages and public callers that
supply unknown-extension SourceType values retain their previous policy.
Standalone CLI .ts files already carry actual path-derived metadata. This is
not a formatter dependency update, synthetic filename, post-print comma
rewrite, broader dialect fix, or extra parser/stage. Six stabilization passes,
fresh arenas, cached trimmed lengths, check-mode/Auto, sort settings and
parse/refusal behavior remain unchanged. One nonsemantic statement separator
keeps the original production owner at 350 lines.

All ten historical script function bodies and their original/current function
hashes are unchanged; only the complete current owner hash is refreshed.
Old #7868's unrelated unpublished dirty eighteen-file worktree remains exact
and is neither reset nor replayed into this branch. Both instruction-cap files
and every existing expected/captured formatter byte remain unchanged.

Seven full-result Rust laws cover original setup/ordinary scripts, removing
TS-only disambiguation commas, keeping mandatory TSX commas under all three
trailing-comma settings, LF/CRLF/Auto/check-mode, explicit public extension and
unknown-extension controls, TS angle assertions versus actual JSX admission,
and constraints/multiple parameters. Twelve authored whole CLI/Oxfmt vectors
retain exact source/expected bytes. The hosted test requires actual workspace
Oxfmt 0.63.0 and its lock importer/catalog owner; it records three complete
reference outputs/errors, the exact original two-file CLI command, and three
write outputs plus read-only initial/final checks per case. Source-built CLI
receipt/revision/binary hash and all raw streams are mandatory. There is no
expected-output regeneration, fallback binary, skipped source guard or native
credit. The JavaScript oracle and locked Rust formatter 0.60.0 are distinct
owners; successful execution of either is not implied by inspecting source.

Current preparation is private on actual signed main f0361cbca4e3cbd4c23bd68c99251a035b01a374.
No Rust build, real Oxfmt fixture execution, source Actions, native campaign,
PR/source push, queue admission or publication has run. Static formatting,
syntax/history/census checks are recorded separately in the preparation packet.
Fresh exact-head source/affected Actions, full protected Rust/current104 gates,
actual signed merge with literal verified reporter credit, issue closure and
released public qualification remain required. nativeHandled stays zero; this
legacy fix does not close formatter fix-history or establish the 10x target.

## Current-main source publication

The original private source `05f4f059` passed independent source-only review
`3873d193`; preparation receipt `0eba1848` preserves the original input and
static checks. A genuine rebase onto actual main `143c1d4a` produces
`ec281f6b` with all thirteen noncanonical owned blobs, original source author,
date, full message and verified reporter trailer exact. All incoming nonowned
tree entries and the complete incoming canonical text are retained; removing
only the owned formatter appendix recovers the original 350-line record.

The conventional Draft and automatic exact-head Actions are now authorized.
The earlier private/no-execution checkpoint remains historical. No local
Rust/Oxfmt/CLI execution or hosted result is inferred from source review.
Fresh whole seven-law/twelve-control and existing historical compatibility,
protected full Rust/unchanged104, actual signed merge and root-owned public
release remain required. The issue comment above is paired with this checkpoint;
`nativeHandled` remains zero and formatter native/default/history completion
is unfinished.
