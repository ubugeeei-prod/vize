---
title: petite-vue rules
---

# petite-vue rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](./all.md#petite-vue-no-unsupported-directive) | [Bad](./all.md#petite-vue-no-unsupported-directive-bad) · [Good](./all.md#petite-vue-no-unsupported-directive-good) | Disallow directives that petite-vue does not support |
| [`petite-vue/valid-v-effect`](./all.md#petite-vue-valid-v-effect) | [Bad](./all.md#petite-vue-valid-v-effect-bad) · [Good](./all.md#petite-vue-valid-v-effect-good) | Require v-effect to have a non-empty expression |
| [`petite-vue/valid-v-scope`](./all.md#petite-vue-valid-v-scope) | [Bad](./all.md#petite-vue-valid-v-scope-bad) · [Good](./all.md#petite-vue-valid-v-scope-good) | Require v-scope to bind an object literal |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
