# Page metadata belongs to the Nuxt integration (#7821)

The [original report](https://github.com/ubugeeei-prod/vize/issues/7821)
shows ordinary `compileSfc`, CLI and Vite compilation removing both an import
from `#imports` and its `definePageMeta` call. A global call is also removed.
Vue's [3.5.41 compiler](https://github.com/vuejs/core/blob/v3.5.41/packages/compiler-sfc/src/compileScript.ts)
retains these ordinary calls. Nuxt's [3.19.3 page-meta plugin](https://github.com/nuxt/nuxt/blob/v3.19.3/packages/nuxt/src/pages/plugins/page-meta.ts)
owns extraction for `?macro=true` requests; the existing Vize artifact supports
that path after [#7035](https://github.com/ubugeeei-prod/vize/issues/7035).

## Decision

Keep global and `#imports` page-meta calls and imports in ordinary SFC runtime
output. Add the explicit native/Vite `nuxtPageMeta` option, set by the Nuxt
module, to enable its artifact extraction and runtime erasure. Scope both the
import edits and call edits to that option. Preserve explicit typed-router macro
handling and locally bound or aliased runtime calls. Keep the shared Croquis
macro lifecycle registry intact; ordinary SFC statement and fallback helpers
defer page-meta erasure to the explicit integration pass.

The new adapter entry point retains all established Rust option structures and
function signatures. Single-file and parallel batch NAPI calls choose the same
explicit policy. The Vite batch options carry it into persistent cache identity.
This adds no pipeline stage, serialization, dependency reversal or instruction
budget relaxation. It makes no native-stage, default-readiness or speed claim.

## Verification and remaining work

Retain the complete issue body, original executable reproduction and whole SFC,
plus the complete #7035 report. The hashed differential corpus covers ordinary
global and imported calls, mixed runtime imports and middleware functions,
aliases and locally bound functions. Source-built complete default components
must mount/render like the official compiler, with exact call arguments and
middleware results, across production module/inline DOM, SSR and Vapor. Vapor
SSR retains the existing explicit VDOM fallback and warning; it gains no direct
Vapor SSR acceptance. Maps must remain additive, and opted-in Nuxt compilation
must retain actual artifacts and remove only its eligible calls.

The existing source-qualified pinned Nuxt 3 fixture gains explicit and global
page-meta routes with real layout selection on SSR and Chromium. All original
SFCs, lockfiles, SSR/CSS/interaction assertions and native custody guards remain.
Fresh exact-source Actions, full protected suites and all 104 unchanged
instruction ceilings, signed actual merge and reporter trailer preservation
remain required. Package publication and public consumer proof belong to the
release owner after actual delivery; no execution or publication is credited
by this preparatory decision.

Paired issue decision: [#7821 comment](https://github.com/ubugeeei-prod/vize/issues/7821#issuecomment-5987072004).
