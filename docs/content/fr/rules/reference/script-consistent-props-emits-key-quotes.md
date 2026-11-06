---
title: "script/consistent-props-emits-key-quotes"
description: "Uniformiser les clés entre guillemets dans les déclarations props et emits."
---

# `script/consistent-props-emits-key-quotes`

Uniformiser les clés entre guillemets dans les déclarations props et emits.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Compatibilité avec les formateurs

Cette règle de lint gère les guillemets propres à chaque déclaration. Configurez Prettier, Oxfmt et le formateur Vize avec `quoteProps: "preserve"` afin que le formatage ne retire pas les guillemets requis par la règle.

## Mauvais

La déclaration emits mélange la clé `update:modelValue`, qui doit être entre guillemets, avec la clé `close` sans guillemets.

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()
</script>
```

## Bon

Toutes les clés statiques de la déclaration sont entre guillemets, car `update:modelValue` ne peut pas être écrit comme identifiant JavaScript sans guillemets.

```vue annotate="add:4,6,7,8,9"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()
const props = defineProps({
  "aria-label": String,
  "disabled": Boolean,
})
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-props-emits-key-quotes": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/consistent_key_quotes.rs#L53)
