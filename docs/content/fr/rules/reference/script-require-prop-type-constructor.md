---
title: "script/require-prop-type-constructor"
description: "Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales."
---

# `script/require-prop-type-constructor`

Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Les déclarations de props utilisent les chaînes `"String"` et `"Number"` comme types à l’exécution, y compris dans le tableau de constructeurs. Ces chaînes ne sont pas des fonctions constructeurs.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    // The type should be the `String` constructor, not the string "String".
    name: "String",
    age: { type: "Number" },
    id: { type: ["String", "Number"] }
  }
}
</script>
```

## Bon

Les déclarations utilisent les véritables identifiants `String` et `Number`, y compris dans le tableau d’union `[String, Number]`.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: String,
    age: { type: Number },
    id: { type: [String, Number] }
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
      "script/require-prop-type-constructor": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L66)
