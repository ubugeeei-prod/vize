---
title: "Vue Rules: Template Structure"
---

# Vue Rules: Template Structure

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-child-content`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-child-content) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-child-content-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-child-content-good) | Disallow child content when using v-html or v-text |
| [`vue/no-dupe-v-else-if`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-dupe-v-else-if) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-dupe-v-else-if-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-dupe-v-else-if-good) | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| [`vue/no-lone-template`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-lone-template) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-lone-template-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-lone-template-good) | Disallow unnecessary `<template>` elements |
| [`vue/no-template-key`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-key) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-key-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-key-good) | Disallow `key` attribute on `<template>` |
| [`vue/no-template-shadow`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-shadow) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-shadow-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-template-shadow-good) | Disallow variable names that shadow variables in outer scope |
| [`vue/no-unused-vars`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-unused-vars) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-unused-vars-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-unused-vars-good) | Disallow unused variable definitions in v-for and v-slot directives |
| [`vue/no-use-v-if-with-v-for`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-use-v-if-with-v-for-good) | Disallow using `v-if` on the same element as `v-for` |
| [`vue/no-useless-template-attributes`](https://vizejs.dev/rules/vue-template-structure.html#vue-no-useless-template-attributes) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-no-useless-template-attributes-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-no-useless-template-attributes-good) | Disallow useless attributes on `<template>` elements |
| [`vue/require-v-for-key`](https://vizejs.dev/rules/vue-template-structure.html#vue-require-v-for-key) | [Bad](https://vizejs.dev/rules/vue-template-structure.html#vue-require-v-for-key-bad) · [Good](https://vizejs.dev/rules/vue-template-structure.html#vue-require-v-for-key-good) | Require `v-bind:key` with `v-for` directives |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
