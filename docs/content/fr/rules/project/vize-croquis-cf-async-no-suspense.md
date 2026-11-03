---
title: "vize:croquis/cf/async-no-suspense"
---

# `vize:croquis/cf/async-no-suspense`

Un composant asynchrone est rendu sans frontière Suspense.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

Prise en charge actuelle: `no-source-async-fact`

Le producteur de diagnostics de cette frontière lit macros.is_async(), mais l’analyse du code source enregistre actuellement le await de premier niveau dans la portée script-setup à la place. La paire complète de sources Mauvais/Bon ci-dessous ne produit donc aucun diagnostic async-no-suspense via le CLI actuel. Elle explique la convention Suspense ; fournir le fait de macro manquant reste un travail d’implémentation à réaliser.

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

L’enfant contient un await au niveau supérieur, mais son parent ne fournit aucune frontière `<Suspense>`. L’analyse actuelle du code source ne fournit pas l’information de macro nécessaire pour émettre ce code de diagnostic.

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
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

## Bon

Le parent enveloppe le même enfant asynchrone dans `<Suspense>` avec un contenu de secours pendant le chargement. Cela illustre la convention ; aucune des deux variantes de code source ne produit ce diagnostic dans la passe actuelle.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](../cross-file.md)
