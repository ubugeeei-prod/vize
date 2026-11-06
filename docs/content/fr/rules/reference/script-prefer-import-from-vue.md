---
title: "script/prefer-import-from-vue"
description: "Préférer les imports depuis 'vue' plutôt que depuis les packages internes."
---

# `script/prefer-import-from-vue`

Préférer les imports depuis 'vue' plutôt que depuis les packages internes.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Oui | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`ref` et `h` sont importés depuis les packages internes `@vue/runtime-core` et `@vue/runtime-dom` plutôt que depuis le package public `vue`.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## Bon

Les utilitaires publics sont importés depuis `vue`. Le privé `isString` conserve `@vue/shared`, car `vue` ne l’exporte pas. Les imports de namespace, par défaut et mixtes publics/privés gardent un avertissement sans correction automatique.

```vue annotate="add:2,3"
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-import-from-vue": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36)
