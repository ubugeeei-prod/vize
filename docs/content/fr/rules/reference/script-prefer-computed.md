---
title: "script/prefer-computed"
description: "Préférer computed() pour l’état réactif dérivé."
---

# `script/prefer-computed`

Préférer computed() pour l’état réactif dérivé.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Le watcher doit uniquement dériver la destination. Les copies modifiables et les callbacks ayant d’autres effets de bord sont autorisés.

## Mauvais

Le watcher ne fait que copier une valeur dérivée de `count` dans une seconde ref, `doubled` ; l’état dérivé est donc maintenu par synchronisation manuelle.

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

## Bon

`computed(() => count.value * 2)` exprime directement la dérivation et supprime à la fois la ref modifiable supplémentaire et le watcher qui la synchronise.

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
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
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41)
