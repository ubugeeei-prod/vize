---
title: "SSR rules"
description: "Code that breaks or behaves differently during server rendering and hydration."
---

# SSR rules

Code that breaks or behaves differently during server rendering and hydration.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Default (2)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Disallow browser-only globals in SSR context. |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Disallow non-deterministic values that cause hydration mismatch. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
