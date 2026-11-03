---
title: "script/no-deep-destructure-in-props"
description: "Interdire la déstructuration profondément imbriquée dans defineProps."
---

# `script/no-deep-destructure-in-props`

Interdire la déstructuration profondément imbriquée dans defineProps.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le motif de liaison descend dans `user` pour déstructurer `name`, dépassant la profondeur limitée de déstructuration des props autorisée par défaut.

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

## Bon

L’objet des props reste intact, et un getter calculé lit `props.user.name`. L’accès imbriqué reste explicite sans motif de liaison profondément imbriqué.

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
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
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39)
