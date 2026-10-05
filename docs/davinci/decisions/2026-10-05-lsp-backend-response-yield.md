# Keep LSP dispatch active while type requests wait

Issue: [#8012](https://github.com/ubugeeei-prod/vize/issues/8012).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8012#issuecomment-5989126941).

The original report and complete App.vue/useToast.ts are retained under
`tests/_fixtures/lsp-corsa-responsiveness-8012/`, with the reported package and
strict Bundler project shape. The two sources were compared byte-for-byte with
the saved issue body. Reported 1.5–2.6 s / 58 s stalls are reporter observations,
not measurements of this private source proposal.

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

Rustfmt, source byte integrity, diff and unchanged 350-line policy checks are
the only local qualification. Rust tests and the source CLI have not been built
or run. TODO: refresh onto actual main after the admitted release, independent
source/rename composition review, exact-source Actions for the whole existing
native LSP corpus and new fixtures, unchanged instruction ceilings, protected
full suites, actual merge and external release verification. No public PR or
merge admission exists for this source. The four-task/100-message transport
bounds still permit saturated-queue backpressure; a fifth syntax request behind
four outstanding typed RPCs is not fixed by worker yielding. Scoped wait itself
is cancellable but adds no new whole-RPC deadline; the configured bound remains
per worker job. An entered IPC is drained rather than CPU-force-cancelled.
No 10x, RSS, editor-speed, release inclusion or completed-issue claim is made.
