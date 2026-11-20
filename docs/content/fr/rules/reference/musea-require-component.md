---
title: "musea/require-component"
description: "Exiger un composant cible résoluble pour le bloc &lt;art&gt;."
---

# `musea/require-component`

Exiger un composant cible résoluble pour le bloc &lt;art&gt;.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Blocs art, variant et style des fichiers .art.vue de Musea.

## Mauvais

Le bloc art fournit un titre, mais n’identifie pas le composant dont il présente l’aperçu.

```vue annotate="remove:1,2"
<art title="Button">
  <variant name="primary"><div /></variant>
</art>
```

## Bon

Le variant affiche le Button importé et identifie ainsi la cible de l’art.

```vue annotate="add:1,2,3,4,5,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-component": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea](../musea.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
