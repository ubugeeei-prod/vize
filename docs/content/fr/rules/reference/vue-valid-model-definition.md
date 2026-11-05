---
title: "vue/valid-model-definition"
description: "Exige des clés valides dans l'option `model` de Vue 2."
---

# `vue/valid-model-definition`

Exige des clés valides dans l'option `model` de Vue 2.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Cette règle à activer explicitement contrôle l'option `model` de Vue 2. Seules les clés statiques `prop` et `event` sont acceptées ; leurs valeurs ne sont pas validées. Vue 3 utilise le contrat plus récent de `v-model` des composants à la place de cette option.

## Mauvais

`events` n'est pas une clé prise en charge dans l'option `model` des composants Vue 2.

```vue annotate="remove:2"
<script lang="ts">
export default { model: { prop: "value", events: "input" } };
</script>
```

## Bon

Utilisez la clé prise en charge au singulier, `event`, avec `prop`, pour décrire le contrat de model de Vue 2.

```vue annotate="add:2"
<script lang="ts">
export default { model: { prop: "value", event: "input" } };
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
        "vue/valid-model-definition": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_model_definition.rs#L21)
