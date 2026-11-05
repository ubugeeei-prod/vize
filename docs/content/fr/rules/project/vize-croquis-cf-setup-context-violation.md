---
title: "vize:croquis/cf/setup-context-violation"
---

# `vize:croquis/cf/setup-context-violation`

Créez l’état réactif propre à chaque composant dans le setup de son instance.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| Selon le contexte | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

## Champ d’application et limites

Ce scénario de l’analyseur expérimental place ref(0) au niveau du module d’un script normal plutôt que dans le setup propre à chaque instance. Vue autorise cet état réactif ; l’exemple exige que chaque instance du composant possède son propre compteur. La CLI n’émet pas ce code individuel.

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

`ref(0)` est créé au niveau du module d’un script normal, en dehors du contexte setup propre à chaque instance représenté par ce scénario d’analyse.

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

## Bon

Déplacer la liaison dans script setup, où chaque instance de composant possède son décompte et où le template peut le lire.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Index des règles inter-fichiers](../cross-file.md)
