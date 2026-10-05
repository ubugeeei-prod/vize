# Same-server module-link context design (2026-10-04)

Tracking: [#6871](https://github.com/ubugeeei-prod/vize/issues/6871),
[#6872](https://github.com/ubugeeei-prod/vize/issues/6872) and
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883).
The [paired decisions](./2026-10-04-module-link-context-issue-draft.md) retain
the original preparation text and actual publication links. This private prerequisite design refines the reviewed private
DocumentLinks proposal `c5044a081a4f65d0b2db0709e08f136d16abb4f5` without
changing that commit or its worktree. Frozen design `8074694ee89a3c8c6f36928a64509aa3a7266de5`
received root and independent peer clearance after the original EOF correction.
Root then authorized private source/law preparation on fresh main. Full root
and independent peer source review cleared `2a4ccc626e398f305771b20da9b9cc6d330ffa93`;
source publication and hosted execution are authorized. Exact ad670 source and
preserved ee22 protected acceptance pass; actual merge remains release-held.

## Current source and boundary

Historical design worktree baseline: literal main
`b9be9065b872086726158b67d1b4e7b8dee78a14`. The original source
reviewed implementation used literal main `da894691fff4da81b4acbe71a935c7c67fb0539a`.
The historical replay used literal main
`9702c014e06a268d2651202b7fcb951db7e729df`; the current isolated genuine union uses
actual main `76f93fe1e3c46ac8c86157a520302e26c50ac871`. All old worktrees remain preserved.
Actual #7737's Names configuration tickets and move-only `navigation/cache.rs`
are present. Preserve those exact owners, routes, locks and lifecycle. Their
parser/linked identities are not module context or project-resolution proof.

This slice supplies only a private same-server **applied project context** and
its guarded lifetime through SourceQueryProject publication. It reads actual
host configuration; it does not load a module, read/canonicalize a target,
register a watcher, choose a source profile, admit a File, parse an AST or return
DocumentLinks. Target physical observation/event coverage is the next genuine
provider; the dependent consumer remains unimplemented.

| Actual callsite under `crates/vize_maestro/src/`                     | Observed behavior and required hook                                                                                                                                                                     |
| -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `server/state.rs::ServerState::new`                                  | Installs actual checker/default timeout 60,000 and native-only empty workspace root; mint an independent module session here.                                                                           |
| `server/state/config.rs::apply_type_checker_config`                  | Sole production checker-tuple write; both real config loaders call it. Gate this assignment together with actual load origin and generation.                                                            |
| `server/state/config.rs::{load_workspace_config,load_lsp_config}`    | Evaluate `load_lsp_config_snapshot` outside locks; apply only when its actual source_path is Some. Pass that original PathBuf, not a displayed-string reconstruction.                                   |
| `server/state/workspace_folders.rs::set_workspace_root`              | Sole production primary-root write, cfg(native). Gate the assignment; leave all existing cache invalidation outside.                                                                                    |
| `server/handlers.rs::initialize`                                     | Loads configuration, folder contexts and options, then sets the actual primary root synchronously; no context capture may treat folder lint contexts as a module project.                               |
| `server/handlers.rs::did_change_configuration`                       | Logs the editor-restart policy and does not apply settings. Do not invent a reload or advance generation for this no-op.                                                                                |
| `server/workspace_folder_events.rs` and `state/workspace_folders.rs` | Update per-folder lint contexts, not the primary root/checker. Preserve behavior; this slice does not promise membership or per-folder module selection.                                                |
| `server/handlers.rs::shutdown`                                       | Currently returns Ok without retirement; synchronously retire module admission before returning, with no global feature shutdown.                                                                       |
| `server.rs::MaestroServer::new`                                      | Foreground and diagnostic worker share one ServerState. Only foreground owns a session-termination lease; the background Self must have None.                                                           |
| `lib.rs::serve_transport`                                            | Both stdio and TCP use the same transport/exit select. Observe actual input EOF/terminal read failure immediately; outer and foreground leases cover completion/drop without changing the legacy drain. |
| `source_project/project/host.rs` and `project/tracked.rs`            | Actual Server host and final source publication seam. Capture/validate only against this same retained host. Bare/shared stores have no context.                                                        |
| `source_project/navigation/cache.rs`                                 | Existing Program workers depend on snapshot/profile, not module settings. Reuse them; a context change alone does not force a new Program parse.                                                        |

Read-only source search finds one checker write and one primary-root write in
production. Future implementation must recheck this inventory on its actual
base; direct writes bypassing the new gate are forbidden for these fields.

## Minimal state and sealed context

Propose one `cfg(experimental-source-navigation)` module gate on ServerState,
separate from Names and worker maps. It owns a private retained session identity,
a checked u64 generation, phase Live/Updating/Retired and optional actual
checker-load origin. Keep the original root and checker tuple as the sole
resident values; do not add another root/config mirror or project cache.

A sealed, non-forgeable ModuleLinkContext retains this session identity,
generation and owned values read under the gate: actual primary root, actual
TypeCheckerConfig/timeout tuple, actual optional load origin and the corresponding
ProjectModel selection. No public constructor, caller-supplied root/defaults,
serialized ticket or copied Names/Corsa identity grants authority. Owned plain
metadata may cross an await; no native arena, Program or File enters this state.

Outstanding contexts retain the session identity allocation. Comparing that
identity and generation, not equal settings or just the server address, rejects
a context from another ServerState even when both have identical values and
generation zero. A new server gets its own identity; a retired one never resets.

Capture under module read guard, then read actual root/checker using internal
already-gated helpers. Release all guards before source-cache access, mailbox
work, filesystem work or await. The factory is unavailable for non-Server hosts,
a missing root, an Updating/Retired session or a minimal build without native.
`experimental-source-navigation` does not imply native; no unguarded native-only
getter and no cwd/filename/DocumentStore default may substitute for that root.

Actual installed defaults may be observed only from this real host after its
real root is configured. That differs from constructing default options to
pretend a bare store has a project. Source language/profile/URI eligibility
belongs to the subsequent consumer; this global context is not a syntax ticket.

Retain config-load origin separately from effective selection. The existing
fresh Corsa configuration formula is
`ProjectModel::new(actual_root, None, &actual_checker)`. Use the same formula
for metadata; do not pass load origin as its config_source or silently retarget
relative tsconfig paths. This proves current applied selection, not a cached
Corsa bridge's running configuration, a tsconfig parser or module resolution.
The original loader may apply fallback defaults with a source_path; provenance
records that actual load/apply event and does not certify file contents as valid.

## Indivisible applied mutation

The two relevant writers are synchronous. Keep the module **write guard held**
for the complete begin→apply→finish operation, including actual authoritative
field assignments. Reserving/advancing an epoch and releasing the gate before
assignments is forbidden: it could expose old values under a new ticket.

Prepare loaded values and the owned origin path outside the gate. Inside it,
enter Updating with a private RAII mutation guard; check the next generation,
perform only the original checker/root assignments and origin update, then
commit the generation and Live phase. Capture cannot enter during this critical
section. On unwind before commit, the RAII guard permanently marks module
admission Retired before releasing the gate; parking_lot does not poison locks.
No observer may capture partially applied state after a panic.

Every real relevant applied mutation advances generation, even an equal-value
reload or root assignment. A→B→A therefore invalidates the original A ticket.
A load with source_path None performs no original checker assignment and keeps
its actual previous origin/generation. Do not impose new legacy loader/error
semantics. Changes only to formatting, Names, lint contexts or Corsa process
lifetime do not modify these context fields and need not invalidate this token.

Use checked_add; on u64 exhaustion permanently retire module admission instead
of wrapping or minting a new session. Existing legacy assignments, invalidation
and logging must still execute normally when module admission is already
retired or overflows. The module gate protects their relevant assignment even
then; retirement is not permission to skip legacy behavior or its guards.

No config evaluation, filesystem call, logging, cache/map access, notification,
waker call, Names lock or await runs under the module guard. The config helper
must release it before original component/overlay invalidation; the root helper
must release it before original package/global/batch/overlay invalidation.
Do not wrap an outer loader and then recursively reacquire the same module
gate in its inner setter. Names application remains its own existing operation.

## Actual termination ownership

Retirement is synchronous, idempotent and sticky. Shutdown marks the module
session retired under its gate before the handler returns. It does not depend
on Arc<ServerState> dropping, since queries and diagnostics can keep that Arc
alive. No asynchronous join, observer list or broad cancellation of unrelated
definition/Names queries is needed by this context-only slice.

Use a private nonClone termination lease holding Weak<ServerState>, not a
strong cycle or a configuration read guard. The foreground MaestroServer owns
one; its diagnostic-worker Self explicitly owns none. Do not retire from every
MaestroServer Drop: background spawn failure/exit must not close the foreground
session. Dropping a foreground lease upgrades the real Weak owner if present
and retires only this module session.

`serve_transport` obtains an idempotent termination lease from the actual
foreground service before moving it into the transport. The outer local lease
covers either select branch's completion, future cancellation and unwinding,
even if a backend Arc remains live. It does not by itself observe input EOF.
Pinned tower-lsp 0.20.0 `transport.rs` disconnects input and then joins output,
input and process_server_tasks; buffered/in-flight handlers drain before the
transport future returns. Retiring only at that return could publish an old
context during the drain. This was the one concrete frozen 3f2286 review correction.

Add a thin observer over the actual AsyncRead passed to this transport, bound
to the same foreground session. Delegate reads without a module guard; on a
nonempty-buffer poll_read returning Ok(0), or a terminal underlying read error,
synchronously retire that session before returning the unchanged read result.
Pending and empty-buffer reads do not mean EOF. Preserve buffer/vector read
semantics and the original failure classification. Codec errors above the
reader do not directly retire the module session; normal transport completion
still retires it. Pinned async-codec-lite 0.0.2 stops its stream after a codec
error, so live admission during that existing drain does not prove later malformed
JSON requests recover or continue. No wrapper
may fabricate bytes, swallow errors, close unrelated requests or skip legacy
draining. The original 3f2286 lease proposal remains preserved as predecessor.

The outer and foreground leases remain idempotent completion/drop fallbacks;
the read observer supplies the earlier actual EOF/failure point. These own
only metadata across await, never a read/config guard. The background object
creates neither a foreground nor transport lease. Shutdown, observed input
termination, completion and object drop may repeat retirement without revival.

Do not infer transport termination from a Corsa backend restart, diagnostic
worker exit, SourceQueryProject drop or the last old ticket dropping. A standalone
SourceQueryProject's actual ServerState lifetime remains its host lifetime;
it cannot mint a session for another host or reactivate a retired one.

## Fixed source/context publication order

Add private SourceQueryProject host capture and a ProjectQueryResult publication
method that uses **that result's own retained DocumentHost**. Do not accept a
separately supplied ServerState or publish by checking some other project's
context. Non-Server hosts return typed unavailable before any native work.

Reuse the existing SourceQueryResult::publish unchanged for its actual Document
read guard and cancellation/revision/version/language checks. Inside its ready
callback, validate session identity, Live phase and generation under the same
actual host's module read guard, then publish only owned synchronous data.
Keep that guard until the callback ends; a writer either commits beforehand and
causes refusal, or waits until the current publication has completed.

Fixed order: DocumentStore read guard → module read guard → ready publication.
Capture holds module → root/checker field reads only. Mutation holds module
write → relevant field write only, then releases before every other effect.
No module-held path may acquire DocumentStore, SourceProject lifecycle, worker
map or Names gate; no publication callback may reenter any store, acquire a
worker map, retire/wake a worker, run user code or suspend. Existing map→source
and map→Names admission stays untouched because module capture finishes before
map access. No field-held path may reenter the module gate.

Preserve the original query/AbortHandle lease across async work and final
publication. Dropping/canceling a source query keeps its existing behavior.
Context invalidation need not cancel unrelated source requests: every dependent
response checks its own authentic token at publication. Superseded/retired/foreign
contexts produce a typed failure and discard ready data, never success [] or
fallback. This guards source plus applied context only; a target ticket must
later join this same publication point before DocumentLinks can be emitted.

No promise about arbitrary future filesystem atomicity is required here. This
provider does not claim path existence, canonical containment, target freshness,
watcher coverage or current on-disk configuration bytes. The next slice must
define and prove a bounded point-in-time physical target contract separately.

## Planned source slice and evidence

The original implementation hold was satisfied by focused design review;
private source preparation was authorized, and complete source review now
permits automatic Actions on the independent draft PR.
Proposed narrow ownership is new `server/state/module_links.rs` and
`source_project/project/host/modules.rs`, tiny state registration, the two real
mutator hooks, result publication helper and foreground/shutdown/transport
lease plumbing. No level crate, Names provider/cache, parser/Program worker,
standard handler or filesystem resolver changes belong to this prerequisite.

Planned hosted laws must use actual ServerState/config loading/root setters and
SourceQueryProject/new_server, not fabricated setting snapshots: complete values
from nondefault root/tsconfig/runtime/timeout and load origin; missing root,
minimal/no-native and bare-store refusal; equal-value reload and root/checker
A→B→A; same-values foreign server; actual private counter-at-max mutation with
sticky overflow; unwind mid-assignment; retirement before ready publication;
source change/close/reopen/cancel and config writer racing its real document
guard. A context change must not create another original Program parse.

Actual service/transport controls must preserve entire initialization/shutdown/
exit/EOF response behavior, retirement while an old state Arc is retained,
foreground drop and background spawn failure/drop without false retirement.
Hold a genuine ready/in-flight source/context query and its actual state Arc,
observe real input EOF before the handler drain completes, then require its
guarded publication to refuse. Retain the original unrelated drain behavior.
Terminal read failure must receive the same complete test; Pending and empty
read buffers preserve the live session. Codec-error-without-read-EOF keeps
admission live during the existing drain, then completion retires it. An outer
future-return assertion alone does not prove EOF-time retirement.
Author full expected values and complete envelopes before execution; never
recapture expectations from observed native output. All cases are planned here,
not executed or credited. Preserve existing Names controls, historical sessions,
minimal/strict checks and all instruction ceilings/ratchets. Use existing
source-built native-navigation Actions and protected full checks after source
authorization, with no local build/install or new manual campaign.

This design supplies the smallest applied-host lifetime prerequisite. Target
observation/events, static module DocumentLinks, resolver semantics, LSP7/history
closure, native equivalence and default replacement remain unfinished. Use a
real native Stack for later unmerged provider→target→consumer slices; no layer
receives queue or completion credit from design review.

## Private source freeze on actual main

Implementation base: literal `da894691fff4da81b4acbe71a935c7c67fb0539a` in a new
`wt` worktree. Both old design commits/worktrees remain intact. Production
inventory still has exactly the same two relevant assignments, now gated inside
`apply_type_checker_config` and `set_workspace_root`. Real loader source_path
reaches the checker helper as a Path, separately from its unchanged display log.
No root/checker mirror, filesystem operation or source reparse was added.
Move-only commits extract the unchanged diagnostic-worker constructor and
ServerState Default implementation; the shutdown handler delegates its exact
result to a small helper so both existing facades satisfy the source ratchet.

Private `ModuleLinkContext` and its constructors remain crate-internal. The new
module Session retains opaque identity/generation/phase/load origin. RAII covers
unwind before commit; retired/overflow sessions still apply legacy settings and
execute original invalidations after releasing the gate. A scope-specific
unused-provider allowance records that target/consumer callsites do not exist
rather than adding a public constructor, fake response or default handler.

The new `ProjectQueryResult::publish_with_module_link_context` keeps its own host
and original cancellation lease, invokes unchanged source publication, then
holds that host's module read guard through ready owned publication. Its error
separates genuine source refusals from context refusals; ready prefixes never
escape either error. It does not expose target authority or a DocumentLinks RPC.

Foreground and transport hold Weak termination leases. The actual diagnostic
worker constructor explicitly owns none; its pre-spawn destruction law covers
the same owned object a failed handoff would destroy, without claiming an
executed OS spawn failure. Shutdown retires before its unchanged Ok response.
The input wrapper delegates original scalar and vectored reads without a gate,
then synchronously retires nonempty EOF/terminal errors. Pending, zero capacity,
Interrupted/WouldBlock controls keep their original results. Codec errors
retain live reader admission during the original drain, followed by transport
completion retirement; no later-request recovery is claimed.
First retirement remains sticky; repeated drop/shutdown cannot revive a session.

Authored laws use real configuration files/loaders/root setters, same-value and
A→B→A updates, equal foreign servers, private max counter, panic mid-assignment,
blocked partial-apply capture and original document/context writer ordering.
Whole source/version/language/close/reopen/cancel refusals use actual projects.
A retained native navigation worker keeps its same original Program and one
parse after root mutation; no Names or source-worker ticket is substituted.

Actual tower-lsp transport laws hold one genuine ready source/context query and
one existing standard DocumentLinks future. EOF/terminal error occurs before
those futures drain; the former refuses and the latter returns its entire
original null result. Separate cancellation and codec-error-without-read-EOF
controls retain full frames, actual source and state Arc. Initialization,
shutdown and exit controls assert complete authored envelopes. Test-only custom
held-query plumbing is not an implemented native link consumer or public RPC.

The existing mandatory native-navigation composite gains a `module_link` law
filter in normal and no-default-features builds; existing source/RPC/minimal/
strict recipes and all instruction caps remain. The paired workflow law records
its exact recipe. This is source preparation only: no Rust build, test execution,
Actions, publication, queue admission or runtime/native/default credit has been
performed by the private freeze. Root and retained peer cleared the complete
source before initial publication on actual da894. The subsequent replay
preserves the reviewed implementation/law blobs and every incoming decision;
only the shared central decision paragraph needed conflict resolution.
Automatic exact-source normal/minimal/strict Actions are the next qualification,
followed only after source acceptance by protected full104 and actual merge.
Target observation/event coverage and the true module-link consumer stay next.

## Publication replay

Draft [#7771](https://github.com/ubugeeei-prod/vize/pull/7771) first published on
actual da894 with paired issue decisions. Main then advanced through #7762,
#7761, #7767 and #7768. A new `wt` replays all genuine commits, retaining the
separate behavior-preserving extraction commits and the old source/worktree.
That historical base was `9702c014e06a268d2651202b7fcb951db7e729df`. The old b7 Actions
reported Rust-builder and tooling failures; their full logs are retained.
Rust E0282 is repaired by an explicit `Vec<Value>` annotation in the unchanged
whole-frame decoder. The tooling boundary now recognizes only the exact
reviewed host `ProjectModel` import at `module_links.rs`, with new foreign-path,
alias, storage, glob and other-call negatives; all old boundary vectors remain.
All other product/law blobs equal the reviewed source. The failed logs grant
no acceptance to the replay. Current-head hosted normal/minimal,
older controls and strict warnings must pass before protected full104/merge.
Physical target observation/events and the true link consumer remain next.

## Genuine minimal-test qualification

Source905 hosted affected Clippy/build, all 152 SourceProject, 43 RPC and 21 normal
module-link laws passed, including actual held EOF/error/drain frames. Its new
no-default test then failed 14 pre-existing feature-owned API references before
the 11 minimal laws could run. Complete original logs remain retained; 905 has no
source acceptance. Two definition laws and one virtual-TS law gain their real
`native` cfg; six formatter laws gain their real `glyph` cfg. Whole old bodies,
inputs, expectations and default execution remain. Only unused feature-owned
imports are gated; no production body changes. Fresh normal/minimal/strict
source and protected full 104/actual merge remain mandatory. Source cd5 then actually passes all 152/43/21 normal and 11 genuine no-default laws plus minimal check, but final strict Clippy rejects two introduced item-level allow attributes on private error reexports. Replace the genuinely unused outer import allowance with a precise conditional expectation, and remove the unnecessary inner allowance; its import is already used by that outer reexport. Every API/body/input/oracle/cap stays exact. Original cd5 logs remain retained, granting no whole source acceptance; fresh strict/current source/protected proof stays required.

Exact ad670 Check 37187035108 and preserved ee22 full Check 37187705938 pass all 39 protected jobs, normal 152/43/21 and genuine no-default 11 laws, minimal check, strict Clippy and all 104 unchanged ceilings measured ×3 with independent raw qualification. The concrete #7778 release owner manually removes the green ee22 entry at 2026-10-04 08:16:37 UTC pending verified public 0.432.0 artifacts; no actual merge is claimed. The genuine actual-main 76f93 union preserves every parent product/law blob and all incoming source/docs/versions, including 912d original typechecker-phase harness required by the future child Cargo.lock path. No helper copying or old candidate proof substitutes for fresh source/current protected acceptance. New true native Stack membership and post-release fresh-main/actual merge remain mandatory. Parent ec68 Check 37189280588 is terminal source SUCCESS. After #7778 literally merges as caee1656927e1232f4aaabff2328246decb8bc58, a separate clean wt preserves all released version/manifest bytes and every parent production/law blob through genuine ancestry; the numeric prose repair changes no qualification. Public artifact verification still gates queue admission; this new source must pass exact-head Actions and current protected acceptance. The genuine release-source90fc and physical2d3 prefix actually merges at 2026-10-04 09:53:06 UTC as signed-valid50c944da9775380afc8c245c02a21765b076788b followed by signed-valid69cab7b3436139c1e4717da0c3aef4844d21141e, both queue entries null. Current parent full Check37192659119/Musea37192658878/Nuxt37192658877 and child full Check37192659855/Musea37192659510/Nuxt37192659548 are terminal SUCCESS, each39 jobs. Parent actual152 SourceProject/43RPC/21normal/11 genuine minimal laws, minimal check/strict Clippy and all four complete Rust archive workers pass; both current raw104×3 packets are independently reconciled against complete original inputs/windows/guest/binary and immutable actual-parent caps/ratchets. Paired actual-delivery receipts are [#6871](https://github.com/ubugeeei-prod/vize/issues/6871#issuecomment-5978768579) and [#6883](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-5978768876). Old campaigns remain historical; this accepts only the same-server context prerequisite, with original module DocumentLinks consumption and default/history still separate.
