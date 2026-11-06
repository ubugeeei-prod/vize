---
title: "musea/unique-variant-names"
description: "Exiger des noms de variants uniques."
---

# `musea/unique-variant-names`

Exiger des noms de variants uniques.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Deux variants du même bloc art utilisent tous deux le nom primary.

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

## Bon

Les variants possèdent les noms distincts primary et secondary.

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/unique-variant-names": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea](../musea.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10)
