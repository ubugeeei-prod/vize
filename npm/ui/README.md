# @vizejs/ui

Accessible, headless Vue components authored as single-file components. Visual
presets are optional; component parts, slots, data attributes, and CSS custom
properties provide the extension points.

## Renderer compatibility

The standard DOM and SSR builds support Vue 3.5+. Vapor is experimental and
requires Vue **3.6.0-rc.11 or later**, with the Vue runtime, compiler, and server
renderer on the same version. The currently qualified Vapor version is rc.11.
Use that same version for rendering server markup and hydrating it in the client.

Vue rc.9 throws while hydrating a Vapor root containing CSS custom properties
under a VDOM parent. Later releases also changed the slot fallback markup used
between SSR and hydration. Combining Vue 3.5 server output with a newer Vapor
client can therefore produce hydration warnings even when the component state
matches. The runtime conformance suite executes the native SSR and Vapor output
with matching rc.11 providers, checks the original client assertions, and
retains the server root.

Some components still have tracked Vapor failures. This requirement describes
the supported runtime pairing; it does not certify every component or close
the remaining renderer work.

The original rc.9 compiler and runtime remain pinned as an independent
regression reference. Maintainers can replay the original comparison with:

```sh
VIZE_UI_VAPOR_RUNTIME=reference vp run --filter './npm/ui' check:vapor-runtime
```
