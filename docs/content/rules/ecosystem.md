---
title: Ecosystem rules
---

# Ecosystem rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Prefer NuxtLink for internal application links |
| [`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Prefer storeToRefs() when destructuring Pinia stores |
| [`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | Require a `to` target on RouterLink and NuxtLink components |
| [`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Require `href` on Void Vue Link components |
| [`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Validate static Void Vue Link method props |
| [`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | Report static vue-i18n keys that are absent from local SFC messages |
| [`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | Prefer named route objects over static path strings in RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Prefer named route objects for Vue Router programmatic navigation |
| [`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | Avoid snapshotting wrapper.html() in Vue Test Utils tests |
| [`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Disallow setting `test` key in Nuxt config |
| [`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup |
| [`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Prefer recommended order of Nuxt config properties |
| [`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Prefer using `import.meta.*` over `process.*` |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For the four typed Vue Router diagnostics, see the [complete project examples](./cross-file.md).
