---
title: "script/no-unused-emit-declarations"
description: "Signaler les événements déclarés qui ne sont jamais émis."
---

# `script/no-unused-emit-declarations`

Signaler les événements déclarés qui ne sont jamais émis.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineEmits` déclare à la fois `change` et `unused`, mais la fonction `emit` récupérée n’émet que l’événement littéral `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## Bon

Supprimer `unused` fait correspondre la liste des événements déclarés à l’émission observée. L’exemple utilise une liaison emit récupérée et non transmise à l’extérieur, ce qui permet cette conclusion sur son utilisation locale.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-unused-emit-declarations": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L82)
