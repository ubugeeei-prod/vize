---
title: "Accessibility interactions"
---

# Accessibility interactions

Use these rules when reviewing how people interact with a component: keyboard access, focus,
mouse handlers, hidden controls, and references to other elements. Check them when introducing
custom controls or changing visibility and event handling.

For example, a focusable element hidden from assistive technology is covered by
[no-aria-hidden-on-focusable](./reference/a11y-no-aria-hidden-on-focusable.md).
Compare its Bad/Good examples, use the shown configuration, then run `vp run lint`.
The [complete accessibility guide](./accessibility.md) explains how these checks fit together.

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Bad](#a11y-mouse-events-have-key-events-bad) · [Good](#a11y-mouse-events-have-key-events-good) | Require focus/blur events with mouse events |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Bad](#a11y-no-access-key-bad) · [Good](#a11y-no-access-key-good) | Disallow the use of the accesskey attribute |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Bad](#a11y-no-aria-hidden-on-focusable-bad) · [Good](#a11y-no-aria-hidden-on-focusable-good) | Disallow aria-hidden="true" on focusable elements |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Bad](#a11y-no-autofocus-bad) · [Good](#a11y-no-autofocus-good) | Disallow the use of the autofocus attribute |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Bad](#a11y-no-distracting-elements-bad) · [Good](#a11y-no-distracting-elements-good) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Bad](#a11y-no-i-for-icon-bad) · [Good](#a11y-no-i-for-icon-good) | Disallow using &lt;i&gt; element for icons |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Bad](#a11y-no-redundant-roles-bad) · [Good](#a11y-no-redundant-roles-good) | Disallow redundant ARIA roles |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Bad](#a11y-no-refer-to-non-existent-id-bad) · [Good](#a11y-no-refer-to-non-existent-id-good) | Disallow references to non-existent IDs |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

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
