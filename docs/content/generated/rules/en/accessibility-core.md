---
title: "Accessibility core"
---

# Accessibility core

Use these rules to check the basic accessibility information in a Vue template: image alternatives,
link text, valid ARIA attributes, keyboard equivalents, and form labels. Start here when adding
images, links, or inputs to a component.

For example, open [form-control-has-label](./reference/a11y-form-control-has-label.md) before adding
an input: compare its Bad/Good examples, enable the rule with the shown Vite+ configuration, and
run `vp run lint`. The [complete accessibility guide](./accessibility.md) explains the broader rule set.

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [Bad](#a11y-alt-text-bad) · [Good](#a11y-alt-text-good) | Require alternative text for media elements |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [Bad](#a11y-anchor-has-content-bad) · [Good](#a11y-anchor-has-content-good) | Require anchor elements to have accessible content |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [Bad](#a11y-anchor-is-valid-bad) · [Good](#a11y-anchor-is-valid-good) | Enforce valid href on anchor elements |
| [`a11y/aria-props`](#a11y-aria-props) | [Bad](#a11y-aria-props-bad) · [Good](#a11y-aria-props-good) | Disallow invalid ARIA attributes |
| [`a11y/aria-role`](#a11y-aria-role) | [Bad](#a11y-aria-role-bad) · [Good](#a11y-aria-role-good) | Elements with ARIA roles must use a valid, non-abstract ARIA role |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [Bad](#a11y-aria-unsupported-elements-bad) · [Good](#a11y-aria-unsupported-elements-good) | Disallow ARIA attributes on elements that do not support them |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [Bad](#a11y-click-events-have-key-events-bad) · [Good](#a11y-click-events-have-key-events-good) | Require keyboard event handlers with click events |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [Bad](#a11y-form-control-has-label-bad) · [Good](#a11y-form-control-has-label-good) | Require form controls to have associated labels |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Require alternative text for media elements

[Bad](#a11y-alt-text-bad) · [Good](#a11y-alt-text-good)

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**Bad**

The image submit control supplies only its image URL; it has no `alt` text describing the action.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**Good**

`alt="Submit search"` gives the image control an accessible name that describes submitting the search.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [All rules](all.md)

### `a11y/anchor-has-content`

Require anchor elements to have accessible content

[Bad](#a11y-anchor-has-content-bad) · [Good](#a11y-anchor-has-content-good)

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**Bad**

The `/settings` link has no text or other naming content, so its destination has no accessible description.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**Good**

The visible `Settings` text supplies content for the same destination link.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [All rules](all.md)

### `a11y/anchor-is-valid`

Enforce valid href on anchor elements

[Bad](#a11y-anchor-is-valid-bad) · [Good](#a11y-anchor-is-valid-good)

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
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**Bad**

The first anchor uses `#` for an action; the second uses a JavaScript URL. Neither provides an ordinary navigation destination.

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**Good**

A native button performs `openPanel`, while the remaining anchor has the real `/docs/javascript-urls` destination.

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [All rules](all.md)

### `a11y/aria-props`

Disallow invalid ARIA attributes

[Bad](#a11y-aria-props-bad) · [Good](#a11y-aria-props-good)

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**Bad**

`aria-lable` is misspelled and is not a supported ARIA attribute.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**Good**

The supported `aria-label` attribute supplies the button name.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [All rules](all.md)

### `a11y/aria-role`

Elements with ARIA roles must use a valid, non-abstract ARIA role

[Bad](#a11y-aria-role-bad) · [Good](#a11y-aria-role-good)

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**Bad**

`datepicker` is not a recognized ARIA role for this section.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**Good**

The section uses the recognized `dialog` role and a label describing the date selection.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [All rules](all.md)

### `a11y/aria-unsupported-elements`

Disallow ARIA attributes on elements that do not support them

[Bad](#a11y-aria-unsupported-elements-bad) · [Good](#a11y-aria-unsupported-elements-good)

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
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**Bad**

The metadata element carries `aria-hidden`, although `meta` does not support ARIA attributes.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**Good**

Removing the ARIA attribute leaves the charset declaration intact.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [All rules](all.md)

### `a11y/click-events-have-key-events`

Require keyboard event handlers with click events

[Bad](#a11y-click-events-have-key-events-bad) · [Good](#a11y-click-events-have-key-events-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**Bad**

The non-interactive `div` has a click handler but no keyboard event handling.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**Good**

A native `button` provides keyboard activation for the same `activate` handler.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [All rules](all.md)

### `a11y/form-control-has-label`

Require form controls to have associated labels

[Bad](#a11y-form-control-has-label-bad) · [Good](#a11y-form-control-has-label-good)

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**Bad**

The search input has no label identifying what the user should enter.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**Good**

Wrapping the input in a label associates the visible `Search` text with the control.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [All rules](all.md)
