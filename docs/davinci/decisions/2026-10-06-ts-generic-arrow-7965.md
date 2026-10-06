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

## First published source failure

Published source `7690c0af` / Check37399499074 is historical RED. Required
security job112063268701 found the existing global13 npm advisories; its
separate owner is repairing them without a waiver. Tooling2 job112063392986
failed this new observer's empty-stderr assumption after the exact original
two-file command returned zero. Official artifact11385080658 is authenticated
at29,543B/SHA25644da4377/all member CRCs, retaining checkoutd2dbd283,
source-built vize0.434.0 binary hash7f586502, workspace Oxfmt0.63.0 and the
whole original command streams. The twelve rows and original-output comparison
were not reached; no whole-corpus success or executable-byte rehash is claimed.

The [paired correction](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6007553939)
changes only this observer's full normal stdout/stderr expectations, independently
derived from existing CLI summary/change branches and literal-file collection.
It retains both original outputs before process assertions. All seven Rust laws,
originals, twelve complete formatted expectations, formatter production,
historical pins/options and ceilings stay exact; there is no record mode,
stderr suppression or formatter change. Strict Rust source build already passed
on7690; actual Rust cases and fresh repaired whole7/12/source/protected104/full
Rust/signed merge/release remain required. No old result transfers to a new head.

## Current affected runtime and upstream dependency

[Current paired evidence](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6007730993)
qualifies source `4f013b88` only. Official artifact11384923040 at32,560B/API
ce0ce223/all17CRCs retains both complete originals and exact command, all12
cases/36 whole Oxfmt0.63 outputs/error vectors/60 whole case CLI observations.
Actual checkout3ccd0f50 has genuine143c+4f parents and the exact reviewed tree;
source-built vize0.434.0 receipt binds binarySHA a4d3f9f0. No executable bytes
were independently rehashed. Proof8e34432c verifies whole bytes, streams,
status/error/signal, unchanged check files and three-pass fixed points.
All7 original Rust laws actually pass once in the four API-hash/CRC-qualified
current JUnit archives (prooff7e7ffa8); strict builder and all4workers succeeded.

Required security job112066920225 still fails on the common13 npm advisories.
The separately owned #8080 must actually merge before a genuine incorporation
and fresh exact-head qualification; this branch remains Draft/auto-null/offqueue.
At that checkpoint canonical DOM/fan-in completion was pending. Old7690's terminal conclusion
is CANCELLED from ordinary superseding concurrency, retaining its real two
failures and seven successes; cancellation grants no passing aggregate. The
new runtime result gives no protected104/full historical/actual merge/release
credit. Current-data adoption, native/default/history and10x remain unfinished.

## One report recovery, audit still blocked

The [paired terminal record](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6007983274)
retains Check37400529655 attempt2 overall FAILURE. The one targeted recovery
actually passed Rust source report112073713803 and PR source report112073738982;
final test-report and the existing report-comment reran automatically. Every21
other executed context retains its original timestamps/result; all16 skipped
markers have no runner/steps. No successful substantive suite repeated.
The earlier cancelled report's network/maximum-job annotations stay historical.
Receipt57eab0d2 authenticates complete API records and execution boundaries.

Only inherited security-audit and final test-report remain failed. Source4f
canonical DOM succeeded, but the common13 advisory repair #8080 must actually
merge before genuine main incorporation and fresh exact source qualification.
There is no further report retry, waiver, queue/protected/merge/release credit.
These docs checkpoints remain local until that incorporation; the current
seven/twelve source-scoped evidence does not transfer to a future composition.

#8080 actually merged to signed main `48cb1d4` at 2026-10-06 05:04:12 UTC.
Replay only our five commits onto that genuine main. Intermediate `2d227db`
retains all 13 owned noncanonical objects, all 22,227 incoming nonowned entries
and every full author, date, message and reporter trailer. Removing only the
owned canonical appendix recovers all incoming bytes at 350 lines.
Preserve the `ee01e11` backup and the old head's 7/12 execution proof;
no runtime credit transfers to the replay. Fresh exact-head Actions and
complete affected results must precede admission. Full protected suites,
unchanged ceilings, signed merge and installed-public twelve cases remain
pending; any finite release admission hold remains honored.
Paired decision: [6009735472](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6009735472).

The exact e7dd source passed Check 37417134551 and all four required contexts.
Its fresh authenticated hosted packet qualified seven Rust laws, the original
two-file command, twelve whole CLI cases, thirty-six Oxfmt outputs and sixty
complete CLI attempts. These results remain source-scoped.

After Ready, the whole admitted #8078 → #8075 → #8073 → #8076 prefix at 6278986
introduced a `.gitattributes` EOF conflict. Auto-merge was not enabled and #8077
never entered the queue. Relocate only our existing literal generic-arrow block
beside stable formatter-history rules on actual main 48cb. The projected union
retains both complete literal controls, without importing unmerged rule or
production bytes into the owned source. Moving that block back mechanically
recovers the prior owned attribute file exactly; its semantics stay unchanged.

No producer, original bytes, expected output, timeout or budget changes. The
queued candidate is prospective, not actual main. [The paired decision](https://github.com/ubugeeei-prod/vize/issues/7965#issuecomment-6010079925)
requires fresh automatic source checks and full healthy-prefix composition
before admission. Protected merge and installed-public replay remain pending.

During private preparation, #8078 actually merged at 05:37:19Z to signed-valid
de40f26cb31b34d3b5e72ddb03f2c0d44b177622. Genuinely replay all seven owned
commits onto that literal actual main: thirteen noncanonical owned objects and
all full author/date/body/footer records remain exact. The complete incoming
canonical prefixes remain at 350 lines. This actual incorporation is distinct
from later prospective queued candidates; fresh source execution, protected
merge and public original-case replay are still pending.
