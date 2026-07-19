---
title: Règles de l’écosystème
---

<!-- Generated translation; source: rules/ecosystem.md -->

# Règles de l’écosystème

Ces règles couvrent les conventions autour de Nuxt, Vue Router, Pinia, vue-i18n, Vue Test Utils et Void Vue.

Les règles de l’écosystème sont activées par le préréglage `ecosystem`. Les hôtes peuvent aussi les activer par leur nom lorsqu’ils utilisent
`incremental`; ils ne font pas partie de `happy-path`, `nuxt`ou `opinionated`.

Lorsque les aides de l’écosystème de l’éditeur sont activées dans le LSP, Vize ajoute également le nom de route Vue Router
complétion, la complétion et le diagnostic des params de routage de fichiers pour `useRoute().params`, la complétion de
de clé Vue I18n, la validation de la clé JSON de l’espace de travail, et des prévisualisations d’incrustation pour les appels statiques `t()` / `$t()` .

## `ecosystem/router-link-require-to`

Nécessite `to` ou `:to` sur `<RouterLink>`, `<router-link>`, `<NuxtLink>`et `<nuxt-link>`.

Sévérité par défaut : `error`
Presets : `ecosystem`

Mauvais :

```vue
<template>
  <RouterLink>Settings</RouterLink>
</template>
```

Bon :

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-link`

Avertissements sur les chaînes de chemins internes statiques dans des composants de type RouterLink. Les objets de route nommés gardent les routes typées de Vue
Router et les complétions de l’éditeur centrées autour des noms et params des itinéraires.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

Bon :

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-push`

Avertit sur `router.push("/path")`, `router.replace("/path")`, et route les objets avec une `path`statique.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```ts
router.push("/settings");
```

Bon :

```ts
router.push({ name: "settings" });
```

## `ecosystem/nuxt-prefer-nuxt-link`

Avertissements sur les liens `<a href="/...">` internes dans le code orienté Nuxt. Les liens externes, téléchargements et
`target="_blank"` restent de simples ancres.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

Bon :

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## `ecosystem/pinia-prefer-store-to-refs`

Avertit lorsqu’un magasin Pinia est déstructuré directement. Utilisez `storeToRefs()` pour l’état et les obtenteurs, et
conservez les actions sur l’instance du magasin.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```ts
const { name } = useUserStore();
```

Bon :

```ts
const store = useUserStore();
const { name } = storeToRefs(store);
```

## `ecosystem/vue-i18n-no-missing-key`

Avertit lorsqu’une clé statique `$t()`, `$te()`, `$tm()`, `t()`, `te()`ou `tm()` touche manque dans le même
bloc `<i18n lang="json">` local de SFC.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Bon :

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## `ecosystem/void-link-require-href`

Nécessite `href` ou `:href` sur Void Vue `<Link>` des composants importés de `@void/vue`.

Sévérité par défaut : `error`
Presets : `ecosystem`

Mauvais :

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

Bon :

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## `ecosystem/void-link-valid-method`

Avertit sur des valeurs statiques inconnues de `<Link method>` Void Vue et sur les props GET-only tels que `prefetch`
ou `reloadDocument` lorsque le lien utilise une méthode de mutation.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

Bon :

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

Avertissements sur `expect(wrapper.html()).toMatchSnapshot()`. Privilégiez des affirmations ciblées autour du texte visible, des attributs
, des événements émis ou de l’état des composants.

Sévérité par défaut : `warning`
Presets : `ecosystem`

Mauvais :

```ts
expect(wrapper.html()).toMatchSnapshot();
```

Bon :

```ts
expect(wrapper.text()).toContain("Saved");
```
