---
title: "Vue Rules: Components and Props"
---

# Vue Rules: Components and Props

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/component-definition-name-casing`](https://vizejs.dev/rules/vue-components.html#vue-component-definition-name-casing) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-component-definition-name-casing-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-component-definition-name-casing-good) | Enforce PascalCase or kebab-case for component definition names |
| [`vue/component-name-in-template-casing`](https://vizejs.dev/rules/vue-components.html#vue-component-name-in-template-casing) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-component-name-in-template-casing-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-component-name-in-template-casing-good) | Enforce specific casing for component names in templates |
| [`vue/multi-word-component-names`](https://vizejs.dev/rules/vue-components.html#vue-multi-word-component-names) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-multi-word-component-names-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-multi-word-component-names-good) | Require component names to be multi-word |
| [`vue/no-mutating-props`](https://vizejs.dev/rules/vue-components.html#vue-no-mutating-props) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-no-mutating-props-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-no-mutating-props-good) | Disallow mutating component props |
| [`vue/no-non-component-keep-alive-child`](https://vizejs.dev/rules/vue-components.html#vue-no-non-component-keep-alive-child) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-no-non-component-keep-alive-child-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-no-non-component-keep-alive-child-good) | Disallow plain element wrappers directly below `<KeepAlive>` |
| [`vue/no-reserved-component-names`](https://vizejs.dev/rules/vue-components.html#vue-no-reserved-component-names) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-no-reserved-component-names-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-no-reserved-component-names-good) | Disallow the use of reserved names as component names |
| [`vue/no-unused-components`](https://vizejs.dev/rules/vue-components.html#vue-no-unused-components) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-no-unused-components-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-no-unused-components-good) | Disallow registering components that are not used inside templates |
| [`vue/no-unused-properties`](https://vizejs.dev/rules/vue-components.html#vue-no-unused-properties) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-no-unused-properties-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-no-unused-properties-good) | Disallow unused properties defined in defineProps |
| [`vue/require-component-is`](https://vizejs.dev/rules/vue-components.html#vue-require-component-is) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-require-component-is-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-require-component-is-good) | Require `v-bind:is` on `<component>` elements |
| [`vue/require-component-registration`](https://vizejs.dev/rules/vue-components.html#vue-require-component-registration) | [Bad](https://vizejs.dev/rules/vue-components.html#vue-require-component-registration-bad) · [Good](https://vizejs.dev/rules/vue-components.html#vue-require-component-registration-good) | Require explicit import or registration for components |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
