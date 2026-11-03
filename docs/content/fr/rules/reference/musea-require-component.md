---
title: "musea/require-component"
description: "Exiger l’attribut component dans le bloc &lt;art&gt;."
---

# `musea/require-component`

Exiger l’attribut component dans le bloc &lt;art&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le bloc art fournit un titre, mais n’identifie pas le composant dont il présente l’aperçu.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

## Bon

defineArt fournit ./Button.vue comme composant du bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
