---
title: "script/require-define-component"
description: "Exiger defineComponent() pour les exports par défaut des SFC TypeScript."
---

# `script/require-define-component`

Exiger defineComponent() pour les exports par défaut des SFC TypeScript.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La SFC TypeScript exporte directement un objet Options API ; ses options ne reçoivent donc pas les types contextuels fournis par `defineComponent()`.

```vue annotate="remove:2,7"
<script lang="ts">
export default {
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
}
</script>
```

## Bon

Envelopper l’objet avec `defineComponent()` applique la vérification TypeScript adaptée à Vue aux props, aux événements, à `this` et aux autres options.

```vue annotate="add:2,3,8"
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
})
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
        "script/require-define-component": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_define_component.rs#L48)
