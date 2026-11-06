---
title: "script/require-typed-ref"
description: "Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined."
---

# `script/require-typed-ref`

Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les appels à `ref` importé n’ont ni argument de type ni valeur initiale utile : l’absence d’argument, `null` et `undefined` ne permettent pas d’inférer le type de valeur futur souhaité.

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

## Bon

Des arguments de type explicites décrivent les refs de chaîne et de User nullable. `ref(0)` possède déjà une valeur initiale numérique concrète et peut s’appuyer sur l’inférence.

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-typed-ref": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55)
