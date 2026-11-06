---
title: "vue/prefer-prop-type-boolean-first"
description: "Exige que `Boolean` soit le premier constructeur dans la liste des types d'une prop."
---

# `vue/prefer-prop-type-boolean-first`

Exige que `Boolean` soit le premier constructeur dans la liste des types d'une prop.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Changer l'ordre des constructeurs peut modifier la conversion booléenne à l'exécution. Ce contrôle à activer explicitement couvre `defineProps` à l'exécution et les listes de types de props de l'Options API ; il ne réordonne pas les unions de types TypeScript.

## Mauvais

`String` précède `Boolean` : un attribut `enabled` sans valeur utilise donc une conversion privilégiant les chaînes plutôt que les booléens.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps({ enabled: [String, Boolean] });
</script>
```

## Bon

Placez `Boolean` en premier pour privilégier la conversion booléenne de la prop, tout en conservant la prise en charge des chaînes.

```vue annotate="add:2"
<script setup lang="ts">
defineProps({ enabled: [Boolean, String] });
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-prop-type-boolean-first": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs#L19)
