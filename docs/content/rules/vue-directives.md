---
title: "Vue Rules: Directive Conventions"
---

# Vue Rules: Directive Conventions

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/scoped-event-names`](https://vizejs.dev/rules/vue-directives.html#vue-scoped-event-names) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-scoped-event-names-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-scoped-event-names-good) | Recommend scoped event names using context:event format |
| [`vue/use-v-on-exact`](https://vizejs.dev/rules/vue-directives.html#vue-use-v-on-exact) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-use-v-on-exact-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-use-v-on-exact-good) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers |
| [`vue/v-bind-style`](https://vizejs.dev/rules/vue-directives.html#vue-v-bind-style) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-v-bind-style-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-v-bind-style-good) | Enforce `v-bind` directive style |
| [`vue/v-on-style`](https://vizejs.dev/rules/vue-directives.html#vue-v-on-style) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-v-on-style-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-v-on-style-good) | Enforce `v-on` directive style |
| [`vue/v-slot-style`](https://vizejs.dev/rules/vue-directives.html#vue-v-slot-style) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-v-slot-style-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-v-slot-style-good) | Enforce `v-slot` directive style |
| [`vue/warn-custom-directive`](https://vizejs.dev/rules/vue-directives.html#vue-warn-custom-directive) | [Bad](https://vizejs.dev/rules/vue-directives.html#vue-warn-custom-directive-bad) · [Good](https://vizejs.dev/rules/vue-directives.html#vue-warn-custom-directive-good) | Warn about custom directives that need registration |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
