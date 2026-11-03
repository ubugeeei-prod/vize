---
title: "script/prefer-ref-over-reactive"
description: "Recommander ref() plutôt que reactive() pour gérer l’état."
---

# `script/prefer-ref-over-reactive`

Recommander ref() plutôt que reactive() pour gérer l’état.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’état est créé avec `reactive`, contrairement à la préférence de cette règle de convention pour les refs. L’exemple illustre une préférence de style ; il ne s’agit pas d’un objet réactif intrinsèquement invalide.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
  count: 0,
  name: 'foo'
})
</script>
```

## Bon

Les exemples créent aussi bien l’état scalaire que l’état objet avec `ref` ; les champs liés peuvent également être répartis dans des refs distinctes. Cela respecte la forme de création d’état préférée.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44)
