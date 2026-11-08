---
title: "script/require-explicit-slots"
description: "Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;()."
---

# `script/require-explicit-slots`

Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;().

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La déclaration typée `defineProps<{ id: number }>()` établit une syntaxe TypeScript, mais setup utilise `useSlots()` sans déclaration `defineSlots`. La règle détecte donc des slots utilisés sans contrat de slots explicite.

```vue annotate="remove:2,4,5,6"
<script setup lang="ts">
defineSlots<{ default(): unknown }>()
</script>
<template>
  <div><slot /><slot name="footer" /></div>
</template>
```

## Bon

`defineSlots` déclare un slot `default` dont les props incluent `msg: string` ; `useSlots()` apparaît désormais aux côtés d’un contrat de slots explicitement typé.

```vue annotate="add:2,3"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-slots": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L57)
