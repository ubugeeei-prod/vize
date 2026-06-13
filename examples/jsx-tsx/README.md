# JSX/TSX Example

This directory keeps focused JSX/TSX inputs for the compiler, linter, type checker, LSP, and
formatter.

```bash
vp run --filter './examples/jsx-tsx' check
vp run --filter './examples/jsx-tsx' lint
vp run --filter './examples/jsx-tsx' fmt
```

Run these commands from the repository root. The example is part of the pnpm workspace and the
repository Vite+ check list, so `vp run check` includes it in aggregate workspace checks.

- `src/StatefulPanel.tsx` shows typed destructured props, setup state, emits, slots, JSX list
  rendering, scoped styles, and CSS custom properties for dynamic styling.
- `src/AccessibleMedia.jsx` is a clean JSX lint sample with accessible media and keyed lists.
- `src/FormattedScriptBlock.vue` verifies `.vue` `<script setup lang="tsx">` formatting.

The samples are intentionally small but stateful. They are meant to stay readable while still
exercising the compiler, type checker, lint, formatter, and editor pipelines.
