---
title: "Vapor rules"
---

# Vapor rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`script/no-get-current-instance`](https://vizejs.dev/rules/vapor.html#script-no-get-current-instance) | [Bad](https://vizejs.dev/rules/vapor.html#script-no-get-current-instance-bad) · [Good](https://vizejs.dev/rules/vapor.html#script-no-get-current-instance-good) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-next-tick`](https://vizejs.dev/rules/vapor.html#script-no-next-tick) | [Bad](https://vizejs.dev/rules/vapor.html#script-no-next-tick-bad) · [Good](https://vizejs.dev/rules/vapor.html#script-no-next-tick-good) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](https://vizejs.dev/rules/vapor.html#script-no-options-api) | [Bad](https://vizejs.dev/rules/vapor.html#script-no-options-api-bad) · [Good](https://vizejs.dev/rules/vapor.html#script-no-options-api-good) | Disallow Options API patterns in Vapor mode |
| [`vapor/no-inline-template`](https://vizejs.dev/rules/vapor.html#vapor-no-inline-template) | [Bad](https://vizejs.dev/rules/vapor.html#vapor-no-inline-template-bad) · [Good](https://vizejs.dev/rules/vapor.html#vapor-no-inline-template-good) | Disallow deprecated inline-template attribute |
| [`vapor/no-vue-lifecycle-events`](https://vizejs.dev/rules/vapor.html#vapor-no-vue-lifecycle-events) | [Bad](https://vizejs.dev/rules/vapor.html#vapor-no-vue-lifecycle-events-bad) · [Good](https://vizejs.dev/rules/vapor.html#vapor-no-vue-lifecycle-events-good) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) |
| [`vapor/prefer-static-class`](https://vizejs.dev/rules/vapor.html#vapor-prefer-static-class) | [Bad](https://vizejs.dev/rules/vapor.html#vapor-prefer-static-class-bad) · [Good](https://vizejs.dev/rules/vapor.html#vapor-prefer-static-class-good) | Prefer static class over dynamic class binding for string literals |
| [`vapor/require-vapor-attribute`](https://vizejs.dev/rules/vapor.html#vapor-require-vapor-attribute) | [Bad](https://vizejs.dev/rules/vapor.html#vapor-require-vapor-attribute-bad) · [Good](https://vizejs.dev/rules/vapor.html#vapor-require-vapor-attribute-good) | Suggest adding vapor attribute to script setup |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
