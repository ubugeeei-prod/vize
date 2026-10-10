---
title: "Règles SSR"
---

# Règles SSR

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Mauvais](#ssr-no-browser-globals-in-ssr-bad) · [Bon](#ssr-no-browser-globals-in-ssr-good) | Interdire les variables globales propres au navigateur dans un contexte SSR |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Mauvais](#ssr-no-hydration-mismatch-bad) · [Bon](#ssr-no-hydration-mismatch-good) | Interdire les valeurs non déterministes qui causent des divergences d’hydratation |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ssr/no-browser-globals-in-ssr`

Interdire les variables globales propres au navigateur dans un contexte SSR

[Mauvais](#ssr-no-browser-globals-in-ssr-bad) · [Bon](#ssr-no-browser-globals-in-ssr-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**Mauvais**

Setup lit immédiatement `window.innerWidth`, alors que `window` n’existe pas lorsque le composant s’exécute sur le serveur.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**Bon**

La largeur initiale est une valeur de ref utilisable sur le serveur, et l’accès au navigateur est déplacé dans `onMounted`, qui s’exécute sur le client plutôt que pendant le setup SSR.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [Toutes les règles](all.md)

### `ssr/no-hydration-mismatch`

Interdire les valeurs non déterministes qui causent des divergences d’hydratation

[Mauvais](#ssr-no-hydration-mismatch-bad) · [Bon](#ssr-no-hydration-mismatch-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**Mauvais**

Le template évalue `Math.random()` pendant le rendu ; le serveur et le client peuvent donc produire des textes différents pour le même paragraphe.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**Bon**

Le paragraphe affiche l’état stable `seed` au lieu d’un nouveau résultat aléatoire. Dans cet exemple de style Nuxt, `useState` fournit l’état partagé et la valeur d’initialisation est la constante `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [Toutes les règles](all.md)
