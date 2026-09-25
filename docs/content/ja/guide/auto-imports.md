---
title: 自動インポート
---

<!-- Generated translation; source: guide/auto-imports.md -->

# 自動インポート

`@vizejs/ui` と `@vizejs/composable` はカタログ駆動のリゾルバーを提供します。すべての名前は直接のサブパス（`@vizejs/ui/dialog` など）に解決されるため、ツリーシェイキングは手書きの import と同じです。Nuxt では `vize.ui` と `vize.composables` をオプトインで有効化し、Vite では `@vizejs/ui/resolver` と `@vizejs/composable/resolver` のリゾルバーを使います。`source: "local"` を指定すると `vize lib pull` でコピーしたソース（`vize-lib.lock.json`）に解決します。

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
