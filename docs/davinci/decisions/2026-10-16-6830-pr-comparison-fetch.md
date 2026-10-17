# Reuse the checked-out PR comparison parent

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Observed source-planning jobs took 20–21 seconds, including 9–11 seconds in
the comparison-base fetch, while the actual source planner took under one
second. Checkout now includes two generations of commits so the tested merge
commit's direct parents are normally available without another network fetch.

The existing comparison-base resolver still validates full SHAs, the checked
out commit and the PR head before selecting the merge's first parent. A stale
event base never replaces that verified parent. An unverified PR merge keeps
its conservative event-base fallback, and a merge-group event keeps its own
declared base even when that commit is not a direct merge parent.

After resolution, the workflow uses `git cat-file -e "${comparison_base}^{commit}"`
to check local availability. It fetches the same validated SHA only when the
commit is missing. An all-zero resolved base still skips fetching. Invalid
comparison identities and failed fetches still fail the planning step. All
source selectors, package closures, native lanes and required reports remain
unchanged.

The existing comparison contracts now execute the actual workflow shell in
temporary Git repositories with shallow checkouts. Git traces prove no fetch
for a present verified parent or zero base, and exactly one fetch for a missing
parent, an unverified PR's event base, or a queue event base outside the direct
parents. Invalid SHAs and unavailable remote commits fail. The resulting file
diffs still exclude newer main changes for the verified PR merge and retain
the broader comparison for the conservative fallback.

Run `node --test tests/tooling/comparison-base.test.ts` for the focused proof.

## Measured Actions result

At `986d6cdb14d866bf23786099acbc4f57dff5c824`, the
[source-planning job](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547/job/116948290240)
completed in **11 seconds** and passed all 33 CI-selection tests. The
comparison step took approximately **62 milliseconds** between its log
boundaries. The log contains only checkout's depth-two fetch, with no additional
comparison fetch.

Checkout tested merge `44bbf9f52219ac9d5407080d3d3c7c60238643d1`. The planner
used its verified first parent,
`4e9366ee86d31cc9c1c03961d83505e24ebd4163`, despite the event still declaring
`cd861aa001f7ea458f8b0147e5b9c850b9db1c87`. This confirms both local-parent
reuse and the intended comparison boundary on the real Actions checkout.

The 11-second job is an observed cross-run improvement over the earlier
20–21-second jobs, not a controlled measure of total PR speed. All nine ordinary
PR workflows passed, but the complete observed interval was **8 minutes
8 seconds**. [Check](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547)
took 8 minutes 2 seconds, including 144 seconds of cancellation handoff before
its first job started, and 5 minutes 38 seconds after that start. This run did
not meet the five-minute target; the initial wait remains part of the reported
end-to-end latency.
