---
title: "ecosystem/vue-router-prefer-named-push"
description: "Préférer les objets de routes nommées pour la navigation programmatique de Vue Router."
---

# `ecosystem/vue-router-prefer-named-push`

Préférer les objets de routes nommées pour la navigation programmatique de Vue Router.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

router.push reçoit une chaîne de chemin liée à la graphie actuelle de l’URL.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

## Bon

router.push reçoit un objet de route possédant le nom stable settings.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-router-prefer-named-push": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de l’écosystème et de Nuxt](../ecosystem.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19)
