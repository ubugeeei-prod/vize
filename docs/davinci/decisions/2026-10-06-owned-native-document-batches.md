# Owned native document batches

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).

On signed `8fabdbcafd3996024b2fa6dc677e1ce3bc1eb862`, Canon builds an
owned vector containing the complete Vue or script project document batch.
Its synchronization method borrows that vector and the bridge immediately
copies every URI and text into another owned vector for the existing worker.
The first vector has no later consumer.

Transfer the prepared vector into a crate-internal bridge entry instead. Keep
the public borrowed entry and its existing payload copy for callers that retain
their batch. The internal entry contains the exact existing worker body:
`did_open_batch_fast`, diagnostic cache length, and cache statistics update.
Materialized project synchronization still finishes before opening the batch.
The worker receives every original document in its original order.

This removes one complete batch copy for each canonical Vue or script open.
It adds no cache or SDK request and changes no source map, project membership,
readiness acknowledgement, notification drain, epoch, timeout, recovery,
cancellation, error, or cleanup contract. The scoped readiness and caller
retirement changes remain separate owners.

Preserve every existing original400 fixture, response vector, protocol capture,
provider and release recipe. Add only the two exact production paths to its
closed source eligibility set so the existing Actions comparison can qualify
this delta; update the recorded scope to name the owned transfer. No workload,
stage, count, deadline or acceptance ceiling changes.

The existing public400 phase captures show significant preparation and protocol
work remains. They are different source pairs, not measurements of this change
or proof of the user's reported cause. The copy's contribution to latency or RSS
is unknown; no measured gain or general speed claim is accepted.

Before admission, require fresh ordinary source checks, the original native
contracts and the existing complete400 Actions comparison on the exact head.
The protected queue must retain all current instruction ceilings and original
whole-output contracts. This record is source preparation, not runtime proof;
#7698 remains open for unfinished performance work.

Paired issue decision: [comment 6019045579](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6019045579).
