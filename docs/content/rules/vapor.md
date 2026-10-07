---
title: Vapor rules
---

# Vapor rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-next-tick`](./reference/script-no-next-tick.md) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](./reference/script-no-options-api.md) | Disallow Options API patterns in Vapor mode |
| [`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Disallow deprecated inline-template attribute |
| [`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) |
| [`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Prefer static class over dynamic class binding for string literals |
| [`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Suggest adding vapor attribute to script setup |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
