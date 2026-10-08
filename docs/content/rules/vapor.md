---
title: Vapor rules
---

# Vapor rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | [Bad](./reference/script-no-get-current-instance.md#bad) · [Good](./reference/script-no-get-current-instance.md#good) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-next-tick`](./reference/script-no-next-tick.md) | [Bad](./reference/script-no-next-tick.md#bad) · [Good](./reference/script-no-next-tick.md#good) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](./reference/script-no-options-api.md) | [Bad](./reference/script-no-options-api.md#bad) · [Good](./reference/script-no-options-api.md#good) | Disallow Options API patterns in Vapor mode |
| [`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | [Bad](./reference/vapor-no-inline-template.md#bad) · [Good](./reference/vapor-no-inline-template.md#good) | Disallow deprecated inline-template attribute |
| [`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | [Bad](./reference/vapor-no-vue-lifecycle-events.md#bad) · [Good](./reference/vapor-no-vue-lifecycle-events.md#good) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) |
| [`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | [Bad](./reference/vapor-prefer-static-class.md#bad) · [Good](./reference/vapor-prefer-static-class.md#good) | Prefer static class over dynamic class binding for string literals |
| [`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | [Bad](./reference/vapor-require-vapor-attribute.md#bad) · [Good](./reference/vapor-require-vapor-attribute.md#good) | Suggest adding vapor attribute to script setup |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
