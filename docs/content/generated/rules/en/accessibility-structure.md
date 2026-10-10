---
title: "Accessibility structure"
---

# Accessibility structure

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Bad](#a11y-heading-has-content-bad) · [Good](#a11y-heading-has-content-good) | Require heading elements to have accessible content |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Bad](#a11y-heading-levels-bad) · [Good](#a11y-heading-levels-good) | Disallow skipping heading levels |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Bad](#a11y-iframe-has-title-bad) · [Good](#a11y-iframe-has-title-good) | Require iframe elements to have a title attribute |
| [`a11y/img-alt`](#a11y-img-alt) | [Bad](#a11y-img-alt-bad) · [Good](#a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Bad](#a11y-interactive-supports-focus-bad) · [Good](#a11y-interactive-supports-focus-good) | Require interactive role elements to be focusable |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Bad](#a11y-label-has-for-bad) · [Good](#a11y-label-has-for-good) | Require labels to have associated form controls |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Bad](#a11y-landmark-roles-bad) · [Good](#a11y-landmark-roles-good) | Validate landmark role placement and uniqueness |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Bad](#a11y-media-has-caption-bad) · [Good](#a11y-media-has-caption-good) | Require media elements to have captions |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

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
