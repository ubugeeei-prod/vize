---
title: Règles de la Musea et du CSS
---

<!-- Generated translation; source: rules/musea-and-css.md -->

# Règles de la Musea et du CSS

Les règles de la musea valident `<art>` et `<variant>` blocages. Les règles CSS inspectent le contenu du style et recommandent
patrons qui gardent les styles de composants thématiques, prévisibles et compatibles avec Vue et Vapor.

## `musea/require-title`

Nécessite que chaque fichier d’art fournisse un titre d’affichage. Le titre peut venir de `<art title="...">`,
`defineArt("./Button.vue", { title: "..." })`, ou du `defineArt` source de secours composante.

Sévérité par défaut : `error`

Mauvais :

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

Bon :

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/require-component`

Il faut que chaque fichier d’art nomme le composant qu’il documente. Préfère `defineArt("./Button.vue", ...)`;
`<art component="...">` reste supporté pour la compatibilité.

Sévérité par défaut : `warning`

Mauvais :

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

Bon :

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/valid-variant`

Exige que `<variant>` blocs aient une `name`valide.

Sévérité par défaut : `error`

Mauvais :

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

Bon :

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/unique-variant-names`

Nécessite que les noms de variantes soient uniques à l’intérieur d’un bloc d’art.

Sévérité par défaut : `error`

Mauvais :

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

Bon :

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## `musea/no-empty-variant`

Signale des variantes vides qui ne documentent pas les accessoires, les emplacements ou l’état visuel.

Sévérité par défaut : `warning`

Mauvais :

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Bon :

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## `musea/prefer-design-tokens`

Il préfère les variables CSS de jetons de conception aux valeurs primitives codées en dur dans les exemples de Musea.

Sévérité par défaut : `warning`

Mauvais :

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button style="color: #d00">Delete</Button>
  </variant>
</art>
```

Bon :

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button class="danger">Delete</Button>
  </variant>
</art>

<style scoped>
.danger {
  color: var(--color-danger-text);
}
</style>
```

## `css/no-important`

Décourage `!important`.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

Bon :

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## `css/no-hardcoded-values`

Suggère des variables CSS au lieu des valeurs de couleur, d’espacement ou de taille codées.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

Bon :

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

## `css/no-id-selectors`

Décourage les sélecteurs d’identification dans les styles de composants car ils sont difficiles à contourner et à réutiliser.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

Bon :

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## `css/no-display-none`

Ça suggère d’utiliser des primitives de visibilité Vue au lieu de masquer les branches de composants avec CSS.

Sévérité par défaut : `warning`

Mauvais :

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

Bon :

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## `css/no-v-bind-performance`

Avertit du coût d’exécution du CSS `v-bind()` en style chaud.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

Bon :

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## `css/prefer-logical-properties`

Recommande des propriétés logiques pour les mises en page internationalisées.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

Bon :

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## `css/prefer-slotted`

Il recommande `::v-slotted()` pour styliser le contenu des emplacements.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

Bon :

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

## `css/require-font-display`

Cela exige `font-display` dans `@font-face` déclarations.

Sévérité par défaut : `warning`

Mauvais :

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

Bon :

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## Règles CSS supplémentaires

`css/no-utility-classes` met en garde contre l’implémentation de classes utilitaires dans les styles de composants. Par défaut :
`warning`.

`css/prefer-nested-selectors` recommande le CSS nesting pour les sélectionneurs descendants. Par défaut : `warning`.
