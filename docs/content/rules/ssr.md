---
title: SSR rules
---

# SSR rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](./all.md#ssr-no-browser-globals-in-ssr) | [Bad](./all.md#ssr-no-browser-globals-in-ssr-bad) · [Good](./all.md#ssr-no-browser-globals-in-ssr-good) | Disallow browser-only globals in SSR context |
| [`ssr/no-hydration-mismatch`](./all.md#ssr-no-hydration-mismatch) | [Bad](./all.md#ssr-no-hydration-mismatch-bad) · [Good](./all.md#ssr-no-hydration-mismatch-good) | Disallow non-deterministic values that cause hydration mismatch |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
