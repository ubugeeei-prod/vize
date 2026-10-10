---
title: "Vapor rules"
---

# Vapor rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good) | Disallow getCurrentInstance() in Vapor mode (returns null) |
| [`script/no-next-tick`](#script-no-next-tick) | [Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good) | Disallow nextTick() usage in Vapor-oriented components |
| [`script/no-options-api`](#script-no-options-api) | [Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good) | Disallow Options API patterns in Vapor mode |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Bad](#vapor-no-inline-template-bad) · [Good](#vapor-no-inline-template-good) | Disallow deprecated inline-template attribute |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Bad](#vapor-no-vue-lifecycle-events-bad) · [Good](#vapor-no-vue-lifecycle-events-good) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Bad](#vapor-prefer-static-class-bad) · [Good](#vapor-prefer-static-class-good) | Prefer static class over dynamic class binding for string literals |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Bad](#vapor-require-vapor-attribute-bad) · [Good](#vapor-require-vapor-attribute-good) | Suggest adding vapor attribute to script setup |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null)

[Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**Bad**

The Vapor-marked setup imports and calls `getCurrentInstance`, relying on an instance API this rule disallows for Vapor-oriented components.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Good**

`inject("app-config")` obtains the explicitly provided configuration without importing or calling `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [All rules](all.md)

### `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components

[Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**Bad**

The Vapor-oriented component imports and awaits `nextTick`, introducing the DOM-flush scheduling dependency that this migration rule rejects.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Good**

The input is obtained through `useTemplateRef` and focused at `onMounted`. The explicit mount boundary replaces the example’s `nextTick` dependency.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [All rules](all.md)

### `script/no-options-api`

Disallow Options API patterns in Vapor mode

[Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**Bad**

The default-export object declares Options API `data()`, a component option form prohibited by this rule.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**Good**

The component state becomes a Composition API `ref` in Vapor `<script setup>`, removing the Options API object and its `data` option.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [All rules](all.md)

### `vapor/no-inline-template`

Disallow deprecated inline-template attribute

[Bad](#vapor-no-inline-template-bad) · [Good](#vapor-no-inline-template-good)

Default severity: `error`  
Presets: `nuxt`, `opinionated`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**Bad**

LegacyCard uses the inline-template attribute for its child markup.

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**Good**

The markup is passed through the default slot instead of an inline template.

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [All rules](all.md)

### `vapor/no-vue-lifecycle-events`

Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)

[Bad](#vapor-no-vue-lifecycle-events-bad) · [Good](#vapor-no-vue-lifecycle-events-good)

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**Bad**

The input uses the @vue:mounted template lifecycle event.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**Good**

onMounted accesses the named template reference and focuses the input through the supported script lifecycle hook.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [All rules](all.md)

### `vapor/prefer-static-class`

Prefer static class over dynamic class binding for string literals

[Bad](#vapor-prefer-static-class-bad) · [Good](#vapor-prefer-static-class-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**Bad**

The class binding evaluates a constant string even though the class does not change.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**Good**

A static class attribute expresses the same panel classes without a binding.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [All rules](all.md)

### `vapor/require-vapor-attribute`

Suggest adding vapor attribute to script setup

[Bad](#vapor-require-vapor-attribute-bad) · [Good](#vapor-require-vapor-attribute-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.

**Configured ID (currently no SFC finding)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**Bad**

The script setup block lacks the Vapor compilation attribute. This is an intended convention: the current empty rule callback does not diagnose it.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**Good**

Adding vapor selects Vapor compilation. It demonstrates the intended repair and does not imply that the current linter emits this catalog rule.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [All rules](all.md)
