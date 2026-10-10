---
title: "Accessibility integrity"
---

# Accessibility integrity

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Bad](#a11y-no-role-presentation-on-focusable-bad) · [Good](#a11y-no-role-presentation-on-focusable-good) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Bad](#a11y-no-static-element-interactions-bad) · [Good](#a11y-no-static-element-interactions-good) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Bad](#a11y-placeholder-label-option-bad) · [Good](#a11y-placeholder-label-option-good) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Bad](#a11y-role-has-required-aria-props-bad) · [Good](#a11y-role-has-required-aria-props-good) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Bad](#a11y-tabindex-no-positive-bad) · [Good](#a11y-tabindex-no-positive-good) | Disallow positive tabindex values |
| [`a11y/use-list`](#a11y-use-list) | [Bad](#a11y-use-list-bad) · [Good](#a11y-use-list-good) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/no-role-presentation-on-focusable`

Disallow role="presentation" or role="none" on focusable elements

[Bad](#a11y-no-role-presentation-on-focusable-bad) · [Good](#a11y-no-role-presentation-on-focusable-good)

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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**Bad**

The focusable billing link requests role=presentation, which conflicts with its interactive link role; browsers must ignore that presentation request.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**Good**

Remove the conflicting presentation request and rely on the native link role and billing destination.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [All rules](all.md)

### `a11y/no-static-element-interactions`

Disallow event handlers on static elements

[Bad](#a11y-no-static-element-interactions-bad) · [Good](#a11y-no-static-element-interactions-good)

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**Bad**

A static section receives an Enter-key action without an interactive role.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**Good**

A native button carries the same action with an appropriate interactive element.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [All rules](all.md)

### `a11y/placeholder-label-option`

Require disabled or hidden on select placeholder option

[Bad](#a11y-placeholder-label-option-bad) · [Good](#a11y-placeholder-label-option-good)

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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**Bad**

The empty-value prompt remains selectable as if it were a country value.

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**Good**

Adding `disabled` distinguishes the prompt from the selectable Japan option.

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [All rules](all.md)

### `a11y/role-has-required-aria-props`

Require ARIA roles to have required properties

[Bad](#a11y-role-has-required-aria-props-bad) · [Good](#a11y-role-has-required-aria-props-good)

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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**Bad**

The checkbox role omits `aria-checked`, which conveys the checkbox state.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**Good**

`aria-checked="false"` supplies the state required by the checkbox role.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [All rules](all.md)

### `a11y/tabindex-no-positive`

Disallow positive tabindex values

[Bad](#a11y-tabindex-no-positive-bad) · [Good](#a11y-tabindex-no-positive-good)

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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**Bad**

A positive tabindex of 3 creates a custom focus order ahead of ordinary controls.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**Good**

The button uses its native focus order without a positive tabindex.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [All rules](all.md)

### `a11y/use-list`

Suggest using list elements for bullet-like text

[Bad](#a11y-use-list-bad) · [Good](#a11y-use-list-good)

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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**Bad**

The task items are separate paragraphs with typed dash markers rather than list elements.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**Good**

An unordered list and list items express the same tasks with list semantics.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [All rules](all.md)

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
