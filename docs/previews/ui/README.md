# Component documentation previews

The component hub is generated at `/guide/ui` in English and each locale. Each
family page uses the basic SFC maintained beside its component source. Only
module specifiers change: catalog entry imports become public `@vizejs/ui/*`
imports. The entire example is used both as copyable documentation and as the
interactive iframe source.

`build-config.ts` binds those public imports to the current workspace catalog
entry files. It uses the same Vue compiler plugin as the UI package build, not
HTML imitations or a previously published component version. The optional CSS
comes from the matching workspace styles. It does not change the documentation
site's renderer or Vue compiler configuration.

The docs build generates the browser app under
`docs/public/component-previews/app/`, which the static site copies unchanged.
All basic examples receive live previews. Eight common examples additionally
receive actual Playwright screenshots: Button, Input, Checkbox, Switch, Tabs,
Dialog, Alert, and Card. These are illustrative captures after the interactions
in `capture.ts`, not pixel-difference baselines.

```bash
vp run --filter './docs' generate:ui-previews
vp node --test tests/tooling/lib-reference-docs.test.ts
vp run --filter './docs' check:ui-docs
```

The preview build fails on browser errors or warnings, broken interactions,
missing public entry mappings, or horizontal overflow at a 360px viewport.
It also verifies dialog Escape/focus return and the theme selector. Generated
browser evidence records the source and successful scenarios. It does not claim
that every component has been audited for accessibility or every advanced
variant has been exercised.

After SSG completes, `check-site.ts` checks eight generated English/Japanese
documentation routes, their local links and images, and an embedded Button's
state update at desktop/mobile widths. The production docs build runs both
browser checks with its normal OG-image configuration.

## Composable examples

Six complete SFCs in `npm/compose/core/examples/` demonstrate disclosure,
bounded quantities, debounced local search, blur validation, undo/redo, and
pagination. Their displayed source and compiled preview are byte-identical,
with a SHA-256 identity exposed by each iframe. They use public composable
imports and can be copied into a Vue 3.5+ project. The preview shell's CSS is
optional presentation; the examples themselves use native controls.

`build-ui-previews.ts` typechecks those SFCs against workspace sources before
compilation. `prerender-composables.ts` uses the existing Vue/Vite configuration
and actual Vue server renderer. A fresh plugin instance builds the browser app.
The client hydrates the matching markup; the checks require all initial
elements and complete markup to survive hydration, without warnings or errors.

`capture-composables.ts` runs every example at 720px and 360px in both initial
and interacted states. It checks keyboard activation/blur, state boundaries,
debounce timing/flush/cancel, validation, undo/redo/batching, and pagination.
Browser clock control advances the actual debounce timer deterministically.
Twenty-four real PNGs and a JSON receipt record source hashes, image hashes,
and individual interaction results under the ignored
`public/component-previews/composables/` build output. Each reference page
offers initial/after images in a native disclosure alongside its live preview.

After the real SSG build, `check-site.ts` verifies sixteen routes at desktop
and mobile widths, including the six exact displayed SFC packets and their
iframe identities, then runs the unchanged composable interaction checks.
The existing Docs Actions build executes both paths. To check deployed content:

```bash
vp node docs/previews/ui/check-site.ts --site https://vizejs.dev --output docs-render-evidence/deployed-composables
```

The supplied deployed URL must expose the expected source hashes; an older
deployment fails instead of receiving current-source credit. These six
examples are a bounded acceptance slice, not complete live coverage of the
composable catalogue. #8374 and the broader #6101 remain open for that work.
