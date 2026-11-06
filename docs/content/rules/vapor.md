---
title: "Vapor rules"
description: "Template and script constraints for components compiled in Vapor mode."
---

# Vapor rules

Template and script constraints for components compiled in Vapor mode.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Default (1)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor). |  |

## Stricter presets (5)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Disallow getCurrentInstance() in Vapor mode (returns null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Disallow Options API patterns in Vapor mode. | `opinionated` |  |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Disallow deprecated inline-template attribute. | `nuxt`, `opinionated` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Prefer static class over dynamic class binding for string literals. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Suggest adding vapor attribute to script setup. | `nuxt`, `opinionated` |  |

## Opt-in (1)

Not in any preset. Turn them on by name in the `linter.rules` passed to `withVue`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Disallow nextTick() usage in Vapor-oriented components. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
