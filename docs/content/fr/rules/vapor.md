---
title: Règles de la vapeur
---

<!-- Generated translation; source: rules/vapor.md -->

# Règles de la vapeur

Ces règles couvrent les contraintes de modèles pour les composants et applications orientés Vapor. L’API de composition et
guidance Vapor au niveau des scripts sont présentes dans [Type and script rules](./type-and-script.md).

## `vapor/no-vue-lifecycle-events`

Rapporte des événements du cycle de vie par élément tels que `@vue:mounted`.

Sévérité par défaut : `error`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

Bon :

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## `vapor/require-vapor-attribute`

Il suggère d’ajouter `vapor` à `<script setup>` quand le préréglage attend des composants compatibles Vapor.

Sévérité par défaut : `warning`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const count = ref(0);
</script>
```

Bon :

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `vapor/no-inline-template`

Rapporte l’attribut `inline-template` déprécié.

Sévérité par défaut : `error`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

Bon :

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

## `vapor/prefer-static-class`

Rapporte des liaisons dynamiques `:class` dont la valeur est une chaîne statique, littéral.

Sévérité par défaut : `warning`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

Bon :

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```
