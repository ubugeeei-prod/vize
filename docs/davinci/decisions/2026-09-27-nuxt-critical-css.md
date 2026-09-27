# Nuxt critical CSS module identity

Issue: [#6897](https://github.com/ubugeeei-prod/vize/issues/6897).

## Decision

Keep plugin-visible SFC paths ending in `.vue`, with the existing ownership
query. Client and SSR use the same ID; hook options or Vite's actual environment
select their separate compilation caches. Existing null-byte and `.vue.ts[x]`
IDs remain readable, including HMR graph lookup. TypeScript stripping stays
explicit for macro and uncached output; compiled JavaScript stays unchanged.

Nuxt's CSS collector rejects dots inside CSS query strings. Encode query dots
as `%2E`; URLSearchParams still recovers the exact authored filename and named
module. Resolve relative emitted inline style entries from their existing
`vize-file` metadata. Preserve #6825's style index, language, scoping and
idempotent repeated resolution.

Nuxt's styles map is relative to `srcDir`. Pass that directory as the optional
`ssrModuleIdRoot` for SSR module registration, including source-relative paths
outside that directory. Keep compilation/configuration/assets rooted at the
project root. Plain Vite's default registration behavior remains unchanged.

## Regression evidence

The executable fixture is `tests/_fixtures/_projects/nuxt-critical-css-build`:
the exact reported config, app, default layout and index page. The configured
versions are Nuxt 4.5.2, Vite 8.3.1, Rolldown 1.2.11, Vue 3.5.43 and published
Vize 0.429.0. A stock control changes only the Nuxt module selection.

`critical-css-build.mjs` copies the fixture and packages into separate stock,
published and candidate workspaces. Dependency caches stay separate. The
published package bytes are checked after the run. Candidate JavaScript comes
from the source build; the released native addon is retained and hashed.

The complete styles map and client manifest, prerender HTML and scoped CSS
are saved. Chromium disables application JavaScript and aborts script files.
Stock/candidate must have exactly two inline CSS blocks, no stylesheet/style
prefetch links, grid layout, 32px gap and purple title. Each inline block must
match the loaded style-module array exactly. Published 0.429.0 must demonstrate
empty styles mapping and block/normal/black computed values.

Pure controls execute the same canonical ID against real client/SSR caches,
load both actual generated components, check distinct render/ssrRender ABIs,
render the SSR component with Vue, and verify source-relative module registration.
Existing #6825 checks still require exact delivered scoped CSS, now accepting
inline or linked delivery. CSS URI snapshots change only the required encoding.

The first Linux Actions run at `a7cf6c508` exposed a separate build-order case:
Nuxt loads an emitted inline CSS entry before its SFC while both compilation
caches are empty. Resolving the source metadata already succeeds, but the style
loader had only read caches and returned `null`. Compile the actual source on
demand using the requested environment's existing options and cache. The
regression uses the emitted relative `app/app.vue.__vize_style_0.css` ID, starts
with empty caches, and checks exact scoped CSS and reuse by the subsequent SFC.
The existing SFC admission check still excludes cold host-owned sources, and
existing cached fallbacks retain their previous behavior. The real SSR unit
test declares Vue via the existing `vue-stable` catalog (3.5.35); its runtime is
separate from the pinned Nuxt fixture's Vue 3.5.43.
Keep the original failing run/artifacts as evidence; its PASS is not inferred
from prior cached-runtime runs.

## Remaining gates

Local source-built JavaScript and cached public native runtime reproduced the
reported failure and the fixed no-JavaScript rendering. Exact-head Actions,
merged publication and published-version reruns remain required before release
claims. This record grants no compiler-history or native-migration acceptance.
[#6898](https://github.com/ubugeeei-prod/vize/issues/6898) separately fixes SSR slot
scope propagation; its production/compiler changes are outside this patch.
