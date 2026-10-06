# Per-document native readiness scheduling (#7698)

The current 534-document public workload acknowledges every changed URI through
the installed native `textDocument/documentSymbol` API. It retains four drains
after each 128 notifications, complete project/options authority, and the current
session generation before accepting native queries.

Pinned Corsa 1.14.0 waits in `std::sync::mpsc::Receiver::recv_timeout` or `recv`
inside its request future's poll. `buffer_unordered(16)` stores sixteen futures
but does not poll another while the first blocks. The existing current-thread
executor therefore sends these readiness requests serially. This source result
does not attribute all observed readiness time to transport waiting.

Use at most sixteen scoped workers inside the existing readiness operation.
Each borrows the same native client and handles multiple URIs using the unchanged
supported request API. Every changed URI still obtains its own complete response;
there is no final-single-response shortcut or new pipeline stage. The caller's
mutable session ownership fixes the mirrored text/version and generation while
the workers run. A symbol response does not echo a document version: that binding
comes from the unchanged overlay and session generation, not an invented wire
field. Explicit project/options checks and notification drains remain unchanged.

On any native error or thread-creation failure, stop assigning further work and
join every started worker before returning the first recorded error. Per-request
timeouts remain the transport's existing limits. An entered Corsa bridge operation
already drains after its async caller abandons it; the scoped workers must finish
before that operation returns or retires its session. A failed or partial barrier
never clears the dirty set or accepts its generation. Panicking workers are
joined and reported as failures, without a fallback readiness result.

The controlled socket law withholds every response until sixteen requests arrive,
then returns full responses in reverse order for all 534 distinct URIs. It checks
the complete result for each URI, exact coverage and the sixteen-request ceiling.
The refusal law checks that every entered request has returned before the barrier
propagates the native error. Existing native and public original400+134 qualification
continues through the same workflow; only the two new owned source paths join its
closed changed-source authority. Inputs, whole query/publication/diagnostic
oracles, budgets and source/binary custody remain unchanged.

The SDK still acquires each requested project's language service synchronously
on its dispatcher. Worker creation and joining also cost time and memory. No
speedup is claimed until the existing source-bound Actions comparison establishes
the complete original outcomes and timings. Record actual overlap, full per-URI
responses, worker CPU/RSS and all cleanup/error outcomes alongside the existing
phase boundaries; do not infer a sixteen-fold gain from the worker count.

The separate Maestro outer ten-second timeout starts before polling diagnostics
and polls the operation before its timer. A blocking poll can pass its deadline;
if it later returns Pending the timer may win, while a Ready result still wins
that poll. This is a source-supported scheduling possibility, not the observed
cause of the retained release assembly-parity failure. This change does not alter
that timeout, its original expected diagnostics or the owner's diagnostic capture.

TODO: retain fresh exact-source native/full/scaling and protected queue results,
then actual signed delivery. The full-command ten-times target remains open.

Paired decision: https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6016599405.

The first exact-source tooling run retained one stale-inventory failure. The
unchanged scanner and renderer regenerate the complete Canon typechecker shard
with two additional rows: the worker's L0 import at line10 and its test import at
line23. Removing those rows recovers every previous byte. The drift guard and
all production, native, workload and expected-result sources remain unchanged;
fresh exact-source Actions must validate the correction.

Paired correction: https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6017107629.

## Qualified source pair and genuine main integration

[Current qualification](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6017672839)
retains exact53b9 run37469766002/artifact11417116944:810 safe CRC members,
79 complete measured envelopes, and18 whole notifications (one initialization
log plus17 diagnostic publications). First534 readiness was463.192→191.205ms
and first hover2011.620→1711.218ms; four128 drains392.235→411.980ms and warm20
median21.136→22.407ms showed no warm improvement. These are one same-worker
shipping-profile source pair, without installed/public/general10x credit.

Both sides retain zero diagnostic timeout warnings and seven release warnings;
final observations have two same-birth zombies before and none after, without
OS-reaping or Program-reuse proof. Four strict consumer artifacts are fresh on
both sides; after reuses28 unchanged linked locals of33. Binary hashes join
receipts without independent ELF bytes. The complete27-package graph binds
actual Vue3.6.0-beta.10 and physical nativeSDK7.0.2.

Genuinely incorporate signed main de6f9e9294cebd27a41fa2884dffefc47d820d3d,
keeping all incoming bytes and scheduling/control bodies. Only the adjacent
owned canonical conflict is resolved. The53b9 receipt remains historical;
fresh current source/native/scaling/protected gates and actual signed delivery
with the terminal reporter footer remain required.
