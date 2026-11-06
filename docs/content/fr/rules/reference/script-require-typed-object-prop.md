---
title: "script/require-typed-object-prop"
description: "Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`."
---

# `script/require-typed-object-prop`

Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les constructeurs `Object` et `Array` employés seuls ne décrivent que des catégories générales à l’exécution ; ni `user` ni la structure des éléments de `items` n’a donc de type statique explicite.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## Bon

`PropType<User>` et `PropType<User[]>` ajoutent les types de l’objet et des éléments tout en conservant les mêmes constructeurs à l’exécution.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-typed-object-prop": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63)
