---
title: "script/no-restricted-members"
description: "Interdire les accès aux membres object.property configurés par le projet."
---

# `script/no-restricted-members`

Interdire les accès aux membres object.property configurés par le projet.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Cet exemple configure window.localStorage. La règle n’a aucune liste d’interdiction par défaut ; son activation seule ne signale pas de membre.

## Mauvais

Lorsque `{ object: "window", property: "localStorage" }` est configuré dans `ruleOptions`, `window.localStorage` accède à la paire objet/membre interdite. Cette règle n’interdit aucun membre par défaut.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## Bon

`authStorage.read("token")` délègue la lecture à l’utilitaire de stockage de l’application et n’accède plus au membre configuré `window.localStorage`.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-members": {
        "members": [
          {
            "object": "window",
            "property": "localStorage"
          }
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53)
