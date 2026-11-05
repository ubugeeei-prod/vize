---
title: "script/no-deprecated-delete-set"
description: "Interdire $set et $delete de Vue 2, retirés dans Vue 3."
---

# `script/no-deprecated-delete-set`

Interdire $set et $delete de Vue 2, retirés dans Vue 3.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`created()` appelle `this.$set`, retiré, pour ajouter `ready`.

```vue annotate="remove:4"
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## Bon

Le même champ est affecté directement avec `this.item.ready = true`.

```vue annotate="add:4"
<script lang="ts">
export default {
  created() {
    this.item.ready = true
  }
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
        "script/no-deprecated-delete-set": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L41)
