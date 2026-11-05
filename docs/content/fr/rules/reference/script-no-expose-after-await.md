---
title: "script/no-expose-after-await"
description: "Interdire expose() et defineExpose() après un await."
---

# `script/no-expose-after-await`

Interdire expose() et defineExpose() après un await.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineExpose()` s’exécute après un `await` de premier niveau, alors qu’aucune instance de composant n’est courante ; rien n’est donc exposé.

```vue annotate="remove:2"
<script setup lang="ts">
await loadData()
defineExpose({ reload })
</script>
```

## Bon

`defineExpose()` s’exécute avant le premier `await`, tant que l’instance du composant est courante.

```vue annotate="add:3"
<script setup lang="ts">
defineExpose({ reload })
await loadData()
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
        "script/no-expose-after-await": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_expose_after_await.rs#L34)
