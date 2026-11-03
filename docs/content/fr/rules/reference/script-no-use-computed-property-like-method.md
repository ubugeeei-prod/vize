---
title: "script/no-use-computed-property-like-method"
description: "Interdire d’appeler une propriété calculée de l’Options API comme une méthode."
---

# `script/no-use-computed-property-like-method`

Interdire d’appeler une propriété calculée de l’Options API comme une méthode.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`this.total()` appelle la valeur exposée par le getter calculé ; ce getter renvoie `3`, qui ne peut pas être appelé.

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

## Bon

`this.total` lit la valeur calculée sans parenthèses d’appel, de sorte que `log` affiche le nombre dérivé.

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
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
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44)
