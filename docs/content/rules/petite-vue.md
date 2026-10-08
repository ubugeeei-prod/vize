---
title: petite-vue rules
---

# petite-vue rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | [Bad](./reference/petite-vue-no-unsupported-directive.md#bad) · [Good](./reference/petite-vue-no-unsupported-directive.md#good) | Disallow directives that petite-vue does not support |
| [`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | [Bad](./reference/petite-vue-valid-v-effect.md#bad) · [Good](./reference/petite-vue-valid-v-effect.md#good) | Require v-effect to have a non-empty expression |
| [`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | [Bad](./reference/petite-vue-valid-v-scope.md#bad) · [Good](./reference/petite-vue-valid-v-scope.md#good) | Require v-scope to bind an object literal |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
