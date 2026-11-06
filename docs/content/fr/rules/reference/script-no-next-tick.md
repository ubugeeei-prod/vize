---
title: "script/no-next-tick"
description: "Interdire l’utilisation de nextTick() dans les composants destinés à Vapor."
---

# `script/no-next-tick`

Interdire l’utilisation de nextTick() dans les composants destinés à Vapor.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires.

## Mauvais

Le composant destiné à Vapor importe `nextTick` et attend son résultat, introduisant la dépendance à la planification des mises à jour du DOM que cette règle de migration rejette.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## Bon

L’input est obtenu par `useTemplateRef` et reçoit le focus dans `onMounted`. Ce point de montage explicite remplace la dépendance de l’exemple à `nextTick`.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40)
