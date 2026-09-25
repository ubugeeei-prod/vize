---
title: 自动导入
---

<!-- Generated translation; source: guide/auto-imports.md -->

# 自动导入

`@vizejs/ui` 和 `@vizejs/composable` 提供由目录驱动的解析器。每个名称都解析到其直接子路径（如 `@vizejs/ui/dialog`），因此摇树优化与手写 import 相同。在 Nuxt 中通过 `vize.ui` 与 `vize.composables` 选择启用；在 Vite 中使用 `@vizejs/ui/resolver` 和 `@vizejs/composable/resolver`。设置 `source: "local"` 可解析到通过 `vize lib pull` 拉取的本地源码（`vize-lib.lock.json`）。

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
