---
title: "script/order-in-components"
description: "Imposer l’ordre par défaut des propriétés des options de composant."
---

# `script/order-in-components`

Imposer l’ordre par défaut des propriétés des options de composant.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`data` est écrit avant `props`, donc un groupe plus tardif précède un groupe plus précoce.

```vue annotate="remove:5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  props: ['label']
}
</script>
```

## Bon

`props` précède `data`, ce qui suit l’ordre des groupes par défaut.

```vue annotate="add:3,6"
<script lang="ts">
export default {
  props: ['label'],
  data() {
    return { count: 0 }
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
      "script/order-in-components": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/order_in_components.rs#L35)
