---
title: SSR rules
---

# SSR rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Disallow browser-only globals in SSR context |
| [`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Disallow non-deterministic values that cause hydration mismatch |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
