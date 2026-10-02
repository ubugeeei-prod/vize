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

At original preparation, ten genuine DocumentStore/query test functions and
eight optional original edit/Program functions were source-ready and unexecuted.
The later actual eighteen-test campaign is recorded below; the new lifecycle
functions still require execution. Initial local checks covered rustfmt, source
API/tree/patch conservation and diff checks only. The generic host query
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

The actual whole-source campaign `37042290190` at
`6dddbd44575a9359f33ac0e86af93cdb7ca2e62b`, Rust job `110955102871`,
compiled the genuine host/edit/parser source together and passed all eighteen
unique test functions: ten default plus eighteen feature executions, with zero
failed or ignored. The minimal feature check passed at 17:48:39Z. Warning-denying
feature Clippy then rejected the URI helper unwrap at edit_tests.rs:11, outside
a test function. The expect correction retains every valid URI input and assertion;
no lint waiver is added. The failed complete Rust/full campaign stays failed.

## Scoped actual host lifecycle cancellation

This bounded source draft follows the genuine current source-project consumer
`6dddbd44575a9359f33ac0e86af93cdb7ca2e62b`. That consumer's eighteen real
host/edit/parser functions still require their in-flight full Actions proof.
This draft contains eleven additional real host/cancellation tests and none has
executed. Rustfmt is the only draft check performed so far.

`SourceQueryProject` borrows the caller's actual DocumentStore and forwards its
real open, version-ordered incremental change, close and rename methods. A
successful lifecycle event removes only matching source-query leases and aborts
them after releasing every registry/lifecycle lock. Rejected stale changes,
rename collisions and no-op same-URI renames preserve pending work. A small
synchronous lock makes original snapshot capture and cancellation registration
one lifecycle boundary; it never survives an await. Dropping work removes its
lease, completed results stay cancellable until final guarded publication, and
project shutdown wakes retained computations. Wakers run after all locks drop.

Source queries continue to use the actual URI, monotonic host revision, client
version and once-retained original buffer. Genuine original edit/parser APIs
remain in the parent consumer. No request dispatch/default-feature switch,
legacy reparse, caller File capability or native admission is introduced.
External raw DocumentStore mutations still fail final freshness checks but do
not automatically notify this scoped project. Product handlers must explicitly
use this real lifecycle API before automatic production cancellation is claimed.

The final publication must fold the same decision into central LSP paragraph
229 and a #6883 comment, verify every parent-provider byte and run actual full
Actions/all100 on the new exact source head. The known previous full failures
remain failures. Native File/SFC integration, host atomic edit mutation,
complete LSP response history and production selection stay separately pending.

Independent source review found a retained Waker destructor could run while
QueryLease removed its final registry handle under the mutex. The correction
moves the entire removed request out, releases the guard, then drops its handle.
A tenth actual stored-Waker destructor regression checks registry re-entry
without hanging the runner. Original five draft bytes and their first receipt
remain retained in original-v1; this correction has no execution credit yet.

The actual host accepts a higher-version empty change list without changing
this document revision/version. The wrapper now compares those real host values
before and after accepted changes and preserves the original cache/leases when
they remain equal. An eleventh real-host regression covers this no-op. The
incremental cancellation witness now uses an actual UTF-16 range over an astral
character. Accepted equal-text replacements still receive fresh host identity
and correctly cancel old work. Original v1/v2 bytes and receipts stay retained.

The new source campaign tests all twenty-one host/lifecycle functions without
the optional parser dependency, then all twenty-nine functions with the genuine
edit/parser feature. These are fifty executions of twenty-nine unique functions;
the eleven new lifecycle functions remain unexecuted at this source freeze.
The private pending-query helper also uses expect for its genuine host URI
precondition. All actual host/provider bytes and production dispatch remain
unchanged. This decision is paired with #6883 in the same publication.

## Actual twenty-nine functions and test-helper correction

The genuine whole-source campaign `37044408941` at
`1dc47741ee45ee6bb5568c0665123a883f617a44`, Rust job `110962163524`,
passed all twenty-nine unique functions: twenty-one default plus twenty-nine
feature executions, with zero failed or ignored. The minimal feature check
passed at 18:07:33Z. Warning-denying feature Clippy rejected the test URI
helper expect at edit_tests.rs:11; replacing unwrap with expect had not
resolved the earlier helper lint. Both original complete campaigns remain
failed. All-100 run `37044412195` passed at the same source head.

The helper now returns the genuine URI parse Result. The same seven successful
URI parses are unwrapped only inside their existing test functions, which
retain all original URI/source inputs and assertions. No lint suppression,
new dependency, production API, host mutation, or product route is added.
The twenty-nine functions are previously executed at the original head; the
new correction needs its own full Actions, minimal check, feature Clippy and
all-100 proof. The actual provider ancestry remains the original public 860
source; it must be conserved onto the owner’s corrected public provider before
any dependent native Stack publication. This decision is paired with #6883.

## Genuine corrected public provider delivery

The own-only delivery is a direct child of public span-edit provider
`ef4eec0566b9f0208bb98a096638df184b54b070`, whose actual parent is
`ccfcc7a229b36816d1f680a06d95ca53e89da23b` on authenticated signed main
`d87109e65ef77e12d9a19f1d096804bddddd6694`. The five genuine L1
edit/embedding source blobs remain byte-identical to original 860. The host
DocumentStore still assigns process-global revisions on open and mutation;
close/reopen with equal client versions cannot recover the retained snapshot.

Only the eleven owned adapter/test sources are copied from corrected 668.
The current lockfile, complete incoming history/security/provider source,
central paragraph 291 and current inventory counts are retained. Semantic
feature/module/CI edits and only the two Maestro-generated inventories are
added. Central paragraph 229 receives this bounded additive record. Earlier
29-function execution and failed complete campaigns remain original-head
proof, while corrected 668 full `37048041455` and all100 `37048042921`
are ongoing at replay time. This new genuine-parent head requires fresh
full Actions, feature Clippy, all100 and protected terminal acceptance.
Production lifecycle/request routing, atomic host edit application, genuine
File custody, complete native responses and #6883 history admission remain
unfinished. The new independent source decision is paired with #6883.

The actual corrective 668 campaign `37048041455`, Rust job
`110974242136`, again passed all twenty-nine unique functions in fifty
executions and minimal feature compilation. Feature Clippy failed only
`manual_inspect` on the URI helper identity-map at 18:40:13Z. The original
log is retained unchanged and the complete campaign remains failed; its
all100 `37048042921` succeeded separately. The final helper uses Result::inspect
for the same successful-parse side effect and returns the unchanged original
Result, with no unwrap/expect outside the existing test functions. All seven
URI/source inputs and assertions remain unchanged. This direct genuine-parent
delivery needs fresh source/full/Clippy/all100 proof at its final exact head.

## Actual merged providers and complete source validation

The original corrected source head `c351625102ccb49d7ad9b21f3651ef26f7b68b36`
passed manual full Check `37049990879`: all twenty-nine unique functions passed
in the designated twenty-one default and twenty-nine feature executions; the
full workspace also reran the default functions. The minimal feature check
and warning-denying feature Clippy passed. All nineteen full jobs succeeded,
with five declared skips; tooling passed 5,484 tests with twelve skips and no
failures. Real instruction-count run `37049991180` passed all one hundred
unchanged pinned ceilings across three executions, with ratchet and ceiling
checks. The original helper-lint failures remain retained original-head proof.

Native Program PR #7454 actually merged at 2026-10-02T19:23:13Z as
`1b9442821aa2044008d0e5e14102e44bbd25cb8d`. Span provider PRs #7434/#7435
actually merged at 2026-10-02T19:25:25Z as `d54c306eb91919a8b5dfcc9c3d3c2018f513f09e`
and `fb41f5216ac61442fda82747ff5ca508eed37d6b`. Their native Stack #7436 is
closed after terminal protected acceptance. This own-only source delivery
uses authenticated signed main `ae08f05c65ab5d951df130b72063d4c163a9c275`,
which also retains formatter PR #7456. The eleven adapter/test source files
remain byte-identical to c351. All incoming native Program, span edit,
formatter, security and history sources and decision paragraphs are retained.
The current lockfile and existing named CI witness are unchanged.

This replay has no new runtime execution credit: its exact new head requires
fresh full Actions, feature Clippy, all one hundred counts and protected
terminal acceptance before completion. Production lifecycle/request routing,
atomic host edit application, genuine File custody and complete native LSP
response/fix-history admission remain unfinished; #6883 stays open.
