# Gate guest contracts before protected merge (#6830)

Main `1b9fce90998ee522e40b8a10b1da3c1334a2690b` failed Davinci Contracts
run `37122730800`, job `111201891095`. The three typed-expression guest
laws failed while building the component with `--locked`, before their
typed WIT round trips could execute. The original failure remains preserved.

The root workspace version is `0.430.1`, but the standalone typed-expression
guest lock still recorded the in-tree `vize_guest` as `0.430.0`. The release
preparation already refreshes the expression guest, output guest and Volt
example locks; its explicit list omitted the typed-expression guest. Updating
the root workspace lock cannot refresh a separate `[workspace]` lock.

The repair in [#7590](https://github.com/ubugeeei-prod/vize/pull/7590) adds
that manifest to the existing offline, package-targeted release refresh and
regenerates its lock with the same command. The only
lock delta is the path SDK version `0.430.0` to `0.430.1`. Registry versions,
checksums, dependencies, guest source, `--locked` and law expectations stay
exact. Historical SDK/WIT compatibility workspaces are preserved.

That repair actually merged as `e56082b30a0b2255fc98228cb5b1e1bff985144f`.
The same #7586 now keeps the complementary regressions and queue enforcement;
its material duplicate lock and updater changes are removed. The earlier
`7f87` failed inventory and successful runtime evidence, plus the corrected
`c52` configured/runtime successes, stay preserved as distinct source epochs.

Two meaningful source laws discover every standalone manifest that points
to the actual in-tree SDK, require release refresh coverage for that complete
set, and compare each SDK lock version with the actual workspace version.
They fail on the original omission/stale lock and pass after the repair.
Ordinary offline locked Cargo metadata validates all six nested guest
workspaces plus Volt. Only the original typed guest fails before the change;
all seven graphs resolve afterward without a build or network access.

The first PR campaign's new guest laws pass, but the audited release inventory
still expects 28 files. Its exact inventory now records 29 and includes this
new regression in the broad, unscoped set. The 26 previously audited scopes
stay exact; no selector is weakened or extended without an input audit.

The original TS-48 workflow runs only on relevant pushes and explicit dispatch.
The protected `Check` report therefore misses a real guest failure. Reuse the
same workflow in one parallel `merge_group` job inside the existing mandatory
PR source workflow. Its unconditional `source-report` requires the WIT result;
the existing required `test-report` then requires that source workflow's result.
Keep the four required contexts and rulesets
unchanged. No direct Contracts queue trigger is added, preventing duplicate
guest builds; ordinary push/manual Contracts remain intact.

The explicit source-report mode requires this new row. It rejects missing, failed,
cancelled or skipped queue results and unknown event modes. Only this row may
skip on supported nonqueue events (PR, push, schedule and dispatch), where its
job is intentionally absent; all five prior source dependencies remain strict.
The top report's original eight dependencies and command, plus the shared
report CLI's default behavior, stay exact. All workflows retain read-only
permissions and distinct concurrency prefixes, and PRs gain no Wasmtime build.
The report's existing PR/queue condition stays exact.
Source laws prove these modes; an actual protected candidate must execute the
reusable WIT lane and its required report before merge acceptance.

The `e83` runtime campaign executes all 36 laws successfully, but configured
Check fails two source gates: the top workflow grows from 690 to 696 lines,
and the old instruction-workflow oracle requires its unchanged report command.
Those failures remain preserved. Moving the parallel WIT job into the existing
332-line source workflow restores the top workflow exactly and stays under 350
lines. The dependency-chain laws cover nested WIT failure, cancellation, skip
and missing results through the outer required report; no gate or cap is waived.

Existing-family drain takes priority. When a predecessor regenerates a queue
candidate, preserve its older green epoch as historical; require WIT/report
success on the current candidate. Do not jump or duplicate entries, change
queue configuration, or replay source merely because main advances.

Fresh exact-head Davinci Contracts must execute the real TS-48 guests in both
hosting modes. Configured Actions and protected candidate validation remain
required before actual merge. Local metadata and source tests establish the
lock repair, not WIT runtime or full language acceptance. This is main CI
stabilization; it adds no compiler pipeline, parser traversal or default route.
