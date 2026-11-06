---
title: "script/no-deprecated-destroyed-lifecycle"
description: "Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy."
---

# `script/no-deprecated-destroyed-lifecycle`

Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Oui | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

`beforeDestroy` est l’option de cycle de vie de Vue 2 supprimée, utilisée ici pour nettoyer le minuteur.

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

## Bon

Renommer le hook en `beforeUnmount` conserve le corps du nettoyage sous son nom de cycle de vie Vue 3.

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-destroyed-lifecycle": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17)
