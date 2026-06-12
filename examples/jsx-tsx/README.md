# JSX/TSX Example

This directory keeps focused JSX/TSX inputs for the compiler, linter, type checker, LSP, and
formatter.

```bash
vp check src
vp lint src
vp fmt --write src
```

- `src/StatefulPanel.tsx` shows typed destructured props, setup state, emits, slots, JSX list
  rendering, scoped styles, and CSS custom properties for dynamic styling.
- `src/AccessibleMedia.jsx` is a clean JSX lint sample with accessible media and keyed lists.
- `src/FormattedScriptBlock.vue` verifies `.vue` `<script setup lang="tsx">` formatting.

The samples are intentionally small but stateful. They are meant to stay readable while still
exercising the compiler, type checker, lint, formatter, and editor pipelines.
