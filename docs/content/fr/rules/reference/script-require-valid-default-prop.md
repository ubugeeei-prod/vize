---
title: "script/require-valid-default-prop"
description: "Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré."
---

# `script/require-valid-default-prop`

Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les props Number et Boolean reçoivent des valeurs scalaires par défaut incompatibles, et les props Array et Object utilisent des valeurs littérales partagées au lieu de fonctions de création.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: '0' },     // string default for Number
    enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
    items: { type: Array, default: [] },        // literal must be a factory
    config: { type: Object, default: {} }       // literal must be a factory
  }
}
</script>
```

## Bon

Les valeurs scalaires par défaut deviennent `0` et `false` ; les valeurs par défaut du tableau et de l’objet deviennent des fonctions renvoyant de nouvelles valeurs. L’exemple `[String, Number]` accepte sa valeur par défaut de type chaîne, car elle correspond à l’un des types déclarés.

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
  props: {
    count: { type: Number, default: 0 },
    enabled: { type: Boolean, default: false },
    items: { type: Array, default: () => [] },
    config: { type: Object, default: () => ({}) },
    label: { type: [String, Number], default: '' }
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
      "script/require-valid-default-prop": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L73)
