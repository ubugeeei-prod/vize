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
