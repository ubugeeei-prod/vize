---
title: "vue/v-if-else-key"
description: "Exige une key sur les composants conditionnels répétés."
---

# `vue/v-if-else-key`

Exige une key sur les composants conditionnels répétés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les instances conditionnelles répétées de `Panel` n'ont pas de key.

```vue annotate="remove:2"
<template>
  <section><Panel v-if="ready" /><Panel v-else /></section>
</template>
```

## Bon

Chaque composant conditionnel possède une key stable et unique.

```vue annotate="add:2"
<template>
  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-if-else-key": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/v_if_else_key.rs#L9)
