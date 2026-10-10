---
title: "Accessibility rules"
---

# Accessibility rules

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
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Bad](#a11y-heading-has-content-bad) · [Good](#a11y-heading-has-content-good) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Bad](#a11y-heading-levels-bad) · [Good](#a11y-heading-levels-good) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Bad](#a11y-iframe-has-title-bad) · [Good](#a11y-iframe-has-title-good) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](#a11y-img-alt) | [Bad](#a11y-img-alt-bad) · [Good](#a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Bad](#a11y-interactive-supports-focus-bad) · [Good](#a11y-interactive-supports-focus-good) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Bad](#a11y-label-has-for-bad) · [Good](#a11y-label-has-for-good) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Bad](#a11y-landmark-roles-bad) · [Good](#a11y-landmark-roles-good) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Bad](#a11y-media-has-caption-bad) · [Good](#a11y-media-has-caption-good) | Require media elements to have captions |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Bad](#a11y-mouse-events-have-key-events-bad) · [Good](#a11y-mouse-events-have-key-events-good) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Bad](#a11y-no-access-key-bad) · [Good](#a11y-no-access-key-good) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Bad](#a11y-no-aria-hidden-on-focusable-bad) · [Good](#a11y-no-aria-hidden-on-focusable-good) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Bad](#a11y-no-autofocus-bad) · [Good](#a11y-no-autofocus-good) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Bad](#a11y-no-distracting-elements-bad) · [Good](#a11y-no-distracting-elements-good) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Bad](#a11y-no-i-for-icon-bad) · [Good](#a11y-no-i-for-icon-good) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Bad](#a11y-no-redundant-roles-bad) · [Good](#a11y-no-redundant-roles-good) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Bad](#a11y-no-refer-to-non-existent-id-bad) · [Good](#a11y-no-refer-to-non-existent-id-good) | Disallow references to non-existent IDs |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Bad](#a11y-no-role-presentation-on-focusable-bad) · [Good](#a11y-no-role-presentation-on-focusable-good) | Disallow role="presentation" or role="none" on focusable elements |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Bad](#a11y-no-static-element-interactions-bad) · [Good](#a11y-no-static-element-interactions-good) | Disallow event handlers on static elements |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Bad](#a11y-placeholder-label-option-bad) · [Good](#a11y-placeholder-label-option-good) | Require disabled or hidden on select placeholder option |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Bad](#a11y-role-has-required-aria-props-bad) · [Good](#a11y-role-has-required-aria-props-good) | Require ARIA roles to have required properties |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Bad](#a11y-tabindex-no-positive-bad) · [Good](#a11y-tabindex-no-positive-good) | Disallow positive tabindex values |
| [`a11y/use-list`](#a11y-use-list) | [Bad](#a11y-use-list-bad) · [Good](#a11y-use-list-good) | Suggest using list elements for bullet-like text |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

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

### `a11y/heading-has-content`

Require heading elements to have accessible content

[Bad](#a11y-heading-has-content-bad) · [Good](#a11y-heading-has-content-good)

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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**Bad**

The `h2` contributes a heading level but has no heading content.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**Good**

`Billing settings` supplies the content of the existing level-two heading.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [All rules](all.md)

### `a11y/heading-levels`

Disallow skipping heading levels

[Bad](#a11y-heading-levels-bad) · [Good](#a11y-heading-levels-good)

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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**Bad**

The heading sequence jumps directly from `h1` to `h3`, skipping level two.

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**Good**

Changing the billing heading to `h2` preserves a consecutive heading hierarchy.

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [All rules](all.md)

### `a11y/iframe-has-title`

Require iframe elements to have a title attribute

[Bad](#a11y-iframe-has-title-bad) · [Good](#a11y-iframe-has-title-good)

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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**Bad**

The checkout frame has a source URL but no `title` describing the embedded content.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**Good**

`title="Checkout preview"` names the content of that frame.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [All rules](all.md)

### `a11y/img-alt`

Require alt attribute on images for accessibility

[Bad](#a11y-img-alt-bad) · [Good](#a11y-img-alt-good)

Default severity: `warning`  
Presets: _none_  
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**Bad**

The avatar image is missing its `alt` attribute.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**Good**

`alt="User avatar"` supplies a text alternative for the avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [All rules](all.md)

### `a11y/interactive-supports-focus`

Require interactive role elements to be focusable

[Bad](#a11y-interactive-supports-focus-bad) · [Good](#a11y-interactive-supports-focus-good)

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**Bad**

Giving a `span` the button role and a click handler does not make the element keyboard-focusable.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**Good**

The native button is focusable and retains the same `open` action.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [All rules](all.md)

### `a11y/label-has-for`

Require labels to have associated form controls

[Bad](#a11y-label-has-for-bad) · [Good](#a11y-label-has-for-good)

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**Bad**

The separate label is neither associated through `for` nor wrapped around the input.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**Good**

`for="email"` matches the input ID and explicitly associates the two elements.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [All rules](all.md)

### `a11y/landmark-roles`

Validate landmark role placement and uniqueness

[Bad](#a11y-landmark-roles-bad) · [Good](#a11y-landmark-roles-good)

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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**Bad**

Two `main` elements declare duplicate main landmarks in the same template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**Good**

The dashboard remains the main landmark; the settings area becomes a named navigation landmark.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [All rules](all.md)

### `a11y/media-has-caption`

Require media elements to have captions

[Bad](#a11y-media-has-caption-bad) · [Good](#a11y-media-has-caption-good)

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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**Bad**

The video has playback controls but no caption track.

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**Good**

A `track` with `kind="captions"` supplies the English captions for the same video.

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [All rules](all.md)

### `a11y/mouse-events-have-key-events`

Require focus/blur events with mouse events

[Bad](#a11y-mouse-events-have-key-events-bad) · [Good](#a11y-mouse-events-have-key-events-good)

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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**Bad**

Preview visibility changes only through mouse enter and leave handlers.

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**Good**

The same preview actions run on focus and blur, and the button can receive keyboard focus.

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [All rules](all.md)

### `a11y/no-access-key`

Disallow the use of the accesskey attribute

[Bad](#a11y-no-access-key-bad) · [Good](#a11y-no-access-key-good)

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**Bad**

The `accesskey="s"` shortcut may conflict with browser or assistive-technology shortcuts.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**Good**

Removing `accesskey` keeps the ordinary Save button available.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [All rules](all.md)

### `a11y/no-aria-hidden-on-focusable`

Disallow aria-hidden="true" on focusable elements

[Bad](#a11y-no-aria-hidden-on-focusable-bad) · [Good](#a11y-no-aria-hidden-on-focusable-good)

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**Bad**

The focusable Close button is hidden from the accessibility tree with `aria-hidden="true"`.

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**Good**

The button remains exposed and receives a `Close` label instead of being hidden.

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [All rules](all.md)

### `a11y/no-autofocus`

Disallow the use of the autofocus attribute

[Bad](#a11y-no-autofocus-bad) · [Good](#a11y-no-autofocus-good)

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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**Bad**

The input requests automatic focus when it appears.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**Good**

Removing `autofocus` avoids this automatic focus request while retaining the query input.

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [All rules](all.md)

### `a11y/no-distracting-elements`

Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;

[Bad](#a11y-no-distracting-elements-bad) · [Good](#a11y-no-distracting-elements-good)

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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**Bad**

The `marquee` element introduces automatically moving text.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**Good**

A paragraph displays the same offer without the distracting marquee element.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [All rules](all.md)

### `a11y/no-i-for-icon`

Disallow using &lt;i&gt; element for icons

[Bad](#a11y-no-i-for-icon-bad) · [Good](#a11y-no-i-for-icon-good)

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**Bad**

The icon is rendered through `i`, whose text semantics do not describe an icon-only action.

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**Good**

A decorative span hides the icon glyph, while the separate `Delete item` text names the button action.

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [All rules](all.md)

### `a11y/no-redundant-roles`

Disallow redundant ARIA roles

[Bad](#a11y-no-redundant-roles-bad) · [Good](#a11y-no-redundant-roles-good)

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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**Bad**

The native button already has the button role, so `role="button"` repeats its implicit semantics.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**Good**

Removing the repeated role keeps the button semantics supplied by HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [All rules](all.md)

### `a11y/no-refer-to-non-existent-id`

Disallow references to non-existent IDs

[Bad](#a11y-no-refer-to-non-existent-id-bad) · [Good](#a11y-no-refer-to-non-existent-id-good)

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
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**Bad**

`aria-labelledby` points to `save-label`, but no element declares that ID.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**Good**

Adding the matching span resolves the reference and provides the button label.

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [All rules](all.md)

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
