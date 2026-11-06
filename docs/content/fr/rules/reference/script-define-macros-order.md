---
title: "script/define-macros-order"
description: "Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;."
---

# `script/define-macros-order`

Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineEmits` apparaît avant `defineProps`, alors que `defineProps` puis `defineEmits` doivent être les premières instructions de `<script setup>`.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## Bon

`defineProps` puis `defineEmits` suivent directement les imports, avant toute autre instruction ; les autres macros peuvent suivre dans n’importe quel ordre.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-macros-order": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44)
