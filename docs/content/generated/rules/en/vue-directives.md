---
title: "Vue Rules: Directive Conventions"
---

# Vue Rules: Directive Conventions

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good) | Recommend scoped event names using context:event format |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good) | Enforce `v-bind` directive style |
| [`vue/v-on-style`](#vue-v-on-style) | [Bad](#vue-v-on-style-bad) · [Good](#vue-v-on-style-good) | Enforce `v-on` directive style |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Bad](#vue-v-slot-style-bad) · [Good](#vue-v-slot-style-good) | Enforce `v-slot` directive style |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Bad](#vue-warn-custom-directive-bad) · [Good](#vue-warn-custom-directive-good) | Warn about custom directives that need registration |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/scoped-event-names`

Recommend scoped event names using context:event format

[Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good)

Default severity: `warning`  
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
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**Bad**

`playAudio`, `pauseAudio`, and `reloadAudio` encode their scope as camel-case suffixes rather than the rule's colon-separated event convention.

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**Good**

`audio:play`, `audio:pause`, and `audio:reload` share an explicit `audio:` scope. The emitting component must use the same names.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [All rules](all.md)

### `vue/use-v-on-exact`

Enforce `.exact` modifier on `v-on` when there are modifier-based handlers

[Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good)

Default severity: `warning`  
Presets: `essential`, `nuxt`, `opinionated`  
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
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**Bad**

The plain click handler can also run on Ctrl-click, overlapping the separate `.ctrl` handler.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Good**

`.exact` limits the ordinary click handler to clicks without modifier keys; the Ctrl-specific handler remains separate.

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [All rules](all.md)

### `vue/v-bind-style`

Enforce `v-bind` directive style

[Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good)

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
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**Bad**

`v-bind:class` uses the long form where the configured binding style requires the colon shorthand.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Good**

`:class` retains the same expression with the required shorthand; this rule concerns spelling rather than the value's type.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [All rules](all.md)

### `vue/v-on-style`

Enforce `v-on` directive style

[Bad](#vue-v-on-style-bad) · [Good](#vue-v-on-style-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**Bad**

`v-on:click` uses the long event-listener form where the rule requires shorthand.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Good**

`@click` keeps the same handler while using the configured shorthand.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [All rules](all.md)

### `vue/v-slot-style`

Enforce `v-slot` directive style

[Bad](#vue-v-slot-style-bad) · [Good](#vue-v-slot-style-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**Bad**

The component uses `#default` and the template uses `v-slot:header`, opposite to the rule's context-specific styles.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Good**

Use `v-slot` for the component's default slot and `#header` for the template's named slot.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [All rules](all.md)

### `vue/warn-custom-directive`

Warn about custom directives that need registration

[Bad](#vue-warn-custom-directive-bad) · [Good](#vue-warn-custom-directive-good)

Default severity: `warning`  
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
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**Bad**

`v-focus`, `v-mask`, and `v-click-outside` require project-specific directive implementations that this optional convention flags.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Good**

The example uses built-in `v-if`, `v-model`, and `v-on`. A correctly registered custom directive can still be valid Vue when this policy is disabled.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [All rules](all.md)
