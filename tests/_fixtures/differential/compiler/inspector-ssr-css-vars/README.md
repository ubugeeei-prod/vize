# Elk CSS variables and scoped slot comparison

`StatusMedia.vue.txt` is the complete unmodified
[Elk source](https://github.com/elk-zone/elk/blob/8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0/app/components/status/StatusMedia.vue)
from the existing pinned application fixture. Its byte identity, official
compiler version and implementation digest are recorded in `source.json`.
The upstream MIT license is retained as `LICENSE`.

`StatusMedia.ssr.original.ts.txt` retains the full official output from the
inspector before its missing `ssrCssVars` input was repaired. That correction
left the DOM reference unchanged. The SSR reference was independently generated through
the pinned public compiler-sfc `parse`, `compileScript`, and `compileTemplate`
APIs, with `ssrCssVars: descriptor.cssVars` and the same filename, production,
scope and binding metadata options as the inspector. Its script compilation
also receives `templateOptions.ssr`, so the SSR reference excludes client-only
`useCssVars` calls while retaining the server-rendered CSS variable.

`NavSideItem.vue.txt` is another complete unmodified source from the same
archive. Its scoped style has no `:slotted()` selector. Independent full DOM
and SSR references pass `slotted: descriptor.slotted` to both inline script
template options and separate template compilation. The corresponding
`.slotted.*` references insert exactly one scoped `:slotted(.nav-icon)` rule
before the existing closing style tag. They retain slot scope handling,
preventing an unconditional `slotted: false` implementation from passing.
`NavSideItem.source.json` records the source hash, exact inserted rule, and
the official Vue Vite plugin implementation supporting both invocation paths.

`tests/tooling/inspector-ssr-css-vars.test.ts` executes the actual JavaScript
embedded in the Rust inspector, using a real Node process and the pinned Vue
compiler. It compares the complete DOM and SSR code, requires no compiler
warnings, and checks that the actual server renderer consumes the CSS variable
sentinel. A negative control replaces only the one authored
`v-bind(gridColumnNumber)` with literal `2`; no CSS variable may be emitted.
It also compares the complete NavSideItem outputs and both scoped-slot cases.

The current complete reference files use `NODE_ENV=production`, matching the
Inspector's production compilation options. The later environment correction
removes Vue's development patch-flag annotations from these six outputs; source
bytes, CSS-variable behavior, and slotted-selector controls remain unchanged.
The original Mutes runtime regression is retained separately in
`../inspector-production-mode/`.

## Historical check

The public [v0.426.3 CLI](https://github.com/ubugeeei-prod/vize/releases/tag/v0.426.3)
at source `0c36541bde455913ad021d3d2f3bf0e7a0fcef08` reproduces the retained
Elk SSR budget of 9201 additions and 25188 removals on this same 259-file source
and Vue provider. Its Mac ARM64 archive SHA-256 is
`de013b84db921181df3ddd6e3bb5da04637b7cc677fed271053c94e0699eab95`;
the executable SHA-256 is
`4fb15d98311cd20d24082914a30e08101421e0262c40bdd79bc8b6ce8e82c26d`.
This release follows #6519 by 51 commits; it is not that PR's exact head.

Native output from the compared clean snapshot
`cd861aa001f7ea458f8b0147e5b9c850b9db1c87` differs because of subsequent
props-rest, page-metadata, nested-await and SSR CSS variable correctness fixes.
The CSS-variable option alone changes only StatusMedia's official output:
additions remain 9217 and removals become 25010. The subsequent scoped-slot
and SSR script-context corrections are documented in
`docs/davinci/decisions/2026-10-16-6830-inspector-ssr-css-vars.md`.
The addition budget still fails. No budget, authored consumer, native compiler
output, or snapshot assertion is relaxed by these fixtures.
