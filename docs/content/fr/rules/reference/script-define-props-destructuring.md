---
title: "script/define-props-destructuring"
description: "Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;."
---

# `script/define-props-destructuring`

Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineProps` est affecté à la seule liaison `props` au lieu d’être déstructuré, contrairement à la préférence par défaut pour la déstructuration.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## Bon

Le motif objet lie directement `foo` et `bar` et donne une valeur par défaut à `bar`, qui est facultatif. Cela repose sur la déstructuration réactive des props de Vue 3.5+ ; le mode configurable `never` préfère la forme opposée.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-props-destructuring": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29)
