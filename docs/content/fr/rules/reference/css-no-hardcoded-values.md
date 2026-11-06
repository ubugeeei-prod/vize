---
title: "css/no-hardcoded-values"
description: "Suggérer des variables CSS plutôt que des valeurs codées en dur."
---

# `css/no-hardcoded-values`

Suggérer des variables CSS plutôt que des valeurs codées en dur.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

Le bouton place directement des nombres d’espacement et une couleur hexadécimale dans les déclarations.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

## Bon

Les déclarations référencent des propriétés personnalisées d’espacement et de couleur nommées, afin que ces valeurs puissent être maintenues sous forme de tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
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
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles CSS](../css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30)
