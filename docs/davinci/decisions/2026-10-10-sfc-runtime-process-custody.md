# Complete SFC runtime child custody

Issue: [#7951](https://github.com/ubugeeei-prod/vize/issues/7951).

The full [Check 38029867272](https://github.com/ubugeeei-prod/vize/actions/runs/38029867272)
at `9b64938d93387c82a2b91f655de67d2ed45e79d5` failed
`nested_component_prop_and_event_arguments_share_the_selected_key` at the strict
SFC child exit assertion. Its binary reported 30 passed and one failed. The
printed Node stderr is empty; the assertion did not retain its exit code,
signal, stdout or complete child input. The actual cause is unknown.

The original `names]` and `events]` string keys, nested selected indices,
shorthand and longhand directives, all eleven state observations, child prop
checks, stale listener checks and full VDOM/Vapor comparison remain unchanged.
The logged parent module executes in three local fresh processes with the pinned
Vue 3.6.0-rc.9 runtime and an official complete child module, producing identical
9,498-byte traces with zero exit and empty stderr. That child is not the original
source-built Vize child; this smoke check neither qualifies the original corpus
nor identifies the failure.

Before enforcing the unchanged successful-exit law, preserve the actual once
serialized stdin, stdout and stderr as separate raw files. Retain the unchanged
authored parent and child SFC bytes separately, including the default child
which is not present in the Node protocol. The receipt identifies
the Rust and child PIDs, command arguments, status, exit code, Unix signal,
stdin write outcome and every stream/authored-source length/SHA-256. Record the checked-out Git
head and tree separately from `GITHUB_SHA`. These source labels do not identify
the actual Node executable or engine; that identity remains explicitly unknown
unless separately captured from this child.

Each invocation owns a process/ordinal directory under
`target/nextest/{pr|full}/vapor-sfc/`. Existing named template-ref receipts keep
their original format. Required archive workers already retain this directory.
The full Check and merge differential callers also upload it after success or failure,
without changing selected tests, concurrency, compiler/runtime behavior,
production modes, or any oracle. A nonzero child still fails before comparing
its output. Both existing stdin-write and child-exit failure messages retain
every stream byte and full input even
if supplemental file capture fails.

The independent isolated original-archive/image/core investigation owns its
existing diagnostic preload and workflow. This change introduces no duplicate
preload, signal handler, timeout, retry, alternate acceptance rule or causal
claim. No product behavior, dependency/publisher graph, release metadata,
instruction ceiling, native-history disposition or product route changes.

TODO: inspect actual source/full/protected outcomes at the new exact head;
diagnose a reproduced failure only from its genuine packet. A successful later
execution does not explain or erase the missing original outcome. #7951 remains
open, and no compiler fix or release completion is claimed by this diagnostic.

The first source Check 38032923241 at `e8c4d1a03` failed to build this new
helper: the pinned SHA-256 output array does not implement `LowerHex`. The
full test-scripts worker 114157485578 reports the same `E0277`; no original
SFC runtime outcome was produced there. Encode each digest byte as two hex
digits, as the existing source-custody helper does. Original tests and all
acceptance predicates remain unchanged; new exact-head Actions are required.

At source head `e632a1e536599431aa60afe794272d60b3c10770`,
[source Check 38033422816](https://github.com/ubugeeei-prod/vize/actions/runs/38033422816)
built the integration tests and all four required Rust shards passed. The
[original nested case](https://github.com/ubugeeei-prod/vize/actions/runs/38033422816/job/114160192440)
passed in 14.205 seconds. Its four original shorthand/longhand VDOM/Vapor
invocations each retained 9,498 stdout bytes with SHA-256
`3ad76a48e60a9595118ec9190333be1ed1cadc37803b9644fe6e64db50bc11a2`.
All 151 SFC packets across the four shard artifacts have matching raw stream
and authored-source lengths/hashes, successful status, code zero and no signal.
Their observed checkout source is the PR merge object
`177db62bb669bfd362ddcbdb900739f337ba799c`, distinct from the PR head; their
actual Node identity remains unknown.

That source run also exposed three tooling workers failing the new integration:
the feature recipe permits only its original executable steps, and the module
layout gate requires ordinary discovery. Retain its original recipe byte for
byte, place always uploads after the two caller invocations, and declare the
evidence helper under the existing ordinary fixture module. No gate or fixture
is weakened. The raw failed worker logs have SHA-256
`7159f66d268530058b7a4384e647fc43b2a4a775230663cfd43fb2c446382596`
(worker 114159269991),
`570e9aa4c119b66c06ba0b69dec5f437a12edf085c3e71addab1e8a2af190890`
(worker 114159270003) and
`f6301322563d0227e6c13dde5a442f5c0e793937ebf86b33905c5d409d566dcd`
(worker 114159269905). The complete source run is not accepted; the corrected
head still requires its own source, full and protected results.
