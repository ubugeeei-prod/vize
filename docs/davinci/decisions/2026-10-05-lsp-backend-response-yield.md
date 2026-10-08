# Keep LSP dispatch active while type requests wait

Issue: [#8012](https://github.com/ubugeeei-prod/vize/issues/8012).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8012#issuecomment-5989126941).

The original report and complete App.vue/useToast.ts are retained under
`tests/_fixtures/lsp-corsa-responsiveness-8012/`, with the reported package and
strict Bundler project shape. The two sources were compared byte-for-byte with
the saved issue body. Reported 1.5–2.6 s / 58 s stalls are reporter observations,
not measurements of this source proposal.

Corsa IPC already runs on a dedicated worker, but the async bridge waits for
its reply synchronously on the foreground executor. One poll therefore prevents
syntax handlers, cancellation and lifecycle messages from being dispatched.
The old #3377 guard justification is stale: IDE contexts own Arc snapshots and
DocumentStore text/version reads release their shard guards before returning.

Retain the synchronous worker API and every existing deadline/drain law. Async
bridge calls receive worker-owned replies through the existing futures oneshot
dependency. One process-wide timer thread wakes expired absolute per-job
deadlines, captured at submission rather than reset when queued work starts.
There is no new dependency, per-request thread, reply-waiter thread, global
runtime change or additional native compilation stage. Dropped replies skip
unentered work; entered synchronous IPC drains without delivering its result,
preserving framing and session resource keepalive. Outstanding abandonment
retains the existing fast timeout refusal until it drains. Completion/drop also
clears abandonment when worker failure drops a queued closure.

Startup also needs retained ownership. The server caches the pending bridge
before awaiting its handshake; cancellation keeps that same worker instead of
starting another backend. An additive `CorsaBridge::is_draining` probe refuses
retry during uncancellable work. After drain, a failed cancelled startup may
retry on that worker without poisoning the existing initialization-failure
latch; actual returned timeout/failure retains the old latch and fallback.
Initialization flags and entered shutdown's session cleanup belong to the
worker, including when their awaiting caller disappears. Completed shutdown
does not clear a later successful spawn from the caller side. Superseded
configuration still discards only its matching pending owner. A real held-IPC
state fixture checks owner identity, one backend during cancellation, immediate
refusal, drain/retry, the returned-timeout failure latch and superseded failure
without a timing or successful-query performance claim.
Existing public method signatures and dependencies remain unchanged; the drain
probe is an additive API. Successful native startup/shutdown replay is still
required before adoption.

Cancellation must preserve deferred disk invalidation. A drop guard restores
the dirty bit if a queued flush is cancelled or fails; a completed flush never
clears a later watcher mark. A regression exercises cancellation of an actual
queued bridge invalidation rather than constructing only that guard.

Yielding exposes multi-step source interleaving, so transaction authority is a
required source correction. One async native scope is acquired inside each
cancellable typed handler before its IDE context, held across open/query/map,
and also used for diagnostic collection and native file-rename closes. Nested
IDE helpers never reacquire it. Syntax-only requests, cancellation, shutdown and
exit remain outside. Diagnostic locks precede the native scope; file-rename
releases its native scope before publishing diagnostics. didChange applies
owned editor text and virtual-cache updates before awaiting diagnostic work.
Source-set mutation stamps include completion boundaries and refuse partial
sets, changes and ABA reopen even when client versions/text are reused; ignored
older changes and absent closes retain the complete result.

Independent source review found a convergence blocker in the first proposal:
rejecting a collected/publishing batch after unrelated document B changes can
lose A's work because the initial worker already removed its ready job. Keep
the whole stale-result fence and return current A work to the existing 64-URI
coalesced scheduler with its existing grace. Both collection and publication
rejection paths retain the current version; a borrowed attempt also retains
work if collection/publication is cancelled. The active worker can insert into
its shared pending queue without owning a channel sender. Foreground retry
wakes the same lane; unavailable workers, closed documents and disabled type
diagnostics do not claim queued work. Existing/newer initial jobs keep their
feedback and deadline, retries do not add another parser-only stage, and a
complete current publication retires the job. Source tests use actual
LspService/ClientSocket notifications, the original project, cancelled scoped
collection, stale publication with a newer version, and a held-IPC worker
rejecting an unrelated change; complete resulting notifications are compared
with a current idle collection. Closed/disabled inverse controls are retained.
All are prepared but unexecuted; ec18 alone is superseded and ineligible.
All three held-backend fixtures record their PID and use `exec cat`, so cleanup
targets the actual sole stdin/IPC owner rather than relying on shell child
optimization. No production output or complete RPC assertion is weakened.

A project/config/watcher/backend epoch accompanies that source stamp. Stale
native requests return whole ContentModified, never filtered or clamped edits;
diagnostic batches keep their own stamp through their separate publication
boundary. Config/root replacement retires the cached bridge without waiting for
IPC. Pending startup cannot publish an old configured session or record its
failure into a replacement generation. The private rename authority lane
(#8009/#8010) still owns exact retained source/range/library/writable ownership;
these transaction guards do not substitute for those laws.

The corpus includes six worker reply/cancellation/deadline controls, a real
queued invalidation, whole dispatcher response controls for root/dependency
changes, same-version reopen, config/disk/root changes and unchanged inputs.
The source CLI fixture uses the unchanged LspProcess harness and original
project. It compares complete symbols/folds with an idle control, checks exact
string/numeric cancellation and shutdown envelopes, unknown IDs, queued typed
cancellation, uncancelled shutdown and direct exit with stdin open. Changed
root/dependency inverses retire a deliberately non-answering backend and check
whole ContentModified on actual stdio; they prove failure/refusal, not a
successful native-query replay. Existing full RPC, versioned diagnostics,
editor, churn, native backend and deadline fixtures remain required.

The first source Actions for Draft [PR #8022](https://github.com/ubugeeei-prod/vize/pull/8022) at e6670dc failed before test execution: fixture helpers need their exact local Clippy expectations, unused expectations must be removed, and the observational LSP migration inventory must be regenerated after the source moves. The correction is restricted to those test attributes and inventory plus this paired record; production source, original fixtures and every complete RPC assertion remain unchanged. The automatic native typechecker phase passed, but it does not replace execution of the new lifecycle/diagnostic controls or protected native suites. Fresh source Actions remain required.

Current-source Check 37283003963 built the archive and passed Clippy, doctests, native navigation and all tooling, but the four runtime shards ran 16,085 tests with seven failures. Six are new controls: the held fixture redirected away its last backend stdout descriptor, causing immediate EOF instead of a held handshake, and the request helper sent unsupported `shutdown` `params: null`. The fixture now preserves the original stdout pipe on fd3 before redirecting, using `exec cat 3>&1 > /dev/null` with the same sole recorded PID; parameterless requests omit `params`, preserving complete legacy response/error expectations. The seventh failure came only from adding the raw original App.vue to the global L3 file sweep. A move-only commit stores those identical 172 bytes as App.vue.txt; all actual RPC workspaces still load them as src/App.vue, and the existing L3 baseline is untouched. Production source and all 25 controls remain intact. Nineteen controls passed, but the premature-EOF fixture invalidates complete held-IPC acceptance until the corrected original lifecycle/diagnostic controls run. Fresh source Actions, full native gates, actual merge and release are pending.

The corrected exact 19e23cdf source passed current Check [37285162482](https://github.com/ubugeeei-prod/vize/actions/runs/37285162482): all 25 new lifecycle/source/diagnostic controls and all 16,085 affected-source Rust tests passed, along with Clippy, doctests, native navigation, tooling and the automatic native typechecker phase. The complete four raw shard logs authenticate those controls; this does not transfer protected full-profile or release credit. A genuine merge with current 40e80ded-main preserves all 36 noncanonical source/control file bytes from that qualified head, every incoming main clause and the exact private additive decision suffix within 350 lines. Incoming default-prop diagnostics and current workflows remain intact. Fresh exact-union Actions, existing protected native/full instruction gates, actual merge and external release evidence remain required.

Rustfmt, source byte integrity, diff and unchanged 350-line policy checks are
the only local qualification. Rust tests and the source CLI were unbuilt/unrun at the source-only peer
checkpoint. Independent review cleared b25d89f; the genuine main replay starts
from d8c3f466 and preserves all incoming module-link registration and canonical
clauses, with 34 peer files byte-identical before this status update. Runtime
qualification remains pending. TODO: independent source/rename composition
review, exact-source Actions for the whole existing
native LSP corpus and new fixtures, unchanged instruction ceilings, protected
full suites, actual merge and external release verification. Source Actions, merge admission and external
release evidence remain required. The four-task/100-message transport
bounds still permit saturated-queue backpressure; a fifth syntax request behind
four outstanding typed RPCs is not fixed by worker yielding. Scoped wait itself
is cancellable but adds no new whole-RPC deadline; the configured bound remains
per worker job. An entered IPC is drained rather than CPU-force-cancelled.
No 10x, RSS, editor-speed, release inclusion or completed-issue claim is made.

The later full-release Check at b899eaf9, run [37292039582](https://github.com/ubugeeei-prod/vize/actions/runs/37292039582), exposed a genuine diagnostic publication regression: Misskey's unchanged whole churn oracle saw three complete empty version-87 publications after rapid root edits 84–87, with the dependency still at 81. All 249 normalized PublishRecord rows and original budgets are retained in the added corpus. The source proof identifies an old caller adopting the latest root version after waiting for the native scope; the retained normalized stream has no actor IDs or stamps, so it cannot attribute individual duplicate rows to edit versus retry owners. The aggregate remained failed despite the original 500-file cold/warm controls passing at 5,484/3,300/3,283 ms. No oracle, cap, expected vector or performance threshold is relaxed.

Bind the scheduled root version after acquiring that scope, then give the complete publication one per-document owner identified by the full source/environment stamp, root version and diagnostic vector. The existing lock entry retains one complete payload, released on document close/rename, and holds no document/native/map guard across transport waits. A changed dependency/configuration/source world can publish again at the unchanged root version; a changed full vector also remains eligible. Equivalent already-taken retry/foreground results coalesce, while stale or cancelled undispatched work keeps the existing bounded/coalesced retry law.

The pinned tower-lsp 0.20 / futures-channel 0.3 transport matters: Client clones a sender per notification, whose first poll uses its reserved slot to enqueue. Its send future can then remain Pending while flushing that already-owned message. Commit the publication at the first actual notification poll, not at flush completion: cancelling a flush cannot retract its queued message and must not requeue a second copy. The pending claim rolls back by Arc identity before dispatch; an older rollback cannot remove a newer claim. Validate source/version inside the short publication mutex before comparison or replacement, then validate again for both new and duplicate outcomes; an old caller cannot overwrite or retire newer-world ownership. Explicit didSave keeps each legacy complete notification through a saved cause, without fabricated environment invalidation. A whole-envelope control fills the original channel, verifies the diagnostic lock is free during backpressure, drops the publisher and checks exactly one complete App notification alongside the original filler and unchanged one-shot unavailable warning. A coalesced current result still completes queued initial work and publishes that notice, because cancelling the original flush may have prevented its post-send notice path. A dependency-source control requires a new complete publication at the same root version even when the deliberately missing backend leaves the fallback vector unchanged; actual changed-native-vector proof remains the unchanged Misskey/full Vue gate. All four existing complete original notification/convergence controls and the held worker controls stay intact.

The added six controls and retained original source/stream are unbuilt and unexecuted locally. Fresh current-head Actions, the unchanged full churn/native/RPC/configuration suites and original 500-file controls, protected merge, and external release evidence remain mandatory. No elapsed-time, 10x, RSS, release inclusion or completed runtime claim is derived from this source-only correction.

### Packaged editor rename refusal evidence

The actual `bc81bf4ec3c27716b9cd56afe01338cc77a2ffc1` full Check run
`37300057411`, editor job `111730397813`, rejected the unchanged scorecard
rename with `Content modified` after the complete hover, quick fix,
format-on-save and semantic-token assertions passed. Its raw log records a
209 ms rename attempt but no source/environment stamp or watched-file
payload. The fixture/profile cleanup removed the existing server log; the
cause is therefore unqualified. The genuine original 500-file and complete
Misskey controls passed at this head independently of the failed editor gate.

The same existing PR adds opt-in `VIZE_LSP_TRACE_NATIVE_SCOPE=1` evidence to
the real editor-host step: complete incoming watched-file events,
environment-change caller provenance, and captured/current stamps only when
a native result is already refused. Successful native requests perform no
additional trace lookup, source scan, backend query, epoch update, or await.
Failure-only retention copies the server's existing `lsp.log` and an explicit
allowlist of whole controlled fixture files before cleanup, with SHA-256 and
missing-file custody. These are disk bytes; unsaved editor buffers are not
misrepresented as disk content. A bounded artifact retains those files,
including the hidden log directory. Retention errors preserve the original
test failure.

No actor is presumed, no rename retry or wait is introduced, and every
original assertion, stale-result refusal, cancellation/lifecycle law and cap
remains mandatory. A fresh whole editor-host/full/native run must establish
the actual cause before a production behavior correction is selected.

### Deterministic contract-hover fixture topology

The genuine `44030abdb59f42bfb4a310d36a106f48cf9cf4a4` full Check
`37305562676`, editor job `111748289882`, retained artifact `11343840939`
shows the recorded invalidation path: at 11:53:58.304272 UTC the client sent
a batch containing two Created events each for ContractChild.vue and
ContractHost.vue; `mark_corsa_disk_state_dirty` moved environment 16 to 18.
The refused result at 11:53:58.333057 retained document stamp 108 unchanged
and observed the newer environment. This records the actual invalidating
notification, without claiming that disk content or topology was equivalent
at every earlier backend observation. The conservative whole-result refusal
is preserved. All original 500-file and complete Misskey controls passed at
this head; the packaged editor gate failed.

Contract hover exercises an existing imported component, so the same two
complete authored files are now copied from the retained immutable corpus
before the packaged client starts. The hover step verifies both entire disk
inputs against the unchanged original expected-source constants instead of
creating new project members while the scorecard is running. All three
complete hover responses, child definition, typed-ref broken-to-unsaved-repair
sequence, quick fix, format-on-save, semantic tokens and one-shot rename
assertions remain. A focused preparation/launcher control checks full bytes
and SHA-256 at client launch while preserving the unrelated scorecard input.
Its recorded launcher proves preparation order, not native/editor behavior.

No watcher events are filtered, delayed or reclassified, no source/environment
stamp or backend lifecycle changes, and no retry, sleep or budget waiver is
added. Existing create/delete/closed-Vue/declaration/configuration invalidation
controls and all six original native stale-result/cancellation laws remain
byte-exact and mandatory. Fresh whole source/native/editor/full Actions and
actual protected parent delivery remain required; this source correction alone
does not qualify a release or performance improvement.
