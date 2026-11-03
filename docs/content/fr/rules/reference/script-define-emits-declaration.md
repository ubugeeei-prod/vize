---
title: "script/define-emits-declaration"
description: "Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau."
---

# `script/define-emits-declaration`

Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`defineEmits(["change"])` utilise une déclaration par tableau à l’exécution ; cette règle de style préfère une déclaration typée.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

## Bon

`defineEmits<{ change: [id: number] }>()` place la déclaration de l’événement dans un argument de type et décrit explicitement la charge utile numérique utilisée par `emit("change", 1)`.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
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
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39)
