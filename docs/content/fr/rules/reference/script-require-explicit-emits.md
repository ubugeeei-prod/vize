---
title: "script/require-explicit-emits"
description: "Exiger la déclaration des événements émis dans defineEmits ou l’option emits."
---

# `script/require-explicit-emits`

Exiger la déclaration des événements émis dans defineEmits ou l’option emits.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La fonction emit récupérée émet `save`, mais `defineEmits([])` ne déclare pas cet événement.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

## Bon

Ajouter `"save"` à la déclaration intègre l’événement littéral émis au contrat explicite d’événements du composant.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-explicit-emits": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L53)
