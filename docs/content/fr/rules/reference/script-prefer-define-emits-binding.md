---
title: "script/prefer-define-emits-binding"
description: "Préférer la fonction capturée de defineEmits au $emit du template."
---

# `script/prefer-define-emits-binding`

Préférer la fonction capturée de defineEmits au $emit du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Oui | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le template appelle le helper `$emit` du composant alors que `<script setup>` capture déjà la fonction typée renvoyée par `defineEmits`.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## Bon

Le template appelle la liaison `emit` capturée, de sorte que les noms d’événements et les payloads passent par le contrat `defineEmits` du composant.

```vue annotate="add:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-define-emits-binding": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16)
