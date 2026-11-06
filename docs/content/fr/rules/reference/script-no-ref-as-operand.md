---
title: "script/no-ref-as-operand"
description: "Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande."
---

# `script/no-ref-as-operand`

Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`count + 1` utilise l’objet ref lui-même comme opérande arithmétique au lieu du nombre qu’il contient.

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

## Bon

`count.value + 1` lit le nombre contenu avant d’ajouter un ; l’arithmétique dans le script exige cet accès explicite à la ref.

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-ref-as-operand": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41)
