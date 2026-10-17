# Inspector comparison fixes the official compiler's production environment

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The Inspector requests production SFC compilation for both native and official
outputs, but its Node subprocess inherited the caller's `NODE_ENV`. Vue's
`isProd: true` API option does not select its production compiler-core/compiler-dom
implementations. Their package entry points also consult the Node environment.
The same Inspector command could therefore receive different reference output
without changing source, options or provider version.

The original pinned Elk `app/pages/mutes.vue` demonstrates an observable case.
Its MainContent default slot contains `TimelineMutes v-if="isHydrated"`. When a
render-function host invokes that complete generated slot without an SSR push
callback, the development compiler emits `<!--v-if-->` for the false branch;
the production compiler emits `<!---->`. Actual Vue 3.6.0-beta.10 SSR execution
confirms the difference. The true branch and direct SSR-push route produce
identical full HTML in both modes. The source-built native output from Actions
[38900472471](https://github.com/ubugeeei-prod/vize/actions/runs/38900472471), source
`4a3fe42686fc9713b0fafaccb47684a5622ead13`, matches the production result in all
four observations. This mismatch predates the later loop-fragment repair.

The embedded Node script now selects `NODE_ENV=production` before loading any
Vue compiler module. Only that comparison subprocess is affected. The surrounding
749-line Rust source does not grow. Native compiler options, comparison metrics,
source identities, budgets and expected error counts are unchanged.

The new whole-source fixture retains all 359 original bytes and both complete
DOM/SSR reference outputs, with upstream license and source/provider hashes.
The test invokes the actual embedded script under unset, development and
production parent environments. All must produce the same complete production
reference. It also executes the complete original emitted SSR render with real
Vue helpers, controlled child hosts, both slot routes and both condition values.
The development reference remains an executable failing control for the false
VNode-slot branch. The two new tests failed before the fix and passed after it.
Original script setup, whole-app behavior and browser hydration are outside this
focused runtime check.

The six existing full StatusMedia/NavSideItem references now use the production
compiler environment. Their only changes are removal of debug patch-flag and
slot annotations; the original source, CSS-variable and slotted-selector
positive/negative controls remain exact. Historical pre-CSS-variable output is
retained separately without regeneration.

A separate read-only replay preserved every original input across Elk (259),
VueFes (71) and npmX (220), using the same exact official script and provider
implementations. Its development output reproduced all 1,100 hosted comparison
packets exactly. Selecting production changed official code/formatted-code in
529 DOM packets and 379 SSR packets, with unchanged warnings, error counts and
native formatting. Loaded compiler implementation paths and SHA256 values were
captured. Full packets and the original-source runtime evidence are retained at
`/tmp/vize-inspector-production-oracle-20261016` for this investigation.

Those broad output changes are not all native correctness improvements. Native
DOM code still contains some development-style debug annotations/comment labels,
and native SSR has a separately known redundant client CSS-variable helper.
Fixing the reference environment does not establish semantic equivalence for all
remaining output differences or authorize numerical budget changes. Fresh
source-built Linux qualification and a separate review of any remaining
individual directional overage are required; neither the withdrawn earlier
proposal nor intermediate source counts are a replacement baseline.
