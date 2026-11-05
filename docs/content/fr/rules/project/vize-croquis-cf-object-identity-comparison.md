---
title: "vize:croquis/cf/object-identity-comparison"
---

# `vize:croquis/cf/object-identity-comparison`

Comparez les identifiants stables des enregistrements représentés par des objets bruts et des proxies réactifs.

| Gravité par défaut | Correction automatique | Options |
| --- | --- | --- |
| Non émis | Aucune ; examinez les fichiers concernés et appliquez la correction | Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité |

Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

L’exemple suppose que les identifiants distinguent les enregistrements de façon unique. Comparer deux références au même proxy réactif reste valide ; ce contrat n’a actuellement aucun producteur de diagnostics. Un objet brut et son proxy réactif ont des identités distinctes, et toRaw() renvoie l’objet d’origine.

## Champ d’application et limites

Un objet brut et son proxy réactif ont des identités d’objet distinctes, et toRaw() renvoie l’objet original. Ce contrat illustre la comparaison d’identifiants stables d’enregistrement lorsque c’est l’identité de l’enregistrement qui est recherchée.

## Fichiers communs du projet

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

## Mauvais

Ce contrat illustre la comparaison d’identifiants stables d’enregistrement lorsque c’est l’identité de l’enregistrement qui est voulue. `proxy === raw` compare l’identité de l’enveloppe ; le résultat est donc faux même si les deux représentent le même enregistrement utilisateur. L’application visait l’identité de l’enregistrement, et non celle de l’enveloppe de l’objet.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## Bon

Comparer le `id` stable de l’enregistrement répond à la question voulue sans dépendre du fait que l’objet soit brut ou enveloppé dans un proxy.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## Configuration (Vite+)

Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.

## Voir aussi

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](../cross-file.md)
