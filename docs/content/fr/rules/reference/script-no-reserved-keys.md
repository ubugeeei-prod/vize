---
title: "script/no-reserved-keys"
description: "Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API."
---

# `script/no-reserved-keys`

Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

La clé de données renvoyée `$el` entre en conflit avec une propriété intégrée de l’instance du composant Vue et utilise également le préfixe réservé `$`.

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

## Bon

Renommer les données de l’application en `elementLabel` évite l’API intégrée de l’instance et le préfixe réservé.

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-reserved-keys": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30)
