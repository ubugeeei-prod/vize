---
title: Règles SSR
---

<!-- Generated translation; source: rules/ssr.md -->

# Règles SSR

Ces règles couvrent les codes et modèles qui peuvent perturber le rendu serveur ou l’hydratation. Ils sont
documentés séparément des règles HTML et Vapor car le mode défaillance est la frontière serveur/
client.

## `ssr/no-browser-globals-in-ssr`

Rapporte des globals uniquement du navigateur dans le code qui peuvent s’exécuter pendant le SSR.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

Bon :

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Les vérifications de garde telles que `typeof window === "undefined"` sont autorisées car le formulaire d’identifiant `typeof`
direct est sécurisé lors du rendu du serveur. Les chaînes de caractères, commentaires et lettres régulières sont également
ignorés lorsqu’ils contiennent des noms comme `window` ou `document`. Accéder à un membre comme
`typeof window.innerWidth` rapporte toujours, car cela évalue le navigateur globalement.

## `ssr/no-hydration-mismatch`

Rapporte des valeurs de modèles non déterministes qui peuvent différer entre le rendu serveur et l’hydratation
client.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

Bon :

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```
