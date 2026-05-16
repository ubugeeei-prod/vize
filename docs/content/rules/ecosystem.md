---
title: Ecosystem Rules
---

# Ecosystem Rules

These rules cover conventions around Nuxt, Vue Router, Pinia, vue-i18n, and Vue Test Utils.

Every ecosystem rule is opt-in. They are listed in metadata so hosts can enable them by name, but
they are not part of `happy-path`, `nuxt`, or `opinionated`.

## `ecosystem/router-link-require-to`

Requires `to` or `:to` on `<RouterLink>`, `<router-link>`, `<NuxtLink>`, and `<nuxt-link>`.

Default severity: `error`  
Presets: none

Bad:

```vue
<template>
  <RouterLink>Settings</RouterLink>
</template>
```

Good:

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-link`

Warns on static internal path strings in RouterLink-like components. Named route objects keep Vue
Router typed routes and editor completions centered around route names and params.

Default severity: `warning`  
Presets: none

Bad:

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

Good:

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-push`

Warns on `router.push("/path")`, `router.replace("/path")`, and route objects with a static `path`.

Default severity: `warning`  
Presets: none

Bad:

```ts
router.push("/settings");
```

Good:

```ts
router.push({ name: "settings" });
```

## `ecosystem/nuxt-prefer-nuxt-link`

Warns on internal `<a href="/...">` links in Nuxt-oriented code. External links, downloads, and
`target="_blank"` remain plain anchors.

Default severity: `warning`  
Presets: none

Bad:

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

Good:

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## `ecosystem/pinia-prefer-store-to-refs`

Warns when a Pinia store is destructured directly. Use `storeToRefs()` for state and getters, and
keep actions on the store instance.

Default severity: `warning`  
Presets: none

Bad:

```ts
const { name } = useUserStore();
```

Good:

```ts
const store = useUserStore();
const { name } = storeToRefs(store);
```

## `ecosystem/vue-i18n-no-missing-key`

Warns when a static `$t()`, `$te()`, `$tm()`, `t()`, `te()`, or `tm()` key is missing from the same
SFC's local `<i18n lang="json">` block.

Default severity: `warning`  
Presets: none

Bad:

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Good:

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

Warns on `expect(wrapper.html()).toMatchSnapshot()`. Prefer focused assertions around visible text,
attributes, emitted events, or component state.

Default severity: `warning`  
Presets: none

Bad:

```ts
expect(wrapper.html()).toMatchSnapshot();
```

Good:

```ts
expect(wrapper.text()).toContain("Saved");
```
