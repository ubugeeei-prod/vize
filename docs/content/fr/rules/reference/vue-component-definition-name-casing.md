---
title: "vue/component-definition-name-casing"
description: "Imposer PascalCase ou kebab-case aux noms de définition des composants."
---

# `vue/component-definition-name-casing`

Imposer PascalCase ou kebab-case aux noms de définition des composants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Le nom de fichier du composant est contrôlé. PascalCase et kebab-case sont acceptés ; une casse mixte est signalée.

## Mauvais

Le nom de fichier myComponent.vue combine une initiale minuscule et une majuscule interne au lieu d’utiliser PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## Bon

Renommer le fichier en MyComponent.vue adopte PascalCase ; le contenu de son template reste inchangé.

`MyComponent.vue`

```vue
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
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36)
