---
title: Annotations de commentaires
---

<!-- Generated translation; source: guide/comment-annotations.md -->

# Annotations de commentaires

Vize fournit des annotations basées sur des commentaires pour contrôler le linting, les diagnostics et le comportement du codegen. Il existe deux systèmes d’annotation selon leur lieu d’utilisation :

- **`<!-- @vize:xxx -->`** — Commentaires HTML en `<template>` (directives Patina linter)
- **`// @vize forget: reason`** — Commentaires JS dans `<script>` (suppression d’analyse croisée de fichiers)

Toutes `@vize:` directives modèles sont **retirées de la sortie de compilation** — elles n’apparaissent jamais dans le code de production.

## Directives modèles (`@vize:`)

Utilisé à l’intérieur `<template>` comme commentaires HTML. Celles-ci contrôlent le comportement de la patine (le linter intégré).

### `@vize:expected`

Attendez-vous à un diagnostic sur la ligne suivante. Si aucun diagnostic n’est produit, c’est une opération interdite. Similaire à `@ts-expect-error`.

```vue
<template>
  <ul>
    <!-- @vize:expected -->
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
```

### `@vize:ignore-start` / `@vize:ignore-end`

Supprime tous les diagnostics dans une région.

```vue
<template>
  <!-- @vize:ignore-start -->
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <!-- @vize:ignore-end -->
</template>
```

### `@vize:level(warn|error|off)`

Passer outre la gravité des diagnostics sur la ligne suivante.

```vue
<template>
  <!-- @vize:level(warn) -->
  <img src="/photo.png" />

  <!-- @vize:level(off) -->
  <li v-for="item in items">{{ item }}</li>
</template>
```

| Valeur  | Effet                           |
| ------- | ------------------------------- |
| `warn`  | Rétrogradation en avertissement |
| `error` | Passer à l’erreur               |
| `off`   | Supprimer complètement          |

### `@vize:todo`

Émettez un avertissement TO.

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

Affichez une erreur FIXME.

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

Émettez un avertissement de dépréciation.

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

Commentaire sur la documentation. Aucun effet de charpie.

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

Marquez un nœud à dépouiller dans les versions de production, gardé en développement.

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### Résumé

| Directive                | Effet                                               | Gravité       |
| ------------------------ | --------------------------------------------------- | ------------- |
| `@vize:expected`         | Attendez-vous à un diagnostic sur la ligne suivante | —             |
| `@vize:ignore-start/end` | Supprimer tous les diagnostics dans la région       | —             |
| `@vize:level(...)`       | Annuler la sévérité de la ligne suivante            | —             |
| `@vize:todo <msg>`       | Émettre TODO                                        | Avertissement |
| `@vize:fixme <msg>`      | Émet FIXME                                          | Erreur        |
| `@vize:deprecated <msg>` | Émettre un avis de dépréciation                     | Avertissement |
| `@vize:docs <text>`      | Documentation (sans effet de peluche)               | —             |
| `@vize:dev-only`         | Bande dessinée en production                        | —             |

## Suppression de script (`@vize forget`)

Utilisé à l’intérieur `<script>` comme commentaire JS. Supprime les avertissements d’analyse croisée (Croquis) sur la ligne suivante.

### Syntaxe

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

Une **raison est nécessaire** — vous devez expliquer pourquoi la suppression est nécessaire.

### Exemple

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

Sans cette annotation, Vize avertirait que déstructurer une valeur de retour de `inject()` réactive casse le suivi de la réactivité.

### Règles

| Règne                    | Description                                                             |
| ------------------------ | ----------------------------------------------------------------------- |
| Raison requise           | `// @vize forget` sans raison est une erreur                            |
| Côlon requis             | Doit utiliser `// @vize forget: <reason>` (deux-points avant la raison) |
| Ligne suivante seulement | S’applique à la ligne suivante non commentée, non vide                  |
| Pas d’orphelins          | Une suppression à la fin d’un fichier sans code après est une erreur    |

### Suppressions multiples

Chaque `@vize forget` s’applique indépendamment à la ligne de code suivante :

```vue
<script setup>
import { inject } from "vue";

// @vize forget: one-time read for display name
const { name } = inject("user");

// @vize forget: static config value
const { theme } = inject("config");
</script>
```

### Passer les commentaires

La suppression vise la ligne **de code** suivante, en sautant les commentaires et les lignes blanches :

```vue
<script setup>
// @vize forget: read-only access
// This comment is skipped
const { count } = inject("state");
</script>
```

### Raisons courantes

| Raison                       | Quand utiliser                                     |
| ---------------------------- | -------------------------------------------------- |
| `intentionally non-reactive` | La valeur n’a pas besoin d’être réactive           |
| `read-only access`           | Seulement la lecture, pas le suivi des changements |
| `legacy code`                | Problème connu, je vais refactoriser plus tard     |
| `third-party integration`    | Exigé par une bibliothèque externe                 |

### Exemples invalides

```ts
// @vize forget
const { count } = inject("state");
// ^ Error: requires a reason

// @vize forget because I said so
const { count } = inject("state");
// ^ Error: requires a colon before the reason

// @vize forget:
const { count } = inject("state");
// ^ Error: reason cannot be empty
```
