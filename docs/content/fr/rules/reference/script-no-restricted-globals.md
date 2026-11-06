---
title: "script/no-restricted-globals"
description: "Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée."
---

# `script/no-restricted-globals`

Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

## Mauvais

L’exemple lit directement les variables globales restreintes par défaut `process`, `localStorage` et `sessionStorage`, en contournant les utilitaires explicites de configuration et de stockage du projet.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

## Bon

`useFeatureFlag`, `authStorage.read` et `viewStorage.write` suppriment ces références directes aux variables globales restreintes. L’accès restant à `window.scrollY` ne fait pas partie des restrictions par défaut de cette règle ; la sécurité SSR est une question distincte.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-restricted-globals": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57)
