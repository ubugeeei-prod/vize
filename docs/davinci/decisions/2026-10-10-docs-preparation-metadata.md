# Publisher preparation metadata visibility

Issue: [#8521](https://github.com/ubugeeei-prod/vize/issues/8521).

Status: source correction; exact-head Actions, protected merge and actual repaired
main publisher execution remain required.

## Original failure

Two actual main publishers executed their validation step before GitHub's primary
Jobs API exposed the step's start timestamp:

| Publisher                                                                                      | Job          | Later primary start timestamp |
| ---------------------------------------------------------------------------------------------- | ------------ | ----------------------------- |
| [38065064638](https://github.com/ubugeeei-prod/vize/actions/runs/38065064638/job/114250930859) | 114250930859 | `2026-10-10T15:48:23Z`        |
| [38065628811](https://github.com/ubugeeei-prod/vize/actions/runs/38065628811/job/114252559712) | 114252559712 | `2026-10-10T15:56:37Z`        |

The unchanged original entrypoint at
`f2f8a250c5b22882c33786a97727557d7c662927` retained `actual: null` at its
`preparationStartedAt` assertion. Both actual Pages steps were skipped. The later
primary job records expose the timestamps above; they do not retroactively
qualify either failed publisher.

The observed public source advance from `b7198eaea1710fec56d532ffe0a2dfa206d6080e`
to `5ba29d8e204725965856fd17b8a548f516748350` follows the actual protected main
lineage. It was forward publication, not a rollback. The complete five-status
primary audit found no active obsolete publisher requiring cancellation.

## Bounded current-job refresh

Read only the same authenticated current run, attempt, job and validation step.
If its primary `started_at` is null, wait two seconds and read again, using at
most six snapshots and five waits: the existing ten-second metadata refresh
budget. The first visible authentic timestamp is retained as the writer-journal
cutoff. A visible initial timestamp adds no delay.

Every snapshot retains the original repository, main branch, workflow, trigger,
run, attempt and source checks. Its job must bind that publisher's run, attempt
and source. The first job/workflow identity remains fixed across all snapshots.
The publisher and job must remain active, and exactly one named preparation step
must have no terminal conclusion or completion timestamp. Missing, duplicate,
malformed, foreign, changed or exhausted metadata refuses publication.

There is no local-clock substitute. The existing whole-job Pages lock, complete
successful Docs gates, source ancestry, original artifact custody, immutable
receipt, active-writer journal and actual Pages-effect assertions are unchanged.
Unknown concurrent environment writers still refuse publication.

## Verification and delivery

The regressions retain both observed run/job/timestamp identities and exercise
null-to-visible starts, immediately visible starts, six exhausted null snapshots,
foreign jobs/repositories/branches/workflows, changed run/attempt/job/source,
invalid or terminal steps, and a terminal transition after an initial null. All
original deployment laws remain unchanged.

Exact-source Actions and protected merge qualification are separate from source
tests. After actual merge, require an actual main publisher to execute the
repaired entrypoint and bind its primary preparation timestamp to the original
successful Docs source. A clean equal/older-source skip is a valid guarded
publisher outcome; an actual successful Pages step is required to claim a new
publication. Existing historical public browser proofs remain bound to their
own deployed sources.
