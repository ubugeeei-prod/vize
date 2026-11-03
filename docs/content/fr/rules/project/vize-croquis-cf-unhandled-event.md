---
title: "vize:croquis/cf/unhandled-event"
---

# `vize:croquis/cf/unhandled-event`

Un enfant émet un événement qu’aucun parent ne gère.

Gravité par défaut: info  
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

`Child.vue` émet `save`, mais le composant qui l’enveloppe directement ne l’écoute pas ; les événements de composants ne remontent pas automatiquement à travers les composants intermédiaires.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

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

## Bon

`Wrapper.vue` associe un écouteur `save` à son enfant direct. La fonction de rappel vide illustre la gestion de l’événement pour cette règle, et non une implémentation complète de l’enregistrement.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
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

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Index des règles inter-fichiers](../cross-file.md)
