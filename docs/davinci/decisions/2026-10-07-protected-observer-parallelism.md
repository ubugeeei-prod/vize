# Protected observer parallelism: local proposal

Status: reviewed implementation authorized for an independent conventional PR.
Root owns protected queue admission and actual merge/release decisions.
Actual throughput improvement remains **unmeasured** until exact-head Actions.
The delivery policy in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6037135723)
continues to govern finite cohort admission and verified release completion.

## Evidence and decision

The successful protected [Check 37612234749](https://github.com/ubugeeei-prod/vize/actions/runs/37612234749)
on `538370a5db8f4443538145c8ae891e5e84008328` took 20m26s from run creation
to final update. Its canonical job took 19m22s: fixture hydration 75s,
DOM 52s, SSR plus Pug 401s, and production reach 595s. These are historical
timestamps from actual jobs and raw logs, not measurements of this proposal.
The Rust producer took 12m41s, including 483s of feature differential work after
its completed 4,236,383,020-byte archive upload. Workers waited 507.703s after
archive completion for that dependency. A timestamp replay with only canonical
parallelism and unchanged Rust gives about 17m12s; it proves no new runtime.

Schedule DOM, SSR plus Pug, and production reach as three independent required
workers. They retain the complete original commands, fixture exclusions, runtime
setup, features, harnesses, byte comparisons, native reach controls, and budgets.
One finalizer retains the existing `s2 dom corpus` job name and legacy evidence
artifact. It succeeds only after all three workers succeed and their actual
GitHub executions and artifacts pass custody checks. No compiler or level gains
a pipeline stage or serialization; only CI job dependencies change.

Move the original merge-group Rust differential composite and fixture coverage
step into a sibling job alongside archive workers. The sibling depends on the
completed archive producer, verifies its unchanged HEAD/tree/archive digest and
runtime receipt, then restores the exact archive paths before running the same
recipe. The Rust report requires producer, every shard, and differential sibling.
An empty, skipped, failed, cancelled, or pending required tier fails closed.
Ordinary PR producer/shard behavior and worker selection remain unchanged.

## Complete fixture ownership and hydration

Cache only submodule Git object directories, keyed by committed `.gitmodules`
bytes and the entire ordered committed gitlink vector. PR and merge-group events
restore only; trusted main events may save through the existing cache trust
policy. Every worker checks candidate HEAD/tree, index gitlinks, committed URLs,
each fixture HEAD, and tracked-content cleanliness. Expected Vue paths and Git
blob identities come from every pinned commit's complete raw Git tree graph,
authenticated against the committed gitlinks. Each proof includes the raw commit,
all reachable trees, and symlink blobs; object hashes, full traversal, exact owners,
and unused or missing objects are checked independently of physical enumeration.
The physical manifest must match every expected path and blob exactly and also
records whole-byte SHA-256 and byte length. Enumeration/read errors, omissions,
replacements, untracked Vue additions, and changed bytes fail. The same proof and
manifest are required after execution and across all three workers. Cached test
conclusions receive no credit.

The parent index and tracked worktree must also match the candidate HEAD before
any identity is emitted. A quiet diff against HEAD rejects staged and unstaged
source changes; fixture gitlinks keep their independent complete checks.
Temporary Git controls retain unchanged HEAD and whole fixture bytes while
mutating parent compiler source, then require refusal, restored-source acceptance,
untracked generated-output acceptance, and complete committed fixture growth.
This closes receipt custody only; it grants no hosted or throughput qualification.

The historical 147-gitlink/42,998-file baseline is a parser control, not a fixed
limit on legitimate committed fixture additions. Expected counts are derived from
the complete pinned graph. Symlink files and directory aliases retain the original
walker's dereferenced contents and paths, including cross-fixture targets owned by
the complete committed graph. Path components resolve in physical order before
`..`; cycles, absolute or escaping targets, and foreign physical paths fail.
Original `node_modules` and `_git-worktrees` exclusions apply before resolution.
Nested unselected gitlinks remain empty directories, matching nonrecursive hydrate.

Real temporary Git controls cover committed file and new-gitlink additions,
omission of valid inputs without any known-invalid fixtures, replacements, raw
object forgeries, and symlink ownership. A read-only check of n8n at committed
`e882e8a483f433facb47bab9b407d0ec00a81172` derives all 1,369 Vue paths and confirms
their physical blob identities exactly. The old 147-fixture object stores were
unavailable at the supplied local paths; they were neither hydrated nor broadly
searched. Full 148-fixture/44,367-file pipeline acceptance is unexecuted. The
existing n8n fixture parent owns legitimate legacy producer/finalizer updates;
this proposal preserves those older source files and adds no fixed-count veto.

[Matrix 37617991549](https://github.com/ubugeeei-prod/vize/actions/runs/37617991549)
on `8a9110229414769be71595d7888c6cbd8f3add88` observed 37,487 Vue files after a
bulk-shallow failure and successful serial fallback. Its 147 selected gitlinks
and HEAD status bytes matched the earlier complete 42,998-file Matrix exactly.
The missing 5,511 files removed seven known-invalid inputs. The original finalizer
correctly refused nine skips against the unchanged required sixteen; source
compiler/parser/corpus files were identical. HEAD-only inventory cannot establish
complete checkout. The precise historical filesystem loss cause is unavailable
because the original helper discarded underlying Git stderr.
The committed graph also refuses this 5,511-file loss if the omitted inputs are
ordinary valid Vue files: exact coverage does not depend on the skip allowlist.

Keep the original hydration helper and fallback controls byte identical. After it
returns success, force checkout all selected committed submodules before taking
the full-byte snapshot. Retain raw stdout/stderr, exact command arguments, source
identity, and both checkout and status outcomes in the worker artifact. The new
operation cannot recover discarded stderr inside the unchanged original helper;
its top-level warnings/stdout/stderr are retained through fail-closed pipefail
logging, including when it fails before the forced stage. Failure remains a
gate refusal with diagnostic evidence. A real temporary Git/submodule fixture
proves that a missing authored file with unchanged HEAD is restored byte exactly;
failure and replaced-diagnostic controls receive no credit.

## Execution and artifact custody

The finalizer reads official job and artifact APIs for the same repository,
caller, source SHA, run, and attempt. Required commands, snapshot, receipt, and
upload steps must finish successfully in order within their execution windows.
An older green job cannot hide a latest failure or incomplete job. A failed-only
retry may carry an earlier successful execution only when the complete official
execution signature is identical, retaining its original attempt-bound artifact.
All three artifact bodies must agree on candidate/tree, complete committed
gitlinks, metadata, authenticated tree proof, whole file manifest, original sixteen skip reasons, full
canonical counters, and zero divergence/refusal requirements. The finalizer
reselects custody after downloading the exact artifact IDs with digest refusal.
For pull requests, the verified event PR head is the API provider identity while
`GITHUB_SHA` and its tree remain the candidate merge checkout identity. Both are
retained and checked separately; an API head is never credited as candidate bytes.
The verified event and official run also retain separate base and head repository
IDs, allowing a fork PR without crediting foreign base or artifact metadata.

## Review and publication TODOs

- Local focused tests, syntax/format/correctness lint, capped-file inventory, and
  frozen-source preservation are review evidence; they do not prove full corpus
  execution, Actions compatibility, or throughput.
- Root reviewed the concrete implementation and authorized publication after the
  tracked-source custody guard and its negative controls. Preserve the genuine
  current-main source union and all original fixture bytes.
- Pair the issue comment below with the shared decision record's paragraph 326.
  Only that paragraph gains an additive reference; the other 349 rows remain exact.
- Attend fresh exact-head source and full protected merge-group Actions. Compare
  actual critical-path timestamps, complete file/counter receipts, every Rust
  result, and fresh merge identity before claiming a throughput improvement.

## Paired #6830 issue-comment

Protected CI can schedule the unchanged DOM, SSR/Pug, and production-reach
observers independently, then require one custody-checking finalizer under the
existing status name. All three must cover the entire committed submodule vector
and every Vue path/blob derived from complete authenticated pinned Git trees,
with exact whole-byte manifests before and after execution. Legitimate committed
fixture additions change the derived inventory; omissions and replacements fail.
The historical 147/42,998 baseline, original sixteen known-invalid inputs, and
all refusal/byte/budget gates remain required controls. Git-object
caches are identity-keyed and PR/merge-group restore-only. A forced checkout with
retained diagnostics closes HEAD-only incomplete-hydration credit. The original
Rust feature recipe and fixture coverage run as a required sibling after the
verified archive producer, alongside all required archive shards. No compiler
stage, serialization, budget increase, test waiver, or release acceptance changes.
Parent tracked source and index must match candidate HEAD before any receipt;
staged and unstaged drift refuse even with unchanged fixture bytes and HEAD.
Historical protected Check took 20m26s; this proposal has no Actions runtime
measurement yet. Fresh exact-head source, full protected execution and actual
merge proof remain required; root owns queue admission.

## First source Actions and bounded repairs

Check 37644321941 at c0ee refused all three canonical workers before their
observer commands: the forced upstream checkout preserved authored CRLF JSON
blobs, but upstream text attributes made Git's filtered diff report changes.
Keep pinned HEAD and clean-index refusal. For Git-reported worktree changes,
compare the original raw blob and executable/symlink mode directly with the clean
HEAD index, without text filters. The independent complete Vue path/blob manifest
still refuses omissions, replacements, untracked additions and normalized bytes.
A real temporary repository proves the original attribute mismatch and refuses
staged, unstaged, missing, executable-mode and symlink changes.

Tooling also refused a cloned temporary fixture commit without an author;
configure only those test invocations and run them without global Git identity.
Await every original Node test registration to satisfy the existing type-aware
Promise rule. Zizmor refused the rotating synthetic stable action commit in the
new sibling. Pin both producer and sibling to official master ancestor d103106;
its execution steps are byte-identical to 6bed076, differing only in the unused
stable default. Every invocation retains explicit Rust 1.98.0 and all features.
The failed run's strict finalizer remained red. No corpus success, queue delivery,
throughput or full-feature sibling execution is credited to that source head;
fresh complete source and genuine protected qualification are still required.
