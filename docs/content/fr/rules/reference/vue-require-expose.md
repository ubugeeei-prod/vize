---
title: "vue/require-expose"
description: "Exige que les composants de l'Options API déclarent explicitement leurs propriétés publiques avec `expose`."
---

# `vue/require-expose`

Exige que les composants de l'Options API déclarent explicitement leurs propriétés publiques avec `expose`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup.

Ce contrôle concerne les objets de composants de l'Options API. Une liste `expose` explicitement vide est valide. Un appel à `expose` dans setup ou le retour d'une fonction de rendu satisfait également le contrôle ; les mixins sont exclus. Cette passe sur le script n'inspecte pas un bloc `<script setup>` voisin, et `defineExpose` n'est pas la cible de cette règle.

## Mauvais

Le composant de l'Options API possède une méthode `focus` sans déclarer explicitement l'interface de son instance publique.

```vue annotate="remove:2"
<script lang="ts">
export default { methods: { focus() {} } };
</script>
```

## Bon

Une liste `expose` rend explicitement `focus` accessible par l'instance publique du composant.

```vue annotate="add:2"
<script lang="ts">
export default { expose: ["focus"], methods: { focus() {} } };
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
        "vue/require-expose": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_expose.rs#L24)
