---
title: "script/no-watch-after-await"
description: "Interdire les observateurs créés après un await dans setup() sans conserver leur fonction d’arrêt."
---

# `script/no-watch-after-await`

Interdire les observateurs créés après un await dans setup() sans conserver leur fonction d’arrêt.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`watch()` est créé après `await loadData()` et sa fonction d’arrêt est ignorée ; l’observateur n’est donc jamais arrêté avec le composant.

```vue annotate="remove:5"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    await loadData()
    watch(() => props.id, reload)
  },
}
</script>
```

## Bon

L’observateur est créé avant le premier `await`, donc Vue l’arrête lorsque le composant est démonté.

```vue annotate="add:6"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    watch(() => props.id, reload)
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
        "script/no-watch-after-await": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44)
