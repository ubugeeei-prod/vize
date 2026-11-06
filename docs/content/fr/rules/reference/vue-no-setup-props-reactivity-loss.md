---
title: "vue/no-setup-props-reactivity-loss"
description: "Interdit les copies des valeurs des props de setup qui perdent leurs mises à jour réactives."
---

# `vue/no-setup-props-reactivity-loss`

Interdit les copies des valeurs des props de setup qui perdent leurs mises à jour réactives.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Contrôle les copies de props dans la portée racine de `setup` de l'Options API et de `<script setup>`. La déstructuration réactive des props de Vue 3.5 reste autorisée ; une copie ultérieure d'une prop déstructurée à la racine du script est signalée. Les lectures dans une autre fonction sont autorisées.

## Mauvais

Copier `props.count` à la racine du script conserve sa valeur actuelle et ne suit pas les mises à jour ultérieures de la prop.

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps({ count: Number });
const snapshot = props.count;
</script>
```

## Bon

Lisez `props.count` dans un getter calculé pour que le résultat dérivé continue de suivre les changements de la prop.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps({ count: Number });
const snapshot = computed(() => props.count);
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
        "vue/no-setup-props-reactivity-loss": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss.rs#L44)
