---
title: "script/valid-define-props"
description: "Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales)."
---

# `script/valid-define-props`

Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le même appel `defineProps` fournit à la fois `{ title: string }` comme argument de type et `{ title: String }` comme argument d’exécution, ce que le compilateur n’autorise pas conjointement.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

## Bon

Supprimer l’objet d’exécution laisse une déclaration typée unique pour `title` au lieu de combiner les deux formes de déclaration.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-props": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L46)
