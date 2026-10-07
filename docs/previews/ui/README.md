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
in `capture.mjs`, not pixel-difference baselines.

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

After SSG completes, `check-site.mjs` checks eight generated English/Japanese
documentation routes, their local links and images, and an embedded Button's
state update at desktop/mobile widths. The production docs build runs both
browser checks with its normal OG-image configuration.
