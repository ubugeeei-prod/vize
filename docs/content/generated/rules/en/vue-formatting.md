---
title: "Vue Rules: Template Formatting"
---

# Vue Rules: Template Formatting

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good) | Enforce attribute naming style on custom components |
| [`vue/attribute-order`](#vue-attribute-order) | [Bad](#vue-attribute-order-bad) · [Good](#vue-attribute-order-good) | Enforce a consistent order of attributes |
| [`vue/html-quotes`](#vue-html-quotes) | [Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good) | Enforce quotes style of HTML attributes |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good) | Enforce self-closing style |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good) | Enforce consistent spacing inside mustache interpolations |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Bad](#vue-no-boolean-attr-value-bad) · [Good](#vue-no-boolean-attr-value-good) | Disallow explicit values for boolean HTML attributes |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Bad](#vue-no-inline-style-bad) · [Good](#vue-no-inline-style-good) | Discourage use of inline style attributes |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good) | Disallow multiple consecutive spaces |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good) | Recommend shorthand syntax for props (Vue 3.4+) |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good) | Enforce a casing for declared prop names |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good) | Require valid attribute names |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/attribute-hyphenation`

Enforce attribute naming style on custom components

[Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**Bad**

The component attribute uses the camelCase spelling firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Good**

The first-name spelling follows the configured hyphenated component-attribute convention.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [All rules](all.md)

### `vue/attribute-order`

Enforce a consistent order of attributes

[Bad](#vue-attribute-order-bad) · [Good](#vue-attribute-order-good)

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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**Bad**

The event handler appears before the structural v-if directive and ordinary id attribute.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Good**

v-if comes first, followed by id and then the event handler, following the rule ordering.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [All rules](all.md)

### `vue/html-quotes`

Enforce quotes style of HTML attributes

[Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good)

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
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**Bad**

The attributes use single quotes or no quotes instead of the double-quote convention.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Good**

Both ordinary attributes and directive expressions use double quotes.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [All rules](all.md)

### `vue/html-self-closing`

Enforce self-closing style

[Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
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
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**Bad**

The empty component uses a closing pair, while void img and br elements omit the configured self-closing spelling.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Good**

The component and void elements use self-closing syntax; a div with content retains its closing tag.

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [All rules](all.md)

### `vue/mustache-interpolation-spacing`

Enforce consistent spacing inside mustache interpolations

[Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good)

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
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**Bad**

The text interpolation is missing a space at one or both delimiter boundaries.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Good**

Spaces separate the expression from both opening and closing mustache delimiters.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [All rules](all.md)

### `vue/no-boolean-attr-value`

Disallow explicit values for boolean HTML attributes

[Bad](#vue-no-boolean-attr-value-bad) · [Good](#vue-no-boolean-attr-value-good)

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
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**Bad**

The boolean disabled and checked attributes redundantly contain string values.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Good**

The presence of each boolean attribute expresses the same enabled state without a value.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [All rules](all.md)

### `vue/no-inline-style`

Discourage use of inline style attributes

[Bad](#vue-no-inline-style-bad) · [Good](#vue-no-inline-style-good)

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
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**Bad**

The static style attribute embeds the color declaration in the element.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Good**

Classes express the fixed color; the ratio-dependent width remains a dynamic style binding, outside the static-attribute check.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [All rules](all.md)

### `vue/no-multi-spaces`

Disallow multiple consecutive spaces

[Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good)

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
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**Bad**

Two spaces separate attributes or the element name and the first attribute.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Good**

Single spaces separate the same attributes.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [All rules](all.md)

### `vue/prefer-props-shorthand`

Recommend shorthand syntax for props (Vue 3.4+)

[Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good)

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
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**Bad**

Each binding repeats the corresponding variable name, including the camelCase equivalent of a hyphenated argument.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Good**

Vue 3.4+ same-name binding shorthand removes the repeated expressions; a different source variable such as bar remains explicit.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [All rules](all.md)

### `vue/prop-name-casing`

Enforce a casing for declared prop names

[Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Checks declared prop names, not the casing of attributes passed to a child.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**Bad**

The declared prop name user_name uses underscore-separated spelling.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Good**

The declaration and its template reference use the camelCase name userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [All rules](all.md)

### `vue/valid-attribute-name`

Require valid attribute names

[Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Bad diagnostic: `parser/template`

Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**Bad**

The quote inside `my"attr` makes the attribute name malformed. This example produces the parser's `parser/template` diagnostic rather than promising a separate rule diagnostic.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Good**

`my-attr` is a well-formed attribute name, so the template parser can read the attribute and its value.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [All rules](all.md)
