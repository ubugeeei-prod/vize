---
title: "type/strict-boolean-expressions"
description: "Exiger des expressions booléennes sûres dans les conditions des scripts et des templates."
---

# `type/strict-boolean-expressions`

Exiger des expressions booléennes sûres dans les conditions des scripts et des templates.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Activez explicitement typeAware et cette règle. Par défaut, les nombres pouvant être nuls sont interdits, tandis que les nombres non nuls sont autorisés.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

`if (count)` repose sur la conversion implicite en booléen d’une liaison numérique nullable plutôt que sur un test booléen explicite ; cela confond aussi zéro avec l’absence de valeur.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## Bon

`count !== undefined && count > 0` teste séparément la présence et la positivité, produisant une condition booléenne explicite après avoir affiné le type de la valeur optionnelle.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
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
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7)
