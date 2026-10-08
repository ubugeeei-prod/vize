---
title: Ecosystem rules
---

# Ecosystem rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | [Bad](./reference/ecosystem-nuxt-prefer-nuxt-link.md#bad) · [Good](./reference/ecosystem-nuxt-prefer-nuxt-link.md#good) | Prefer NuxtLink for internal application links |
| [`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | [Bad](./reference/ecosystem-pinia-prefer-store-to-refs.md#bad) · [Good](./reference/ecosystem-pinia-prefer-store-to-refs.md#good) | Prefer storeToRefs() when destructuring Pinia stores |
| [`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | [Bad](./reference/ecosystem-router-link-require-to.md#bad) · [Good](./reference/ecosystem-router-link-require-to.md#good) | Require a `to` target on RouterLink and NuxtLink components |
| [`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | [Bad](./reference/ecosystem-void-link-require-href.md#bad) · [Good](./reference/ecosystem-void-link-require-href.md#good) | Require `href` on Void Vue Link components |
| [`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | [Bad](./reference/ecosystem-void-link-valid-method.md#bad) · [Good](./reference/ecosystem-void-link-valid-method.md#good) | Validate static Void Vue Link method props |
| [`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | [Bad](./reference/ecosystem-vue-i18n-no-missing-key.md#bad) · [Good](./reference/ecosystem-vue-i18n-no-missing-key.md#good) | Report static vue-i18n keys that are absent from local SFC messages |
| [`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | [Bad](./reference/ecosystem-vue-router-prefer-named-link.md#bad) · [Good](./reference/ecosystem-vue-router-prefer-named-link.md#good) | Prefer named route objects over static path strings in RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | [Bad](./reference/ecosystem-vue-router-prefer-named-push.md#bad) · [Good](./reference/ecosystem-vue-router-prefer-named-push.md#good) | Prefer named route objects for Vue Router programmatic navigation |
| [`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | [Bad](./reference/ecosystem-vue-test-utils-no-html-snapshot.md#bad) · [Good](./reference/ecosystem-vue-test-utils-no-html-snapshot.md#good) | Avoid snapshotting wrapper.html() in Vue Test Utils tests |
| [`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | [Bad](./reference/nuxt-no-nuxt-config-test-key.md#bad) · [Good](./reference/nuxt-no-nuxt-config-test-key.md#good) | Disallow setting `test` key in Nuxt config |
| [`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | [Bad](./reference/nuxt-no-page-meta-runtime-values.md#bad) · [Good](./reference/nuxt-no-page-meta-runtime-values.md#good) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup |
| [`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | [Bad](./reference/nuxt-nuxt-config-keys-order.md#bad) · [Good](./reference/nuxt-nuxt-config-keys-order.md#good) | Prefer recommended order of Nuxt config properties |
| [`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | [Bad](./reference/nuxt-prefer-import-meta.md#bad) · [Good](./reference/nuxt-prefer-import-meta.md#good) | Prefer using `import.meta.*` over `process.*` |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For the four typed Vue Router diagnostics, see the [complete project examples](./cross-file.md).
