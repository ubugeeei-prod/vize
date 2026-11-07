---
title: "script/prefer-define-options"
description: "Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs."
---

# `script/prefer-define-options`

Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La seule instruction significative du script ordinaire exporte un objet contenant uniquement `name` et `inheritAttrs` ; ces options peuvent être exprimées par `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## Bon

La méthode `data()` montrée donne au script une véritable logique de l’Options API ; il échappe donc à la suggestion prudente de cette règle, limitée aux options seules. Cet exemple Bon démontre une exception autorisée ; la migration directe placerait `defineOptions({ name: 'MyComponent', inheritAttrs: false })` dans `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
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
      "script/prefer-define-options": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L44)
