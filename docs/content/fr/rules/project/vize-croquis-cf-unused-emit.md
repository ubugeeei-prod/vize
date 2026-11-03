---
title: "vize:croquis/cf/unused-emit"
---

# `vize:croquis/cf/unused-emit`

Un événement déclaré n’est jamais émis.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

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

L’enfant déclare `save`, mais n’appelle jamais la fonction d’émission d’événements avec ce nom.

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
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

## Bon

L’exemple appelle `emit("save")`, utilisant ainsi l’événement déclaré. Les interactions réelles doivent l’émettre lorsque l’action correspondante se produit.

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
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Index des règles inter-fichiers](../cross-file.md)
