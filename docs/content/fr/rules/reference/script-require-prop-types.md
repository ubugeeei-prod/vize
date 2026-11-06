---
title: "script/require-prop-types"
description: "Exiger que chaque prop déclare un type."
---

# `script/require-prop-types`

Exiger que chaque prop déclare un type.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’entrée du tableau ne déclare que le nom `status` ; la valeur `null` et le descripteur vide ne déclarent pas non plus de type de prop à l’exécution.

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
  props: ['status']            // array form: no types
}

export default {
  props: {
    status: null,              // no type
    other: {}                  // empty descriptor: no type
  }
}
</script>
```

## Bon

`status: String` fournit un constructeur sous forme abrégée, et `other` fournit `type: Number` dans son descripteur. Les deux props possèdent désormais des déclarations de type.

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    status: String,
    other: { type: Number, default: 0 }
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
      "script/require-prop-types": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58)
