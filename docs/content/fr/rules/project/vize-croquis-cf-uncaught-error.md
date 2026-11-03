---
title: "vize:croquis/cf/uncaught-error"
---

# `vize:croquis/cf/uncaught-error`

Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| info | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

Le producteur actuel analyse les expressions de template telles que JSON.parse(input). Il ne signale pas une instruction throw présente uniquement dans le bloc script.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## Mauvais

Le template de l’enfant appelle `JSON.parse` sur une entrée mal formée et le parent accessible n’a aucune frontière de capture d’erreur.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

## Bon

Le parent enregistre `onErrorCaptured` autour de cet enfant. Renvoyer `false` arrête la propagation ; une frontière utilisée en production doit aussi présenter une interface de récupération utile.

`App.vue`

```vue
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](../cross-file.md)
