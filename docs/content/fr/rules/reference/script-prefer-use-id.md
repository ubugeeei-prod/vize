---
title: "script/prefer-use-id"
description: "Recommander useId() pour générer des identifiants uniques (Vue 3.5+)."
---

# `script/prefer-use-id`

Recommander useId() pour générer des identifiants uniques (Vue 3.5+).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`id` contient `Math.random()` : l’identifiant généré pour l’input et le label peut donc différer entre les rendus serveur et client. Sa liaison dont le nom désigne un ID est le contexte de génération reconnu par la règle.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Bon

`useId()` de Vue 3.5+ génère l’identifiant, et `:for` comme `:id` continuent de lire la même liaison au lieu de générer indépendamment des valeurs aléatoires.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-id": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45)
