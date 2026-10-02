# Real host snapshots, cancellation and authored edit output

The source-project adapter consumes the actual Maestro DocumentStore source at
`860dfb87802d871352e9a6bbb95ba33605eaa607`, a genuine span-edit child on
accepted signed main `21187137f15e981844898d90276fda5df1b6c3cc`. Its actual
document/store/text bytes equal the retained 513a host source. Capture reads URI,
Document::revision(), client version and text together under one read guard.
The once-owned Arc<str> is reused only for the same actual host revision and
version; equal text or reset editor versions cannot substitute identity. Actual
incremental changes, same-text updates, reopen and rename invalidate old stamps.
No shard guard survives any await.

SourceQuery uses the genuine futures AbortHandle/Abortable registration and an
initial cooperative yield. Explicit cancellation wakes suspended work; dropping
the request future drops its computation. Cancellation does not preempt arbitrary
synchronous work within one poll. Completed results must call publish, which
checks the actual current host and cancellation under a fresh read guard before
its synchronous callback. That callback must not await or re-enter DocumentStore
in any way, including reads or nested apply_edits. A queued writer can make even
recursive shard reads deadlock under writer fairness. Compute native edit output
before this callback; publish only a prepared synchronous response or cache
result within it. This boundary does not cover asynchronous transport completion
or an atomic editor/server mutation.

The optional experimental-source-project feature binds the genuine original
VersionedSource/EditSet source at `4341acd9ad6df6bc8eb5d2ea45aca0daca436709`; docs-only
delivery `81382c1e5af399cdb6d7358420890ebdaccef23f` preserves those five runtime source
files exactly. Current source replay `c8500067a66dcf960fbb4e212df79d4d3e8c17bf`
then `860dfb87802d871352e9a6bbb95ba33605eaa607` preserves those five original
runtime blobs; this consumer branches from that actual provider head. Its public
Actions and terminal merge remain pending at preparation. It borrows the
captured original buffer and uses actual URI plus
revision as key and the actual client version. Edit application checks the live
host while read-locked, then supplies the same retained physical root to the
source-owned edit model. A foreign equal-text root is refused. Successful
application produces new authored text and leaves DocumentStore unchanged.
DocumentStore::get_mut bumps a stamp before returning and apply_changes is
version-ordered; neither currently exposes a conditional-revision atomic write.
Such a host CAS API and client edit dispatch remain future work.

The authentic Shared parser API is source-owned
`a04fdf81bad61ad32d1bd96a7cfd816e4796cb70`, now genuinely merged through
Stock #7427 / Shared #7428 and retained in signed main 21187137. All original
parser-provider blobs match this actual parent. Actual native
query tests parse the retained host buffer once through parse_program_once,
assert original parser-owned admission/source identity and retain comment spans;
recovery remains unadmitted. No helper provider, copied equal-string parser input
or historical cached ABI supplies execution evidence. A fresh exact consumer
head must contain the actual authored-root/edit and Shared parser source prefixes
before full feature Actions can compile this adapter. They are explicit source
dependencies. The signed main already contains genuine Shared; the current
span-edit provider and this consumer still need their own fresh exact-head
Actions and protected terminal merges.

Ten genuine DocumentStore/query test functions and eight optional original edit/
Program functions are source-ready and unexecuted here. Only rustfmt, source API
checks, tree/patch conservation and diff checks were run. The generic host query
module changes no request dispatch/default features; the optional dependency
points from crates to davinci. The existing frozen four-layer packet and runtime
captures are untouched.

Native File is deliberately a typed NativeFileUnavailable precondition with an
uninhabited success type. No caller-supplied capability or zero-origin Program
mints complete File authority. The actual full-file producer and its authentic
Program/component receipts are still required. Production LSP replacement waits
for #6883 closure; native File/product/runtime comparison, whole-history closure,
all instruction ceilings, fresh Actions, protected queue and actual merge remain
pending. Publish this decision with the paired #6883 comment in the same change.

Current provider publication is independently verified at
`860dfb87802d871352e9a6bbb95ba33605eaa607` in native Stack #7436
(#7434/#7435); terminal provider merge is still pending. The original consumer
receipt preserves its preparation-time public-verification false value. This
additive record corrects that current transport fact without claiming execution.
The native Program campaign exposed two inherited tooling witnesses that assumed
fixed workspace/differential step positions. This consumer applies the same
exact named-step lookup and TSGO envelope; the original workspace condition,
eleven differential commands, argument/count checks and simulated failure exit
remain intact. All four affected tooling tests pass locally. Original eighteen
host/edit/parser test functions and all actual provider source bytes remain
unchanged. Whole feature Actions, all100, actual provider merge and protected
consumer acceptance are still pending; no Cargo build or installation ran locally.
