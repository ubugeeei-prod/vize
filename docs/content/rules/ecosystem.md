---
title: Ecosystem Rules
---

# Ecosystem Rules

These rules cover conventions around Nuxt, Vue Router, Pinia, vue-i18n, Vue Test Utils, and Void Vue.

Ecosystem rules are enabled by the `ecosystem` preset. Hosts can also enable them by name when using
`incremental`; they are not part of `happy-path`, `nuxt`, or `opinionated`.

When editor ecosystem helpers are enabled in the LSP, Vize also adds Vue Router route-name
completion, file-route param completion and diagnostics for `useRoute().params`, Vue I18n key
completion, workspace JSON key validation, and inlay previews for static `t()` / `$t()` calls.

## `ecosystem/router-link-require-to`

Requires `to` or `:to` on `<RouterLink>`, `<router-link>`, `<NuxtLink>`, and `<nuxt-link>`.

Default severity: `error`
Presets: `ecosystem`

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
Presets: `ecosystem`

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
Presets: `ecosystem`

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
Presets: `ecosystem`

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
Presets: `ecosystem`

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
Presets: `ecosystem`

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

## `ecosystem/void-link-require-href`

Requires `href` or `:href` on Void Vue `<Link>` components imported from `@void/vue`.

Default severity: `error`
Presets: `ecosystem`

Bad:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

Good:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## `ecosystem/void-link-valid-method`

Warns on unknown static Void Vue `<Link method>` values and on GET-only props such as `prefetch`
or `reloadDocument` when the link uses a mutation method.

Default severity: `warning`
Presets: `ecosystem`

Bad:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

Good:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

Warns on `expect(wrapper.html()).toMatchSnapshot()`. Prefer focused assertions around visible text,
attributes, emitted events, or component state.

Default severity: `warning`
Presets: `ecosystem`

Bad:

```ts
expect(wrapper.html()).toMatchSnapshot();
```

Good:

```ts
expect(wrapper.text()).toContain("Saved");
```
