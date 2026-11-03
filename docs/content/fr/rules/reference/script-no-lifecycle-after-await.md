---
title: "script/no-lifecycle-after-await"
description: "Interdire l’enregistrement de hooks de cycle de vie après un await dans setup()."
---

# `script/no-lifecycle-after-await`

Interdire l’enregistrement de hooks de cycle de vie après un await dans setup().

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`setup()` appelle `onMounted()` après `await loadData()` ; sans instance de composant courante à ce moment, le hook n’est jamais enregistré.

```vue annotate="remove:5"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    await loadData()
    onMounted(() => {})
  },
}
</script>
```

## Bon

`onMounted()` est enregistré avant le premier `await`, tant que `setup()` s’exécute de manière synchrone pour le composant.

```vue annotate="add:6"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    onMounted(() => {})
    await loadData()
  },
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
        "script/no-lifecycle-after-await": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45)
