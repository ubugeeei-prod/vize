---
title: "Musea and CSS rules"
---

# Musea and CSS rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`css/no-display-none`](#css-no-display-none) | [Bad](#css-no-display-none-bad) · [Good](#css-no-display-none-good) | Suggest using v-show instead of display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [Bad](#css-no-hardcoded-values-bad) · [Good](#css-no-hardcoded-values-good) | Suggest using CSS variables instead of hardcoded values |
| [`css/no-id-selectors`](#css-no-id-selectors) | [Bad](#css-no-id-selectors-bad) · [Good](#css-no-id-selectors-good) | Discourage use of ID selectors in CSS |
| [`css/no-important`](#css-no-important) | [Bad](#css-no-important-bad) · [Good](#css-no-important-good) | Discourage use of !important in CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [Bad](#css-no-utility-classes-bad) · [Good](#css-no-utility-classes-good) | Warn against implementing utility classes in component styles |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [Bad](#css-no-v-bind-performance-bad) · [Good](#css-no-v-bind-performance-good) | Warn about performance cost of CSS v-bind() |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [Bad](#css-prefer-logical-properties-bad) · [Good](#css-prefer-logical-properties-good) | Recommend CSS logical properties for better i18n support |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [Bad](#css-prefer-nested-selectors-bad) · [Good](#css-prefer-nested-selectors-good) | Recommend using CSS nesting for descendant selectors |
| [`css/prefer-slotted`](#css-prefer-slotted) | [Bad](#css-prefer-slotted-bad) · [Good](#css-prefer-slotted-good) | Recommend ::v-slotted() for styling slot content |
| [`css/require-font-display`](#css-require-font-display) | [Bad](#css-require-font-display-bad) · [Good](#css-require-font-display-good) | Require font-display in @font-face rules |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Bad](#musea-no-empty-variant-bad) · [Good](#musea-no-empty-variant-good) | Disallow empty &lt;variant&gt; blocks |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Bad](#musea-prefer-design-tokens-bad) · [Good](#musea-prefer-design-tokens-good) | Prefer design token CSS variables over hardcoded primitive values |
| [`musea/require-component`](#musea-require-component) | [Bad](#musea-require-component-bad) · [Good](#musea-require-component-good) | Require component attribute in &lt;art&gt; block |
| [`musea/require-title`](#musea-require-title) | [Bad](#musea-require-title-bad) · [Good](#musea-require-title-good) | Require title attribute in &lt;art&gt; block |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Bad](#musea-unique-variant-names-bad) · [Good](#musea-unique-variant-names-good) | Require unique variant names |
| [`musea/valid-variant`](#musea-valid-variant) | [Bad](#musea-valid-variant-bad) · [Good](#musea-valid-variant-good) | Require name attribute in &lt;variant&gt; blocks |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `css/no-display-none`

Suggest using v-show instead of display: none

[Bad](#css-no-display-none-bad) · [Good](#css-no-display-none-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**Bad**

The `.message` declaration hides the local paragraph through CSS rather than a template visibility condition.

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**Good**

`v-show="isSaved"` makes the visibility condition explicit on the local paragraph and removes `display: none`.

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [All rules](all.md)

### `css/no-hardcoded-values`

Suggest using CSS variables instead of hardcoded values

[Bad](#css-no-hardcoded-values-bad) · [Good](#css-no-hardcoded-values-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**Bad**

The button embeds spacing numbers and a hexadecimal color directly in the declarations.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**Good**

The declarations refer to named spacing and color custom properties, so these values can be maintained as tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [All rules](all.md)

### `css/no-id-selectors`

Discourage use of ID selectors in CSS

[Bad](#css-no-id-selectors-bad) · [Good](#css-no-id-selectors-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**Bad**

`#submit` ties the style rule to an ID selector.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**Good**

The `.submit` class expresses the reusable styling hook without an ID selector.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [All rules](all.md)

### `css/no-important`

Discourage use of !important in CSS

[Bad](#css-no-important-bad) · [Good](#css-no-important-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**Bad**

The color declaration overrides normal cascade priority with `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**Good**

The color comes from a custom property without an important declaration.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [All rules](all.md)

### `css/no-utility-classes`

Warn against implementing utility classes in component styles

[Bad](#css-no-utility-classes-bad) · [Good](#css-no-utility-classes-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**Bad**

The authored selectors use utility-shaped names such as `.flex`, `.mt-4`, and `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**Good**

A component-specific `.my-component` selector groups the component styling under one semantic name.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [All rules](all.md)

### `css/no-v-bind-performance`

Warn about performance cost of CSS v-bind()

[Bad](#css-no-v-bind-performance-bad) · [Good](#css-no-v-bind-performance-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**Bad**

The stylesheet reads the changing `offset` through the SFC CSS `v-bind()` mechanism.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**Good**

The element receives the changing transform directly through its style binding.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [All rules](all.md)

### `css/prefer-logical-properties`

Recommend CSS logical properties for better i18n support

[Bad](#css-prefer-logical-properties-bad) · [Good](#css-prefer-logical-properties-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**Bad**

`margin-left` fixes the margin to a physical side regardless of writing direction.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**Good**

`margin-inline-start` follows the start of the inline direction instead.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [All rules](all.md)

### `css/prefer-nested-selectors`

Recommend using CSS nesting for descendant selectors

[Bad](#css-prefer-nested-selectors-bad) · [Good](#css-prefer-nested-selectors-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**Bad**

The `.card .title` descendant selector repeats the parent selector in a flat rule.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**Good**

The `.title` rule is nested inside `.card`, keeping the parent-child styling relationship together.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [All rules](all.md)

### `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content

[Bad](#css-prefer-slotted-bad) · [Good](#css-prefer-slotted-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**Bad**

The scoped stylesheet targets the `slot` outlet rather than the elements supplied through the slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**Good**

`:slotted(.label)` targets the supplied label element through the scoped slot selector.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [All rules](all.md)

### `css/require-font-display`

Require font-display in @font-face rules

[Bad](#css-require-font-display-bad) · [Good](#css-require-font-display-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**Bad**

The font-face declaration defines the font source but omits its font-display policy.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**Good**

`font-display: swap` explicitly selects the fallback-to-font display policy.

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [All rules](all.md)

### `musea/no-empty-variant`

Disallow empty &lt;variant&gt; blocks

[Bad](#musea-no-empty-variant-bad) · [Good](#musea-no-empty-variant-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**Bad**

The named primary variant is empty, so it provides no preview content.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**Good**

The variant renders a primary Button with its Save content.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [All rules](all.md)

### `musea/prefer-design-tokens`

Prefer design token CSS variables over hardcoded primitive values

[Bad](#musea-prefer-design-tokens-bad) · [Good](#musea-prefer-design-tokens-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: See [typed options and defaults](options.md).

Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-prefer-design-tokens-bad"></span>

**Bad**

The art example uses the literal blue color instead of the configured primary design token.

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**Good**

The style refers to --color-primary, the token configured for this example.

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [All rules](all.md)

### `musea/require-component`

Require component attribute in &lt;art&gt; block

[Bad](#musea-require-component-bad) · [Good](#musea-require-component-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**Bad**

The art block supplies a title but does not identify the component being previewed.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**Good**

defineArt supplies ./Button.vue as the component for the art block.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [All rules](all.md)

### `musea/require-title`

Require title attribute in &lt;art&gt; block

[Bad](#musea-require-title-bad) · [Good](#musea-require-title-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**Bad**

The art block identifies Button.vue but supplies no title.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**Good**

The defineArt options supply the Button title for the art block.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [All rules](all.md)

### `musea/unique-variant-names`

Require unique variant names

[Bad](#musea-unique-variant-names-bad) · [Good](#musea-unique-variant-names-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**Bad**

Two variants in the same art block both use the primary name.

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**Good**

The variants have distinct primary and secondary names.

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [All rules](all.md)

### `musea/valid-variant`

Require name attribute in &lt;variant&gt; blocks

[Bad](#musea-valid-variant-bad) · [Good](#musea-valid-variant-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**Bad**

The variant omits the name needed to identify the preview.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**Good**

The primary name identifies that variant.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [All rules](all.md)
