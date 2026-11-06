---
title: "vue/no-ref-object-reactivity-loss"
description: "Interdit de copier la valeur d'une ref dans la même portée en perdant ses mises à jour réactives."
---

# `vue/no-ref-object-reactivity-loss`

Interdit de copier la valeur d'une ref dans la même portée en perdant ses mises à jour réactives.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Le contrôle suit les fonctions reconnues qui créent des refs et les copies de valeurs dans la même portée. Les lectures dans une autre fonction et les écritures dans `.value` ne sont pas contrôlées. Ce contrôle à activer explicitement analyse le code source et ne nécessite pas de requête de type native.

## Mauvais

Lire `count.value` dans `snapshot` à la racine du script capture le nombre actuel, sans suivre les mises à jour ultérieures de la ref.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const snapshot = count.value;
</script>
```

## Bon

Un getter calculé lit `count.value` lorsque sa dépendance change, préservant un résultat réactif plutôt qu'une copie dans la portée racine.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed, ref } from "vue";
const count = ref(0);
const snapshot = computed(() => count.value);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-ref-object-reactivity-loss": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss.rs#L39)
