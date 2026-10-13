# Preserve explicit Nuxt component import order

Issue: [#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

The complete pinned VueFes `app/app.vue` imports `main.css` before its
explicit `NuxtPage` and `NuxtRouteAnnouncer` imports. The native Nuxt bridge
removed that explicit statement and prepended all resolved components,
combining it with the separate template auto-import path. This changes
side-effect dependency order. The actual immutable `399e71b` run remains
failed: its original dev-home screenshot differs by `0.1235105465271594`
against the unchanged `0.002` budget; 37 cases were not run.

The primary Nuxt 4.0.3 component
[TransformPlugin](https://raw.githubusercontent.com/nuxt/nuxt/v4.0.3/packages/nuxt/src/components/plugins/transform.ts)
uses unimport's `injectAtEnd` for explicit imports. Its separate
[LoaderPlugin](https://raw.githubusercontent.com/nuxt/nuxt/v4.0.3/packages/nuxt/src/components/plugins/loader.ts)
prepends implicit template bindings. Before the production edit, the
complete original 840-byte app and its complete official alpha.2 compiled
script were passed through the actual native bridge and unchanged official
TransformPlugin. Native eager component imports preceded `main.css`; the
actual official transform retained the CSS first. Complete inputs, outputs,
static import specifiers and generated local bindings were retained.

Explicit resolved bindings now replace their original authored import
statement. Implicit `resolveComponent` bindings keep their existing prepend
behavior. Existing async/client-only helpers, aliases, leftover type and
unresolved specifiers, and the RouteAnnouncer wrapper exception remain.
This preserves the authored statement position; it does not globally move
all imports or sort CSS. It matches the complete original app's official
static import sequence and local bindings. Unimport can place explicit
bindings after later static imports in other sources; this bounded repair
preserves authored explicit positions rather than adopting a global policy.

A separate move-only commit extracts the existing six complete import
injection blocks into a maintained module. All original block bytes and
comments are retained; the original module re-exports the same public
function. A separate move-only commit extracts the whole missing-SSR-edge
counterexample object before the fixture gains the component bridge. The
350-line growth gate remains unchanged.

The new differential corpus retains the original full app and its source
hash, upstream commit and Git blob identity. Three meaningful regression
tests fail on the entire original `399e71b` producer and pass after repair;
all 26 existing component resolver/injection controls pass. The durable
source oracle executes the unchanged 4,846-byte primary TransformPlugin
with actual Nuxt 4.0.3 runtime entry paths and locked unimport 5.2.0. Its
stable Vue peer is used only by that source transform; the existing alpha.2
compiler, runtime and lock remain the full SFC/SSR/DOM/hydration oracle.

The existing full 1,359-byte VFHeading and 5,959-byte global stylesheet
fixture now passes its entry through the real native and official Nuxt
component bridges. It retains complete bridge input/output modules,
original eight-case SSR HTML, full cold/hydrated DOM, all stylesheet
responses, the missing-SSR-edge counterexample and four unchanged
`18px`/`27px` reverse-order assertions. CSS rules, original app sources,
reference images, viewport and VRT budgets are unchanged.

The historical full app packet and current immutable screenshots show a
98px accumulated heading deficit. The current full app's transformed root
module and eager VFHeading dependency graph were not captured. The import
compatibility defect is proved; its repair alone does not establish the
complete app's cascade repair. The same original 38-case VRT must pass on
the exact composed source before queue readiness or release acceptance.

The dedicated qualifier reuses its one original ci-profile native build. An
opt-in wrapper retains the original NAPI arguments and entrypoint synchronizer,
captures complete Cargo JSON and streams, and admits a distinct ci receipt
only after exact command/profile, source, environment, toolchain, ELF and
local/frozen byte checks. The existing strict debug receipt stays unchanged.
The two exact NAPI-generated `index.js`/`index.d.ts` outputs retain their whole
before/after bytes and complete diff; all other authored source must remain
unchanged. Generated declarations are never normalized back to source.
The actual existing live-owner wrapper binds that same addon and post-build
source diff for supplementary package tests; no second native build is added.
Negative command, profile, producer, untracked declaration, stream/hash, ELF,
path and missing-authority controls run in the original tooling suite.
The original missing-receipt failure remains preserved. Only exact composed
Actions can qualify this genuine ci capture and complete app acceptance.

A local full-source bridge control caught an adjacent transport-state
declaration moved into the counterexample helper. Its failure packet and
log remain retained. Both variables are restored to their original driver
scope before qualification; the corrected official full-SFC/CSS source
control preserves complete SSR and both cold/hydrated `20px`/`34px` results.
This local control is explicitly official-only and gives no native credit.

Local authority controls pass, including the original debug admission
contracts. Broader local Art boundary verification has no fresh `target/ci/vize`
in this isolated worktree, and the existing MoonBit package-build control
fails in its local compiler preparation. Their complete logs remain retained;
neither is counted as passed. Original hosted tooling/native preparation and
the exact source qualifier remain mandatory; no check or environment failure
is replaced with fixture expectations.

The exact `0dd6a7b` qualifier
[38125005499](https://github.com/ubugeeei-prod/vize/actions/runs/38125005499)
is terminal failed. All three original dev-home attempts retain the same
`0.1235105465271594` ratio and byte-identical candidate/reference PNGs;
37 cases remain unrun. Native metadata passed 58 controls, but both existing
public formatter controls failed without a build receipt. The complete
app import and SSR/DOM/hydration differentials did not execute after the
Nuxt oracle's actual `npm ci` failure. Their absence gives no native credit.
The current full app still has no captured root-module/CSSOM authority.
The source bridge corpus covers the original script and official compiled
script, not the complete implicit-template dependency graph.

The one native ci build really ran on that runner, but Vite Task removed the
custom capture flag from its cached script environment. A separate actual
installed Task control reproduced the absent flag while retaining
`GITHUB_ACTIONS` and `GITHUB_RUN_ID`. The qualifier now writes a source-bound
capture request before the same build. Both request creation and reading
require the actual Linux Actions context, matching run ID and committed
source. A malformed or stale request fails before Cargo. This request records
intent only; actual Cargo completion, profile, raw streams and emitted/local/
frozen ELF custody remain necessary for the distinct ci receipt. The strict
debug admission remains unchanged, and no second native build is introduced.

The isolated Nuxt oracle failed with the actual missing `commander@11.1.0`
lock entry. The actual npm 11.9.0 lock generator moved that identical entry
from the root to `node_modules/svgo/node_modules/commander`; all other parsed
lock data, dependency versions and integrity hashes remain unchanged.
Fresh `npm ci` fails before this generator correction and installs all 676
packages afterward. The original alpha.2 compiler/runtime lock is unchanged.
The typed-lint failure in the stream-custody control is corrected with named
records, preserving both full stream checks. Fifteen admission controls and
the complete original-app source bridge pass locally. These repairs qualify
preparation only; actual full-app cascade repair still requires source proof
and the unchanged original 38-case Actions verdict.

A bounded before/after control uses the complete frozen `0dd6a7b` wrapper
and corrected wrapper with the flag absent and an invalid request present.
The old wrapper silently invokes both controlled process callbacks; the
corrected wrapper rejects admission before either callback. No native process
executes in this control. Its regression belongs to the original tooling
suite and prevents a file-based request from falling back to ordinary mode.

The missing current-app cascade authority is captured only after the original
dev-home PNG matcher fails, before its original context teardown. Passive
listeners retain original document/script/CSS responses, duplicate headers,
decoded body bytes and explicit redirect, failed or pending states. Ordered
CSSOM includes imports, layers, media, adopted sheets, all heading computed
styles and their ancestor context. Its timestamps describe later state,
not atomic screenshot-time authority. No request is intercepted or fetched.

The sidecar keeps the original error and all 38 cases, screenshots, budgets
and preparation intact. A five-second diagnostic guard and 30-second reserve
use an explicitly approximate callback start, excluding framework fixture
setup; no test timeout is extended. Streamed retry-specific packets remain
incomplete on refused, missing or late evidence. Late promises cannot finalize
their partial files. This source correction supports one bounded existing
Actions discovery run; an expected red is no readiness or acceptance credit.

The exact `f5824ae` discovery run
[38130325058](https://github.com/ubugeeei-prod/vize/actions/runs/38130325058)
did not reach VRT. Its complete retained process packet proves that the
original NAPI command and entrypoint synchronizer both exited zero; all 478
Cargo JSON records end with `build-finished: true`. The distinct ci receipt
was admitted on the actual source. The outer Vite Task then failed with
`EAGAIN`; the pinned-source and local pressure control support an inference
that its stdout forwarding failed, without a hosted Linux syscall trace. Its later contract
step hung after the original browser installation was skipped; no completed
Linux browser-control packet was retained. The run was cancelled. That
cancelled run, the earlier failed row and the unexecuted VRT remain separate
outcomes.

Pinned Vite Task maps output-pipe errors to its `Failed to spawn process`
message; cache-update failures have different handling. A bounded local
replay of the entire retained Cargo streams through installed Vite Plus
0.1.24 succeeds with an active consumer, but fails with actual pipe pressure
after a producer-completed marker. The task summary records macOS `EAGAIN`.
This proves the local output path, not an absent Linux syscall trace.

The captured process attempt and complete raw streams are now persisted
before display. Console output reports stdout's retained path, length and
hash rather than replaying Cargo artifact and build-script metadata. The
complete records remain in the raw binary files and process packet;
diagnostics, unknown JSON, non-JSON stdout, final build result and full
stderr remain visible. Synchronizer stdout is unchanged. The same local
pressure replay passes with full retained stdout and byte-identical stderr.
Native command, exit handling, cache policy, source admission, ci/debug
receipts and the original 38-case workload stay unchanged.

The original upload omitted the emitted/local/frozen ELF bytes, retaining
only their admitted hashes. Its producer-time receipt is not retrospective
whole-binary custody. The existing evidence upload now includes all three
actual binary paths without another native build. The complete generated
declaration diff remains recorded; the strict debug source gate is unchanged.
The generated declaration bytes equal the earlier `0dd6a7b` artifact, whose
original row built all local packages before reaching the failed VRT.
Three tracked types differ from generation: `EndOfLineNapi`, the options in
`OxlintHtmlRule` and the fix in `OxlintHtmlDiagnostic`. This existing public
type mismatch is not a proved new fatal gate; declarations are not rewritten
or exempted in this transport correction.

The browser control starts its server and launches Chromium inside its
existing `try/finally`. A genuine complete-child missing-browser control
previously needed a five-second watchdog because the server survived launch
refusal; the correction exits failed promptly and releases the server.
Its original browser assertions and the app's passive sidecar are unchanged.
These are preparation and transport repairs. The original current-app
CSSOM/module packet and complete unchanged VRT remain unqualified.

The exact `fd8b2d9` ordinary JS check rejects one warning: the missing-browser
watchdog negates the optional `ChildProcess.pid`. It now sends the original
process-group signal only when that PID is a number. The same installed
Vite Plus 0.1.24 whole-source lint reproduces that diagnostic before the
guard and removes it after; its other 269 local diagnostics are unchanged.
The genuine missing-browser child still exits
failed promptly. Local warning counts do not establish Actions acceptance.
The running `fd8b2d9` discovery source and its 38-case verdict stay immutable.
