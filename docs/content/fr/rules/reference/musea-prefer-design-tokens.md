---
title: "musea/prefer-design-tokens"
description: "Préférer les variables CSS de design tokens aux valeurs primitives codées en dur."
---

# `musea/prefer-design-tokens`

Préférer les variables CSS de design tokens aux valeurs primitives codées en dur.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | [Options des règles](/rules/options.md) |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

Nécessite un fichier .art.vue et l’inventaire des tokens présenté ci-dessous. Aucun token n’est déduit d’une couleur arbitraire.

## Mauvais

L’exemple art utilise la couleur bleue littérale au lieu du design token primaire configuré.

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

## Bon

Le style référence --color-primary, le token configuré pour cet exemple.

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea](../musea.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32)
