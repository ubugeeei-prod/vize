---
title: "script/no-with-defaults"
description: "Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+)."
---

# `script/no-with-defaults`

Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`withDefaults` enveloppe la déclaration typée des props uniquement pour fournir les valeurs par défaut de `count` et `name`, au lieu du style de valeurs par défaut dans la déstructuration de Vue 3.5+ préféré ici.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## Bon

Le motif de déstructuration place `count = 0` et `name = "Ada"` à côté de leurs liaisons et supprime l’enveloppe `withDefaults`.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
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
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41)
