---
title: "vue/prefer-v-model"
description: "Préfère v-model à une paire bind et update équivalente."
---

# `vue/prefer-v-model`

Préfère v-model à une paire bind et update équivalente.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bind et le listener synchronisent manuellement la même valeur.

```vue annotate="remove:2"
<template>
  <TextField :model-value="value" @update:model-value="value = $event" />
</template>
```

## Bon

`v-model` exprime une fois le même contrat bidirectionnel.

```vue annotate="add:2"
<template>
  <TextField v-model="value" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-v-model": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/prefer_v_model.rs#L19)
