# CI stack replay after v0.429.0 publication

Tracked in [#6861](https://github.com/ubugeeei-prod/vize/issues/6861),
[#6891](https://github.com/ubugeeei-prod/vize/issues/6891) and
[#6865](https://github.com/ubugeeei-prod/vize/issues/6865).

- Replay #6899, #6900 and #6901 on main `dac6b2dac6d8f3d394b1ddd7a8d974ca79be5883`.
  Preserve the original branches and every existing source suite and gate.
- Preserve main's Nuxt critical CSS decision entry when appending the timing
  decision. Preserve every existing L2/L3 baseline row while adding the two
  formatter inputs. The merged L2 baseline contains 448 files, 1031 remarks,
  158 applied and 873 missed; its generated backlog summary uses those counts.
  These counts are derived from the merged baseline, pending exact-head Actions
  corpus validation. No new Rust build or formatter CLI capture is claimed.
- Preserve the original formatter capture and `corpus-integration.json` as
  historical observations of their recorded source revisions. Their counts
  and publication state are historical evidence, not claims about this replay.
- Release publication is no longer a merge ordering blocker. Fresh exact-head
  Actions must pass before enabling squash auto-merge. Actual merge-group
  success remains required for #6865; measured normal CI timing evidence
  remains required for #6861. Neither issue is closed by replay alone.

## Replayed layers

| PR    | Existing head | Replay head before this record | Local replay branch                                  |
| ----- | ------------- | ------------------------------ | ---------------------------------------------------- |
| #6899 | `8e26ee332`   | `ddc5e5bf3`                    | `ci/rust-test-phase-timings-resume-20260927`         |
| #6900 | `c9e02592c`   | `c94a7a292`                    | `test/differential-tooling-receipts-resume-20260927` |
| #6901 | `de61316b9`   | `1c1d77068`                    | `ci/native-stack-resume-20260927`                    |

Seven of the eight replayed commits retain identical zero-context stable
patch IDs. The corpus-integration commit changes only the count summary to
retain main's three additional Nuxt inputs and existing missed remark. Removing
exactly the new formatter rows restores both L2 and L3 baseline bytes to main.
The bottom layer's workflow, timing helper, test and measurement protocol are
byte identical to its existing PR head; the remaining bottom-tree differences
are inherited from the three main fixes.

Native GitHub stack #6902 is a `PullRequestStack`, not a PR or issue. Its
GraphQL ID is `PRS_kwDOQvjuaM4AFqpa`; its six entries are #6899, #6900, #6901,
#6905, #6906 and #6907. The latter three layers are managed separately from this
replay. Existing Check, Zizmor, Nuxt and title workflow PR branch filters accept
only main/davinci, so stacked branches need deliberate CI triggering and base
management. No automatic rebasing behavior was verified by this replay.

## Replay after the first queue merges

#6899 and #6900 merged through the main queue. Replay #6901 on main
`3bc2416ad`, preserving both merged input selectors and the complete
feature-corpus queue recipe. The open layers were unstacked from #6902 to
retarget its next layer after a documentation conflict prevented automatic
retargeting. The two merged members remain as historical stack entries.

The Check/title-policy trigger repair in #6910 has the same stable patch ID
(`b40d5af24e8269d70fdb43503f378df69e4fe916`) as the repair already in #6901;
the parity assertion is also identical. Close #6910 as superseded by #6901
and validate the combined head once. This records deduplication, not a passed
new head or completed T1 gate. Restack dependent CI and foundation layers on
the verified parent; every new head still requires fresh Actions and queue
validation before merge.

## Focused verification

- 37 planner, workflow, receipt, queue and corpus-gate tests pass with no
  failures, cancellations or skips. Real formatter CLI execution is deferred
  to the owning Actions build receipt and test lane.
- Scoped type-aware lint passes with zero warnings; formatting passes on 24
  matching changed files. Whitespace checks pass.
- Croquis consumers, consumer migration surfaces, storage summary, source
  location inventory and rule parity checks pass without regeneration.
  Assertion lint passes with the committed allowlist.
- The repository source-length checker passes against main `dac6b2dac`, with
  no new or grown files over 350 lines. Only this standalone check tool was
  compiled; no product or Rust workspace build was performed.
- An extra corpus-coverage report check is stale both on the replay and on
  unchanged main. This pre-existing hydration/report result is left visible;
  it is not counted among the five passing inventory checks.
