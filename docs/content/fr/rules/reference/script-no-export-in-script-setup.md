---
title: "script/no-export-in-script-setup"
description: "Interdire les instructions export dans &lt;script setup&gt;."
---

# `script/no-export-in-script-setup`

Interdire les instructions export dans &lt;script setup&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`export const count` tente d’exposer un export de module depuis `<script setup>`, où les exports à l’exécution sont interdits.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

## Bon

Supprimer `export` conserve `count` comme liaison setup plutôt que comme export de module.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49)
