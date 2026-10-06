# Native diagnostic timeout status

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698), paired
[source decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6016856398).

The outer ten-second native diagnostic collection deadline previously logged a
warning and returned only the synchronous diagnostics. With an initialized
bridge, the separate unavailable hint was suppressed. The normal publication
path could therefore send an empty, versioned diagnostic vector even though no
complete native result had arrived. This is a source-proven correctness path;
it does not establish why the original release337 assembly parity law failed.

Reuse the existing complete `typecheck-timed-out` warning when that outer
deadline expires. Preserve synchronous diagnostics and the same ten-second
bound for both Vue/script and opt-in JSX collection. Completed empty and
positive replies, answered backend errors, fallback policy, native queries,
source/environment stamps, retries, publication ownership and original parity
expectations retain their existing behavior. The warning explicitly states
that type errors are missing; it is not a fabricated semantic diagnostic.

The authored public corpus binds a complete SFC and whole notification/vector
contracts. One controlled pending future exercises the actual ten-second
production wrapper and actual `ClientSocket` notification serialization;
completed empty, positive and answered-error controls distinguish timeout from
normal replies. These controls do not execute a native backend. Fresh Actions
must run them together with the unchanged original native parity/lifecycle and
non-native laws. No local native build or execution has been performed.

The scheduling audit separately finds that native SDK IPC already runs on the
dedicated `vize-corsa-bridge` worker. Hover and diagnostics share Maestro's
source transaction lock and that worker's FIFO session owner. Dropping an
outer wait cancels only an unentered worker job; entered synchronous work still
drains before its client and mirror keepalive are released. The SDK exposes no
public outgoing request-id cancellation handle. Cache/fingerprint/project
preparation remains synchronous before the first native worker await. These
facts justify investigating queue waits, but do not prove a measured latency
cause or gain. This change adds no offload, preemption, process or pipeline.

Remaining work: authenticate fresh source/native/protected execution and actual
delivery, retain the failed release337 receipt, and qualify the separate
readiness scheduling change with every started worker drained and all original
queries/stamps intact. A timeout warning alone does not make original diagnostic
parity pass or establish that the user's reported hover delay is resolved.

The first exact-head PR scaling check stopped before building or measuring: its
closed source allow-list omitted the new timeout wrapper and its test leaf. Add
only those two literal paths to the existing allow-list, preserving the complete
original400 inputs, vectors, recipe, timers and limits. Retain the actual failed
job and require fresh source-bound qualification; no runtime result transfers.

Paired source-binding correction: [#7698 decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6017106748).
