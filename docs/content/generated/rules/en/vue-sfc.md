---
title: "Vue Rules: SFC Blocks"
---

# Vue Rules: SFC Blocks

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Bad](#vue-no-preprocessor-lang-bad) · [Good](#vue-no-preprocessor-lang-good) | Discourage CSS preprocessor usage in favor of modern CSS |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Bad](#vue-no-script-non-standard-lang-bad) · [Good](#vue-no-script-non-standard-lang-good) | Discourage non-standard script lang values |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Bad](#vue-no-src-attribute-bad) · [Good](#vue-no-src-attribute-good) | Discourage src attribute on SFC blocks |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Bad](#vue-no-template-lang-bad) · [Good](#vue-no-template-lang-good) | Discourage lang attribute on template block |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good) | Require scoped attribute on style tags |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good) | Enforce consistent order of SFC top-level elements |
| [`vue/single-style-block`](#vue-single-style-block) | [Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good) | Recommend having a single style block |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Bad](#vue-warn-custom-block-bad) · [Good](#vue-warn-custom-block-good) | Warn about custom blocks in SFC files |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-preprocessor-lang`

Discourage CSS preprocessor usage in favor of modern CSS

[Bad](#vue-no-preprocessor-lang-bad) · [Good](#vue-no-preprocessor-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**Bad**

The style block selects SCSS with lang. This describes the intended no-preprocessor convention; the current SFC path does not emit this rule.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Good**

The same CSS declarations omit the preprocessor lang. This is the convention repair, not an executable Bad/Good diagnostic difference today.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [All rules](all.md)

### `vue/no-script-non-standard-lang`

Discourage non-standard script lang values

[Bad](#vue-no-script-non-standard-lang-bad) · [Good](#vue-no-script-non-standard-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**Bad**

The script uses CoffeeScript syntax under lang=coffee. The current SFC path does not emit this catalog rule for that language.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Good**

The script uses an ordinary TypeScript declaration with lang=ts, illustrating the intended language convention.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [All rules](all.md)

### `vue/no-src-attribute`

Discourage src attribute on SFC blocks

[Bad](#vue-no-src-attribute-bad) · [Good](#vue-no-src-attribute-good)

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
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**Bad**

The SFC blocks delegate their template, script, and style content to src files.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Good**

Each SFC block contains its own content without an external src attribute.

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [All rules](all.md)

### `vue/no-template-lang`

Discourage lang attribute on template block

[Bad](#vue-no-template-lang-bad) · [Good](#vue-no-template-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**Bad**

The template selects Pug through lang. This is an intended HTML-only convention; the current SFC path does not diagnose this catalog ID.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Good**

An ordinary HTML template omits lang and uses the paragraph directly. This illustrates the convention without claiming a current SFC finding.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [All rules](all.md)

### `vue/require-scoped-style`

Require scoped attribute on style tags

[Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good)

Default severity: `warning`  
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
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**Bad**

The `.button` style is unscoped and can affect matching elements outside this component.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Good**

Adding `scoped` applies Vue's component scope to the same selector and declarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [All rules](all.md)

### `vue/sfc-element-order`

Enforce consistent order of SFC top-level elements

[Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**Bad**

The style block precedes the script block, contrary to the configured SFC block order.

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**Good**

The blocks follow script → template → style. Projects can choose a different order through this rule's typed option.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [All rules](all.md)

### `vue/single-style-block`

Recommend having a single style block

[Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good)

Default severity: `warning`  
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
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**Bad**

The component splits its scoped panel and title styles across two style blocks.

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**Good**

Both selectors stay scoped in one style block, satisfying the single-block convention without dropping either style.

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [All rules](all.md)

### `vue/warn-custom-block`

Warn about custom blocks in SFC files

[Bad](#vue-warn-custom-block-bad) · [Good](#vue-warn-custom-block-good)

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
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**Bad**

The SFC contains an `<i18n>` custom block, which needs an external integration beyond ordinary template/script/style processing.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Good**

The example uses standard template and script-setup blocks. This optional portability warning does not mean every custom block is invalid Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [All rules](all.md)
