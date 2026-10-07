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

## Historical qualification and actual security incorporation

Immutable `718aaa16ec541d2a9644f420b9ba64c01a653553` passed configured formatting/Clippy and all 16,505 source Rust records. Native run `37485363974` preserved six whole corpus/mode output rows, 18 shard memberships and existing freshness controls. Original400 run `37485363954` preserved all 79 complete query vectors and 18 authored envelopes against signed `8fab`. Its ordinary run `37485365115` failed at the inherited new shell-quote advisory and dependent report; retain that failure.

The single same-worker shipping pair demonstrates no hover speed benefit: first hover 2438.65→2476.40ms and warm median 27.57→29.30ms are worse; background median 85.46→78.71ms is better. It does not establish the copy's timing contribution, a general speed gain, old diagnostic-silence cause or installed-public acceptance.

After #8137 actually signed-merged, incorporate genuine main `ef84821d30fa0d8538b2472fef34418e75380523` once. Preserve all three transfer production blobs and incoming readiness, security, workflows and decisions. Every `718` result above remains historical for the composed successor; fresh exact-head source/native/original400 Actions and protected signed delivery are required. Issue #7698 remains open.

Paired incorporation decision: [comment 6033188073](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6033188073).
