# UI Vapor hydration runtime pairing

This resolves the runtime blocker exposed by the finite progress-bar fixtures
in #8555, under #4906. It does not complete the feedback family or all Vapor
components.

## Reproduction and cause

The whole failed #8555 run
[38100684236](https://github.com/ubugeeei-prod/vize/actions/runs/38100684236)
rendered 533 fixtures. Its four new finite-range progress bars failed while
Vue rc.9 checked hydration styles: `styleHasMismatch` called `resolveCssVars`,
which treated a mounted VDOM parent as a Vapor block. A custom property in the
server style defers this check until after the parent mounts. The native and
official Vue compilers both reproduce the exception with the whole minimal SFC
`<div :style="style">`, where `style` contains `--percent: 50%`.

The official correction already shipped in Vue rc.10:
[f765167](https://github.com/vuejs/core/commit/f76516717f8e2be454cb277666bc152edf0d47f4).
Use the published rc.11 closure for UI Vapor support; do not rewrite the
compiler output or replace a runtime helper to bypass the exception.

The same update requires matching SSR and client versions. The Vue rc.10 slot
hydration contract distinguishes fallback output using `<!--(-->`. Vue 3.5
SSR helpers produce the older slot markers, so a rc.11 client combined with
Vue 3.5 server output introduces eleven new warning failures in the existing
527-fixture main corpus. Actual rc.11 server output removes all eleven.

## Compatibility and reference preservation

The new `vue-vapor-runtime-supported` dev alias pins Vue, compiler and server
renderer rc.11. Supported SSR uses its declared Node implementation and server
renderer; the client uses its unmodified runtime-with-vapor browser build.
The public peer range accepts rc.11 and the package README states the required
same-version pairing. Normal Vue 3.5 DOM/SSR tests and their providers remain.

The old `vue-vapor-runtime` rc.9 alias stays byte-exact for existing native,
Davinci and runtime oracles. `VIZE_UI_VAPOR_RUNTIME=reference` selects the
original Vue 3.5 SSR / rc.9 Vapor combination and the separately retained
original failure ledger. Neither provider is relabeled or patched.

The pnpm 12.1-generated lock adds only twelve rc.11 package records, twelve
snapshots and the supported UI importer. Its original 2,305 package records
and 2,384 snapshots are conserved byte-for-byte, including native versions
and the original Test Utils peers. The generated lock's unrelated native and
peer rewrites were excluded from the projection; the projected lock passes
frozen installation. The twelve recent-package policy entries name exactly
the new Vue closure.

## Required execution

The two added whole SFC corpus inputs cover static and dynamic roots, real
custom properties, reactive updates and disposal. Every native and official
rc.9/rc.11 generated module is executed on the original rc.9 runtime, where
the exact exception must recur, and on rc.11, where hydration must adopt the
server root, retain its sole CSS property, update to 75%, and remove every
node on unmount. Native and official SSR output must agree completely.

The supported whole UI corpus keeps the original server rendering and client
assertions. Any warning, thrown error, replaced root, missing component or
failed DOM assertion remains fatal unless already present in the original
ledger. No new failure exemptions or expanded budgets are permitted.

The original server lane checks rendering success; it does not invoke each
fixture's `assertServerMarkup`. A separate local diagnostic with published
native 0.443 exposed prop loss in `icon` and `icon-button` under both providers:
the icon lost its accessible title/description and the button lost its ARIA
label. Official Vue compilation preserves those props. The rc.11
provider additionally fails four old assertions because of its new
fallback comment markers: `media-player-time-display`, `scrubber-preview-time`,
`webcam-capture-start-button` and `webcam-capture-shutter`. The
[complete source-built SSR receipts](../evidence/2026-10-11-ui-vapor-ssr-markup-gaps.json)
confirm all seven against actual native source head `591667c9a6d98b10a7ed9cd3631a2c2e1c0bde30`:
every unchanged authored server-markup assertion was applied to the captured
raw 533-row SSR output. The original reference passes 530 assertions; the
supported provider passes 526. Both render all 533 successfully. Complete actual
HTML, original expected expressions, authored SFC sources and hashes, provider
labels and the actually loaded source binary are retained. Local module loading
only supplies the pure original assertions; no local rendering receives source
credit. TODO under [#6100](https://github.com/ubugeeei-prod/vize/issues/6100)
and compiler fix-history [#6880](https://github.com/ubugeeei-prod/vize/issues/6880): repair scoped union
property extraction for the two confirmed icon defects; qualify
the additional server-markup oracle for the real provider's marker contract
while retaining its existing content assertions and original reference
coverage. These observations create no ledger exemptions. The added minimal
regression already compares complete native and official SSR output.

`signature-pad-canvas` also fails the existing uppercase `viewBox` regex with
both native and official Vue SSR, which emit lowercase `viewbox`. HTML parsing
[adjusts that SVG attribute to `viewBox`](https://html.spec.whatwg.org/multipage/parsing.html#adjust-svg-attributes).
Do not classify this as a native rendering defect from raw spelling alone;
retain the original assertion until its byte and browser semantics are
qualified independently. The official full-corpus diagnostic also exposes
literal-tag assertions that reject scoped slot attributes in `masonry`,
`master-detail`, `responsive-show` and `visually-hidden`. These observations
describe additional oracle work, not permission to weaken existing checks.

Local execution with published native 0.443 and the existing 527-fixture main
corpus observed SSR 527 passing, mount 502 passing / 25 remaining failures,
and hydration 479 passing / 48 remaining failures with same-version rc.11
providers. This is diagnosis, not current-source Actions or protected queue
credit.

The source-built UI job in
[Actions 38107550411](https://github.com/ubugeeei-prod/vize/actions/runs/38107550411/job/114376553931),
at PR head `c3ee6f2dc96d6442421fa559bd5efa2805434e8f`, confirmed those same
527-fixture counts and the complete twelve-cell minimal regression. It failed
solely because 149 rows now passed while still listed in the supported ledger.
Those rows represent 145 distinct entries: 35 mount and 110 hydration entries.
The summary records every remaining failure name; none overlaps a newly passing
name, including duplicate fixture names. This change removes only that exact
source-proven subset and preserves every remaining reason and the byte-exact
original reference ledger.

| Supported lane | Passing rows | Original known entries | Retained known entries |
| -------------- | -----------: | ---------------------: | ---------------------: |
| SSR            |          527 |                      0 |                      0 |
| Mount          |          502 |                     60 |                     25 |
| Hydration      |          479 |                    158 |                     48 |

The stale-ledger diagnostic does not make the overall Check run green. The
private composition starts from the actual #8553 merge
`db6e7805e5c26aafe8ae1614bc77668cd3459d1d` (529 fixtures), retains the exact
#8555 source at `49acf8c4173ae8dde78a0b23520b92f386458af0`, and adds the runtime
pairing to execute all 533. The source-built supported UI gate in
[Actions 38109593412](https://github.com/ubugeeei-prod/vize/actions/runs/38109593412/job/114382213283)
passes the original renderer, lint, static, story and Vapor gates, plus 3,833
tests across 497 files in each standard/production run. SSR renders 533, mount
passes 508 with 25 retained failures, and hydration passes 485 with 48 retained
failures; no new failure or stale entry remains. All four finite-range fixtures
are required successes, with no new ledger entry.

The same complete workload on the original reference retains its 60 mount and
158 hydration ledger entries byte-for-byte. Its only four new failures reproduce
the original ProgressBar `undefined.nodes` exception. Two existing duplicate
names account for 62 mount failure rows and 164 hydration failure rows; the latter
includes the four new failures. The reference process retains its failing exit
status. A private negative oracle checks precisely those four errors, complete
fixture ordering, every retained failure and the reference ledger's original
bytes. The first wrapper replay failed while parsing a pretty-printed story
summary, after both workloads executed; the parser correction changes no
product, fixture or oracle. The corrected complete private run
[38109890341](https://github.com/ubugeeei-prod/vize/actions/runs/38109890341/job/114383084822)
at `3ce7146b283ae69f5c2470a241b58a48bccd5161` is terminal SUCCESS. Its
source-built 533-fixture controls and strict before/after receipt all pass.
Fresh PR head Actions, protected merge and release inclusion remain required.
