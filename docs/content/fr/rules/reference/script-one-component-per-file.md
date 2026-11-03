---
title: "script/one-component-per-file"
description: "Interdire plus d’une définition de composant dans un fichier."
---

# `script/one-component-per-file`

Interdire plus d’une définition de composant dans un fichier.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le fichier exporte `Parent` et définit aussi un composant `Child` dans `components`.

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  name: 'Parent',
  components: {
    Child: { name: 'Child' }
  }
}
</script>
```

## Bon

Le fichier ne définit que `Parent`.

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'Parent'
}
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
        "script/one-component-per-file": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L30)
