---
title: "script/no-get-current-instance"
description: "Interdire getCurrentInstance() en mode Vapor (renvoie null)."
---

# `script/no-get-current-instance`

Interdire getCurrentInstance() en mode Vapor (renvoie null).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `opinionated` | Non | Aucune |

Champ d’application : Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires.

## Mauvais

Le setup marqué Vapor importe et appelle `getCurrentInstance`, s’appuyant sur une API d’instance que cette règle interdit pour les composants destinés à Vapor.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## Bon

`inject("app-config")` obtient la configuration explicitement fournie sans importer ni appeler `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-get-current-instance": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38)
