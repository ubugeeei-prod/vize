---
title: "Ecosystem rules"
---

# Ecosystem rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Bad](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](#ecosystem-nuxt-prefer-nuxt-link-good) | Prefer NuxtLink for internal application links |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Bad](#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](#ecosystem-pinia-prefer-store-to-refs-good) | Prefer storeToRefs() when destructuring Pinia stores |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Bad](#ecosystem-router-link-require-to-bad) · [Good](#ecosystem-router-link-require-to-good) | Require a `to` target on RouterLink and NuxtLink components |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Bad](#ecosystem-void-link-require-href-bad) · [Good](#ecosystem-void-link-require-href-good) | Require `href` on Void Vue Link components |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Bad](#ecosystem-void-link-valid-method-bad) · [Good](#ecosystem-void-link-valid-method-good) | Validate static Void Vue Link method props |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Bad](#ecosystem-vue-i18n-no-missing-key-bad) · [Good](#ecosystem-vue-i18n-no-missing-key-good) | Report static vue-i18n keys that are absent from local SFC messages |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Bad](#ecosystem-vue-router-prefer-named-link-bad) · [Good](#ecosystem-vue-router-prefer-named-link-good) | Prefer named route objects over static path strings in RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Bad](#ecosystem-vue-router-prefer-named-push-bad) · [Good](#ecosystem-vue-router-prefer-named-push-good) | Prefer named route objects for Vue Router programmatic navigation |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Bad](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](#ecosystem-vue-test-utils-no-html-snapshot-good) | Avoid snapshotting wrapper.html() in Vue Test Utils tests |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Bad](#nuxt-no-nuxt-config-test-key-bad) · [Good](#nuxt-no-nuxt-config-test-key-good) | Disallow setting `test` key in Nuxt config |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Bad](#nuxt-no-page-meta-runtime-values-bad) · [Good](#nuxt-no-page-meta-runtime-values-good) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Bad](#nuxt-nuxt-config-keys-order-bad) · [Good](#nuxt-nuxt-config-keys-order-good) | Prefer recommended order of Nuxt config properties |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Bad](#nuxt-prefer-import-meta-bad) · [Good](#nuxt-prefer-import-meta-good) | Prefer using `import.meta.*` over `process.*` |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Prefer NuxtLink for internal application links

[Bad](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](#ecosystem-nuxt-prefer-nuxt-link-good)

Default severity: `warning`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**Bad**

The internal settings destination uses a plain anchor in a Nuxt application.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**Good**

NuxtLink handles the same internal destination through the Nuxt router.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [All rules](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

Prefer storeToRefs() when destructuring Pinia stores

[Bad](#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](#ecosystem-pinia-prefer-store-to-refs-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**Bad**

Destructuring `name` directly from the store separates the value from its reactive store access.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**Good**

The store remains intact and storeToRefs creates a reactive reference for name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [All rules](all.md)

### `ecosystem/router-link-require-to`

Require a `to` target on RouterLink and NuxtLink components

[Bad](#ecosystem-router-link-require-to-bad) · [Good](#ecosystem-router-link-require-to-good)

Default severity: `error`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**Bad**

The nested RouterLink has no `to` destination; it cannot rely on root attribute fallthrough.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**Good**

`to="/settings"` explicitly supplies the nested link destination.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [All rules](all.md)

### `ecosystem/void-link-require-href`

Require `href` on Void Vue Link components

[Bad](#ecosystem-void-link-require-href-bad) · [Good](#ecosystem-void-link-require-href-good)

Default severity: `error`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**Bad**

The Link imported from @void/vue omits its href destination.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**Good**

The same imported Link receives the settings destination through href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [All rules](all.md)

### `ecosystem/void-link-valid-method`

Validate static Void Vue Link method props

[Bad](#ecosystem-void-link-valid-method-bad) · [Good](#ecosystem-void-link-valid-method-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**Bad**

The DELETE action requests prefetching, although prefetch is intended for navigation requests.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**Good**

Removing prefetch keeps the DELETE action without prefetching that non-GET request.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [All rules](all.md)

### `ecosystem/vue-i18n-no-missing-key`

Report static vue-i18n keys that are absent from local SFC messages

[Bad](#ecosystem-vue-i18n-no-missing-key-bad) · [Good](#ecosystem-vue-i18n-no-missing-key-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**Bad**

The template requests auth.missing, but the local English messages declare only auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**Good**

The template requests the auth.login key that exists in the local messages.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [All rules](all.md)

### `ecosystem/vue-router-prefer-named-link`

Prefer named route objects over static path strings in RouterLink

[Bad](#ecosystem-vue-router-prefer-named-link-bad) · [Good](#ecosystem-vue-router-prefer-named-link-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**Bad**

The RouterLink destination is a literal path rather than a named route.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**Good**

The bound route object identifies the destination by its settings route name.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [All rules](all.md)

### `ecosystem/vue-router-prefer-named-push`

Prefer named route objects for Vue Router programmatic navigation

[Bad](#ecosystem-vue-router-prefer-named-push-bad) · [Good](#ecosystem-vue-router-prefer-named-push-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**Bad**

router.push receives a path string that is tied to the current URL spelling.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**Good**

router.push receives a route object with the stable settings name.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [All rules](all.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Avoid snapshotting wrapper.html() in Vue Test Utils tests

[Bad](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](#ecosystem-vue-test-utils-no-html-snapshot-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**Bad**

The assertion snapshots the complete wrapper HTML instead of checking the expected behavior.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**Good**

The assertion checks that the rendered text contains Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [All rules](all.md)

### `nuxt/no-nuxt-config-test-key`

Disallow setting `test` key in Nuxt config

[Bad](#nuxt-no-nuxt-config-test-key-bad) · [Good](#nuxt-no-nuxt-config-test-key-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: Nuxt configuration files (nuxt.config.ts)  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**Bad**

The exported Nuxt config sets the identifier key `test` to the boolean `true`, the obsolete config shape this rule rejects.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**Good**

The empty config removes that boolean `test` property. This example does not forbid a test configuration object.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [All rules](all.md)

### `nuxt/no-page-meta-runtime-values`

Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup

[Bad](#nuxt-no-page-meta-runtime-values-bad) · [Good](#nuxt-no-page-meta-runtime-values-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**Bad**

`useRoute()` is evaluated immediately while building the `definePageMeta` object, although the macro hoists that metadata outside the setup runtime context.

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**Good**

`validate` receives a callback, so its `useRoute().params.id` access is deferred until the callback runs. The rule distinguishes deferred function bodies from eager metadata values.

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [All rules](all.md)

### `nuxt/nuxt-config-keys-order`

Prefer recommended order of Nuxt config properties

[Bad](#nuxt-nuxt-config-keys-order-bad) · [Good](#nuxt-nuxt-config-keys-order-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: Available for supported findings  
Applies to: Nuxt configuration files (nuxt.config.ts)  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**Bad**

The config places `ssr` before `modules`, reversing their order in the rule’s recommended Nuxt config key sequence.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**Good**

Putting `modules` before `ssr` preserves both values while satisfying the prescribed order; the repair changes layout rather than either option’s meaning.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [All rules](all.md)

### `nuxt/prefer-import-meta`

Prefer using `import.meta.*` over `process.*`

[Bad](#nuxt-prefer-import-meta-bad) · [Good](#nuxt-prefer-import-meta-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**Bad**

`process.client` uses a legacy Nuxt environment flag that the rule asks to migrate to `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**Good**

`import.meta.client` keeps the browser-only branch explicit using the replacement environment flag.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [All rules](all.md)
