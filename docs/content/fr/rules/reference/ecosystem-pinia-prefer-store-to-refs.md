---
title: "ecosystem/pinia-prefer-store-to-refs"
description: "Préférer storeToRefs() lors de la déstructuration des stores Pinia."
---

# `ecosystem/pinia-prefer-store-to-refs`

Préférer storeToRefs() lors de la déstructuration des stores Pinia.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Déstructurer `name` directement depuis le store sépare la valeur de son accès réactif au store.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

## Bon

Le store reste intact et storeToRefs crée une référence réactive pour name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/pinia-prefer-store-to-refs": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20)
