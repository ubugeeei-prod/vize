---
title: Importações automáticas
---

<!-- Generated translation; source: guide/auto-imports.md -->

# Importações automáticas

`@vizejs/ui` e `@vizejs/composable` incluem resolvers guiados pelos catálogos. Cada nome resolve para o subcaminho direto (por exemplo `@vizejs/ui/dialog`), então o tree-shaking é igual ao de imports escritos à mão. No Nuxt, ative `vize.ui` e `vize.composables`; no Vite, use `@vizejs/ui/resolver` e `@vizejs/composable/resolver`. Com `source: "local"`, as cópias de `vize lib pull` (`vize-lib.lock.json`) têm prioridade.

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: { ui: { prefix: "Vz" }, composables: true },
});
```

```ts
// vite.config.ts
Components({ resolvers: [VizeUiResolver({ prefix: "Vz" })] });
AutoImport({ resolvers: [VizeUiComposablesResolver(), VizeComposableResolver()] });
```
