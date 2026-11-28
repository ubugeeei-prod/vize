---
title: "vue/component-definition-name-casing"
description: "Imposer PascalCase ou kebab-case aux noms de définition des composants."
---

# `vue/component-definition-name-casing`

Imposer PascalCase ou kebab-case aux noms de définition des composants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrôle le nom statique d’une définition de composant : l’option name, defineOptions({ name }) et le premier argument de app.component(). Le nom de fichier relève de vue/component-file-name-casing.

## Mauvais

Le nom my_component déclaré avec defineOptions sépare ses mots par un tiret bas au lieu d’utiliser PascalCase.

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "my_component" });
</script>
<template><p>Content</p></template>
```

## Bon

Le nom déclaré MyComponent utilise PascalCase, la casse par défaut.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
<template><p>Content</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/component-definition-name-casing": "PascalCase"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L68)
