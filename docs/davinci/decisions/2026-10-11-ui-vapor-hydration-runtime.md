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
fixture's `assertServerMarkup`. A separate local diagnostic applying that
additional oracle exposed failures for `icon`, `icon-button`,
`media-player-time-display`, `scrubber-preview-time`, `signature-pad-canvas`,
`webcam-capture-start-button` and `webcam-capture-shutter`. TODO: reproduce
and repair those server-markup gaps before making that additional oracle a
required whole-corpus gate. They are not new ledger exemptions. The added
minimal regression already compares complete native and official SSR output.

Local execution with published native 0.443 and the existing 527-fixture main
corpus observed SSR 527 passing, mount 502 passing / 25 remaining failures,
and hydration 479 passing / 48 remaining failures with same-version rc.11
providers. This is diagnosis, not current-source Actions or protected queue
credit. Remove only ledger entries genuinely passing in exact-source Actions.
Replay all 529 fixtures after #8553 and all 533 after #8555; retain the four
finite-range fixtures as required successes in the supported lane. Actual
merge and a release containing the changes remain required.
