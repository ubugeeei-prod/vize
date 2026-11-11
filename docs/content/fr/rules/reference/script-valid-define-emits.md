---
title: "script/valid-define-emits"
description: "Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel)."
---

# `script/valid-define-emits`

Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

Le même appel `defineEmits` fournit à la fois un argument de type et le tableau à l’exécution `["save"]`, mélangeant deux déclarations mutuellement exclusives.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## Bon

Supprimer l’argument d’exécution laisse une seule déclaration d’événement typée pour `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-define-emits": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L47)
