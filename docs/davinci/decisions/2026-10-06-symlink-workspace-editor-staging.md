# Session-owned staging for linked workspace SFCs

Paired decision: [#7990](https://github.com/ubugeeei-prod/vize/issues/7990#issuecomment-6008933960).

The report preserves a six-file workspace with `app/node_modules/ui -> ../../ui`,
the original default tsconfig and no Vize config. Its minimal stdio client opens
`Other.vue` and then `App.vue` without waiting between notifications, waits
fifteen seconds and sends shutdown/exit while stdin remains open. The reported
0.432.0 npm build leaves two generated UiButton companions in the real package
in three runs; App-only and a src-only include are reported negative controls.
The complete report and all six original authored files are retained under
`tests/_fixtures/differential/lsp/symlink-workspace-staging-original`.

## Concrete current-source path

On signed source `143c1d4a9fbe6530cdd06f1f001b816f39c1d00a`, the first host can
materialize a private mirror containing an external link to an untouched
package. A later host in the same complete editor overlay revision registers
owned package shadows. The materializer calculates dependency links from the
complete current file union, then reinserts the first host's cached external
link, allowing it to conflict with those newly owned private paths.

Existing ancestry checks already refuse unknown links. A fresh VirtualProject
also lacks the previous successful editor union's exact recorded link targets,
so it cannot safely retire even its own prior link. This is a source-grounded
plan/ownership defect; the original leak has not yet been reproduced on this
current revision. Neither the reported historical failure nor a hypothetical
current refusal is recorded as executed current evidence.

## Narrow repair

Restore only the existing completed editor snapshot's package-link identities
into the freshly constructed mirror. Existing root-first validation still
requires the actual on-disk link target to match that retained identity before
unlinking a session cache entry. A new or retargeted link remains an error.

A preserved external package link yields to the current complete union when
owned files occur below its path. The current dependency plan remains
authoritative, including its existing entry-by-entry package dependency lookup.
Owned internal workspace aliases retain the existing manifest, target and
overlap checks. Source catalog, import resolution, generated code, request
paths, native backend selection and snapshot/lifetime rules remain unchanged.
Cleanup removes only the session owner's private cache. There is no deletion
or overwrite of authored companions, and no source-tree cleanup fallback.

## Prepared qualification and limits

Three Canon filesystem laws prepare the exact same-revision open order,
private companion/request paths, original full authored byte conservation,
existing authored `.vue.ts`/`.d.vue.ts` sentinels and a retargeted-cache-link
refusal. They check conservation before and after the actual session clear.

The direct current-source CLI stdio test prepares three original fifteen-second
runs, App-only/broad-include/src-include/reversed-order/CRLF/authored-companion
controls and whole versioned diagnostic envelopes. It retains the original
empty client capabilities, default configuration and null server-request
responses. It uses the existing default stdio `lsp` transport rather than the
reporter's npm wrapper with explicit `--stdio`, and captures stderr for failure
diagnosis. An added unsaved plain-TS TS2322/repair pair prevents an unavailable
native backend from satisfying the empty Vue expectations vacuously.

The test provisions actual already-installed TypeScript 7.0.2 and Playground
Vue package links outside authored source, without creating a Vize config or
changing initialization flags. Playground Vue is the current source-test
dependency, distinct from the reported 3.5.43 and 3.6.0-rc.10 environments.
All diagnostic arrays are proposed complete source assertions, not observed
successful output. Only Rust syntax/format, original input custody and existing
cheap source inventory checks may be run locally at this private stage. The
local Vite+ format command could not start without a project-local installation;
no dependency installation or native build was attempted.

TODO: independently review this cohesive private source, qualify the new
filesystem and genuine default/native stdio laws on exact-head Actions, replay
on current signed main, then require the existing protected full suites and
unchanged 100+4 instruction ceilings before actual merge. Public installed
replay and issue closure remain separate. No native migration, performance,
fix-history closure, published payload or current-runtime success is claimed.

## Default logger custody from the first real run

Paired decision: [6009236264](https://github.com/ubugeeei-prod/vize/issues/7990#issuecomment-6009236264).

Current47a Check37412151174/Rust4 job112104158982 reached the original queued
Vue publications, then failed the newly authored whole-filesystem comparison.
All14 previous entries, complete file bytes and link targets were identical;
only `app/node_modules/.vize` and its regular `lsp.log` were created. The full
raw SHA2c5a30c3 and logger10,391B/SHA0444784a are retained. No authored UiButton
companions appeared in this first attempt. The failure occurs before the
plainTS probe and repeats2/3, so it qualifies neither all9 sessions nor native
availability. Historical leak-versus-current-refusal remains unknown.

Unchanged `vize_maestro::serve` initializes this exact default append logger.
The observer now requires that exact cache directory and regular UTF8 log
with the source startup marker, removes exactly these two new generated
entries from the comparison, and checks every other complete path/directory,
file byte and symlink target against the original snapshot. It does not exclude
a directory subtree or arbitrary generated file. Default config/transport,
all6 original398B files, all3 filesystem/all9 stdio assertions, nativeTS2322
repair, production code and waits/retries remain unchanged. Fresh exact-head
execution and actual security-main/protected/public qualification remain
required; the initial failed head receives no all-session runtime credit.

## Exact original protocol and full logger record

Paired decision: [6009461873](https://github.com/ubugeeei-prod/vize/issues/7990#issuecomment-6009461873).

Current4ae Rust3/4 reaches whole Vue publications and native TS2322/repair,
then fails the newly authored shutdown-success assertion. The original driver
awaits any reply to literal `params: null` and never asserts success. Pinned
tower-lsp0.20 `FromParams<()>` rejects a present parameter, producing exactly
-32602 `Unexpected params: null`. Keep those complete error envelopes and
original null shutdown/exit in the three original fifteen-second sessions;
the six authored controls independently retain valid omitted-params shutdown
with complete result:null success. Production dispatch is unchanged. Current
raws SHA6f08eab4/a2cbcffb remain failed historical observations, not all9 proof.

Current tooling4 SHAfff39adb rejects the observer's partial logger contains
assertion. Require the exact first stable level/target/message after its
variable timestamp, preserving the regular UTF8 file and exact two cache
entries. Every other full filesystem entry, original input, diagnostic array,
wait, open order and production byte stays exact. No allowlist/subtree filter,
protocol fallback or runtime credit is introduced; fresh complete Actions,
actual security-main replay, protected gates and installed acceptance remain.
