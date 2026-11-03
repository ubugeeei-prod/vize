---
title: "script/require-function-return-type"
description: "Exiger des annotations de type de retour sur les fonctions."
---

# `script/require-function-return-type`

Exiger des annotations de type de retour sur les fonctions.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`add` et `greet` annotent tous deux leurs paramètres mais omettent une annotation de type de retour ; les retours inférés ne satisfont pas cette convention d’annotation explicite.

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
  return a + b
}

function greet(name: string) {
  return `Hello, ${name}`
}
</script>
```

## Bon

`add` déclare `: number`, et `greet` déclare `: string`, rendant leurs contrats de retour explicites sans changer leurs corps.

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
  return a + b
}

function greet(name: string): string {
  return `Hello, ${name}`
}
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
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L55)
