---
title: Vapor rules
---

# Vapor rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`script/no-get-current-instance`](./all.md#script-no-get-current-instance) | [Bad](./all.md#script-no-get-current-instance-bad) · [Good](./all.md#script-no-get-current-instance-good) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-next-tick`](./all.md#script-no-next-tick) | [Bad](./all.md#script-no-next-tick-bad) · [Good](./all.md#script-no-next-tick-good) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](./all.md#script-no-options-api) | [Bad](./all.md#script-no-options-api-bad) · [Good](./all.md#script-no-options-api-good) | Disallow Options API patterns in Vapor mode |
| [`vapor/no-inline-template`](./all.md#vapor-no-inline-template) | [Bad](./all.md#vapor-no-inline-template-bad) · [Good](./all.md#vapor-no-inline-template-good) | Disallow deprecated inline-template attribute |
| [`vapor/no-vue-lifecycle-events`](./all.md#vapor-no-vue-lifecycle-events) | [Bad](./all.md#vapor-no-vue-lifecycle-events-bad) · [Good](./all.md#vapor-no-vue-lifecycle-events-good) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) |
| [`vapor/prefer-static-class`](./all.md#vapor-prefer-static-class) | [Bad](./all.md#vapor-prefer-static-class-bad) · [Good](./all.md#vapor-prefer-static-class-good) | Prefer static class over dynamic class binding for string literals |
| [`vapor/require-vapor-attribute`](./all.md#vapor-require-vapor-attribute) | [Bad](./all.md#vapor-require-vapor-attribute-bad) · [Good](./all.md#vapor-require-vapor-attribute-good) | Suggest adding vapor attribute to script setup |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
