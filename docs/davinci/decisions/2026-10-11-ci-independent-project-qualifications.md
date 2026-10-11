# Overlap the independent project qualifications

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The successful merge-queue Check
[38119948134](https://github.com/ubugeeei-prod/vize/actions/runs/38119948134)
at `c513b959a0096cab2593667f23544361396acb83` spent 268.412 seconds on
`test:js`, then 744.037 seconds on the original project transport. Its existing
artifact `11694987459` has the verified server/download SHA-256
`1e98992cf81df1f4cbef037da150f634e050c63a241dd4646c648b0d0f3fe828`.
The original phase receipts measure:

| Phase                                         |       Wall time |
| --------------------------------------------- | --------------: |
| Complete #8507 source and historical controls | 162.382 seconds |
| Licensed n8n bridge preparation               | 182.237 seconds |
| Both pinned hosts, already concurrent         | 387.199 seconds |
| Complete evidence aggregation                 |   9.997 seconds |

Run the #8507 and n8n branches concurrently in the existing job. Both already
start real worker processes. They only read the same authenticated, frozen
source addon; their evidence and physical temporary projects are separate.
Neither branch consumes the other's result. Join every terminal outcome before
accepting the final qualification, cleaning temporary projects, or uploading.
Keep a complete branch-outcomes record, including failure and interruption.

Retain all 98 #8507 source observations (45 CLI and 8 native control cells), all
24 historical controls, both pinned host/type versions, all 41 HTML cases per
host, and all 1,369 licensed n8n inputs and 18 original bridge/host phases. The
original 180-second mixed-worker deadline, 900-second bridge deadline,
1,800-second host timeout, output bounds, source/native identities, complete
ordered packets and refusal controls remain. A failed or interrupted worker
group is terminated and joined, including descendants whose leader exits first.
Real-process laws cover successful overlap, red/cancelled/timed-out children,
parent cancellation, complete raw outcomes, and remaining-branch completion.
Keep command waits asynchronous in the owning parent, including input laws,
submodule preparation, reference extraction/build and historical comparisons.
The same commands, arguments, input bytes, output caps and timeouts remain.
Otherwise a sibling's synchronous wait can delay another live group's deadline:
the original counterexample let both 500ms workers write at 800ms during a
1,500ms wait. The repaired real-process controls refuse before those writes,
join the remaining command, and drain descendants. Perform the complete final
aggregation only after both workload branches settle.
Preserve the reference existence command's complete outcome before checking it.
Cancellation or spawn failure refuses immediately; only a normal missing-object
exit can take the original pinned fetch path. Real-function controls cover both.

The 162.382-second independent phase is an ideal upper bound on savings, not a
measured speedup. Additional CPU/memory contention is unmeasured. Existing
receipts record wall time, not CPU/RSS. Qualify the sealed source with the
existing JS gate and compare the same complete workload before claiming a
result. No result cache, additional native build, workflow, job, fixture
reduction, or public installed-acceptance credit is introduced.
