---
title: "script/no-multiple-slot-args"
description: "Interdire de passer plusieurs arguments à un appel de fonction de slot à portée."
---

# `script/no-multiple-slot-args`

Interdire de passer plusieurs arguments à un appel de fonction de slot à portée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les appels de slots passent plusieurs arguments positionnels ou déploient une liste d’arguments inconnue. Les slots Vue reçoivent un seul objet de props, et non une liste de paramètres positionnels.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

## Bon

`{ foo, bar }` regroupe les données dans un seul argument ; `slotProps` et l’appel sans argument respectent également la forme d’appel de slot prise en charge.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-multiple-slot-args": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61)
