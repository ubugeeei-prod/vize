---
title: "script/no-top-level-ref-in-script"
description: "Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes."
---

# `script/no-top-level-ref-in-script`

Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le `<script>` ordinaire initialise `count` et `user` dans la portée du module. Pendant le SSR, ces objets d’état peuvent être partagés entre les instances du composant et les requêtes.

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
  setup() {
    return { count, user }
  }
}
</script>
```

## Bon

La ref de setup est initialisée pour chaque instance du composant ; le script ordinaire ne conserve qu’une constante, une fonction produisant de l’état et une ref créée dans `setup()`. Aucun ne crée d’état réactif dans la portée du module ordinaire.

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
  return reactive({ count: 0 })
}

export default {
  setup() {
    // Create state inside setup
    const count = ref(0)
    return { count }
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-top-level-ref-in-script": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63)
