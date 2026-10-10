---
title: "Vue Rules: Template Safety"
---

# Vue Rules: Template Safety

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good) | Disallow duplicate attributes on the same element |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good) | Disallow mustache interpolation in `<textarea>` |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Bad](#vue-no-unsafe-url-bad) · [Good](#vue-no-unsafe-url-good) | Warn about potentially unsafe URL bindings |
| [`vue/no-v-html`](#vue-no-v-html) | [Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good) | Warn against v-html to prevent XSS vulnerabilities |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good) | Disallow v-text / v-html on component elements |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good) | Enforce HTML content model rules |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-duplicate-attributes`

Disallow duplicate attributes on the same element

[Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**Bad**

The same button declares class twice instead of one combined class value.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Good**

Both class tokens appear in a single class attribute.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [All rules](all.md)

### `vue/no-textarea-mustache`

Disallow mustache interpolation in `<textarea>`

[Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**Bad**

The textarea places message in child interpolation instead of binding its value.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Good**

v-model binds the editable textarea value to message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [All rules](all.md)

### `vue/no-unsafe-url`

Warn about potentially unsafe URL bindings

[Bad](#vue-no-unsafe-url-bad) · [Good](#vue-no-unsafe-url-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**Bad**

The anchor destination begins with the executable javascript: scheme.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Good**

The anchor uses the ordinary local /next navigation destination.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [All rules](all.md)

### `vue/no-v-html`

Warn against v-html to prevent XSS vulnerabilities

[Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**Bad**

v-html interprets content as HTML rather than ordinary text.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Good**

Mustache interpolation displays content as escaped text instead of injecting HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [All rules](all.md)

### `vue/no-v-text-v-html-on-component`

Disallow v-text / v-html on component elements

[Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**Bad**

The component tag receives v-html or v-text, which replaces element content rather than supplying component slots.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Good**

Native HTML targets can receive the directives; MyComponent receives its content through the default slot.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [All rules](all.md)

### `vue/permitted-contents`

Enforce HTML content model rules

[Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good)

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
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**Bad**

The examples put block content in p, omit the table body, nest interactive controls, or put a div directly inside ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Good**

The examples use inline paragraph content, an explicit tbody, and li children. The custom MyItem is not treated as a known native ul child.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [All rules](all.md)

### `vue/use-unique-element-ids`

Enforce unique element IDs using useId() instead of static literals

[Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good)

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
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**Bad**

The literal `email` ID is reused by every instance of this component, which can misdirect its label when several instances are rendered.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Good**

`useId()` produces the instance's `emailId`; bind the same value to the label's `for` and the input's `id`.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [All rules](all.md)
