---
title: Imports automatiques
---

<!-- Generated translation; source: guide/auto-imports.md -->

# Imports automatiques

`@vizejs/ui` et `@vizejs/composable` fournissent des résolveurs pilotés par les catalogues. Chaque nom se résout vers son sous-chemin direct (par exemple `@vizejs/ui/dialog`), donc le tree-shaking est identique aux imports écrits à la main. Avec Nuxt, activez `vize.ui` et `vize.composables` ; avec Vite, utilisez `@vizejs/ui/resolver` et `@vizejs/composable/resolver`. Avec `source: "local"`, les copies de `vize lib pull` (`vize-lib.lock.json`) sont prioritaires.

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
