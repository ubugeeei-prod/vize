---
title: "script/no-template-ref-outside-instance"
description: "Interdire useTemplateRef() là où aucune instance de composant ne l’exécute."
---

# `script/no-template-ref-outside-instance`

Interdire useTemplateRef() là où aucune instance de composant ne l’exécute.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`focus()` ne s’exécute que comme gestionnaire de clic du bouton, et le callback de `setTimeout` s’exécute une fois setup terminé. Aucun des deux n’a d’instance de composant courante : les refs qu’ils créent ne sont jamais renseignées.

```vue annotate="remove:2,4,6"
<script setup lang="ts">
import { useTemplateRef } from 'vue'
function focus() {
  useTemplateRef<HTMLInputElement>('input').value?.focus()
}
setTimeout(() => useTemplateRef('input'))
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Bon

`useTemplateRef('input')` s’exécute une seule fois dans setup. Le gestionnaire de clic et le callback de `onMounted` se contentent de lire `input.value`.

```vue annotate="add:2,3,5,7"
<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
function focus() {
  input.value?.focus()
}
onMounted(() => focus())
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-template-ref-outside-instance": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_template_ref_outside_instance.rs#L78)
