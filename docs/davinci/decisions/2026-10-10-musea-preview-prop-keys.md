# Musea preview prop snapshot ownership

Issue: [#8448](https://github.com/ubugeeei-prod/vize/issues/8448).

## Decision

Keep the editor's API palette, editable values and custom controls as raw JSON
snapshots behind shallow refs. Every existing edit/load/reset replaces these
snapshots, so deep proxies add no required mutation tracking. They reinterpret
literal `hasOwnProperty` reads and make nested object defaults uncloneable by
the real iframe `postMessage` path. The reset slot snapshot uses the same rule.

The generated interactive preview uses a shallow ref and whole-object spread
replacement. This retains each literal own key, clears previous keys, and never
assigns `__proto__` through an inherited setter. The frozen-props generator parses
its serialized JSON instead of treating JSON bytes as a JavaScript object
initializer, preserving nested and top-level own `__proto__` keys.

## Regression evidence

The authored differential fixture contains `__proto__`, `constructor`,
`hasOwnProperty`, an ordinary label, a nested own-prototype configuration, and
an ordinary custom object containing arrays, null, booleans and attribute text.
The existing Musea Chromium entry runs the real gallery editor, production
preview HTML/module generators, message handshake, Vue apps and DOM. Explicit
API/art-module fixtures avoid claiming native SFC compilation.

Original deep-ref editor behavior produces `DataCloneError` before delivery.
With only the editor repaired, the original generated store loses its own
`__proto__` and `hasOwnProperty` values and changes prototype. Complete failed
observations are retained before assertions. Corrected execution checks full
store values/own keys/prototype identity and full Vue DOM/attrs after authored
load, label edit/custom addition, exact clipboard execution, Save/reload,
Reset, removal and the separate frozen-props generator. The existing always
uploaded observations retain all phases, clipboard bytes and page errors.

Vue intrinsically reserves a direct `__proto__` component prop, and its props
proxy instruments `hasOwnProperty` reads. The real authored Probe uses Vue's
`toRaw` to observe the delivered literal values; copied-template capture does
the same. These tests preserve those framework behaviors and assert no leaked
inherited attrs; they do not change Vue policy.

## Remaining work and delivery

An independently observed Monaco async mount can finish after its control is
removed during reload/reset and create an editor with a null container. This
bounded cancellation follow-up is unfinished; this ownership regression waits
for actual editor initialization before unmounting controls.

Focused local browser/source checks passed. Fresh exact-head Actions, native
Stack protected qualification, actual merge and root-owned publication remain
required. No dependency, manifest, release authority or upstream changes.
