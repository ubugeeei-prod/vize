---
title: "script/no-options-api"
description: "Interdire les formes de l’Options API en mode Vapor."
---

# `script/no-options-api`

Interdire les formes de l’Options API en mode Vapor.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | Non | Aucune |

Champ d’application : Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires.

## Mauvais

L’objet exporté par défaut déclare `data()` de l’Options API, une forme d’option de composant interdite par cette règle.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## Bon

L’état du composant devient une `ref` de la Composition API dans le `<script setup>` Vapor, supprimant l’objet de l’Options API et son option `data`.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-options-api": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43)
