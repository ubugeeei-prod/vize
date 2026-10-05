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
