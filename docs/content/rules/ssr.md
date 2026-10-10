---
title: "SSR rules"
---

# SSR rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](https://vizejs.dev/rules/ssr.html#ssr-no-browser-globals-in-ssr) | [Bad](https://vizejs.dev/rules/ssr.html#ssr-no-browser-globals-in-ssr-bad) · [Good](https://vizejs.dev/rules/ssr.html#ssr-no-browser-globals-in-ssr-good) | Disallow browser-only globals in SSR context |
| [`ssr/no-hydration-mismatch`](https://vizejs.dev/rules/ssr.html#ssr-no-hydration-mismatch) | [Bad](https://vizejs.dev/rules/ssr.html#ssr-no-hydration-mismatch-bad) · [Good](https://vizejs.dev/rules/ssr.html#ssr-no-hydration-mismatch-good) | Disallow non-deterministic values that cause hydration mismatch |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
