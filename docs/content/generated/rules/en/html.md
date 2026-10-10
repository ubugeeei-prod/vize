---
title: "HTML rules"
---

# HTML rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Bad](#html-deprecated-attr-bad) · [Good](#html-deprecated-attr-good) | Disallow deprecated HTML attributes |
| [`html/deprecated-element`](#html-deprecated-element) | [Bad](#html-deprecated-element-bad) · [Good](#html-deprecated-element-good) | Disallow deprecated HTML elements |
| [`html/id-duplication`](#html-id-duplication) | [Bad](#html-id-duplication-bad) · [Good](#html-id-duplication-good) | Disallow duplicate element IDs |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Bad](#html-no-consecutive-br-bad) · [Good](#html-no-consecutive-br-good) | Disallow consecutive &lt;br&gt; elements |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Bad](#html-no-dupe-style-properties-bad) · [Good](#html-no-dupe-style-properties-good) | Disallow duplicate properties in inline style attributes |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Bad](#html-no-duplicate-class-bad) · [Good](#html-no-duplicate-class-good) | Disallow duplicate class names in a static class attribute |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Bad](#html-no-duplicate-dt-bad) · [Good](#html-no-duplicate-dt-good) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt; |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Bad](#html-no-empty-palpable-content-bad) · [Good](#html-no-empty-palpable-content-good) | Disallow empty elements that expect visible content |
| [`html/require-datetime`](#html-require-datetime) | [Bad](#html-require-datetime-bad) · [Good](#html-require-datetime-good) | Require datetime attribute on &lt;time&gt; element |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `html/deprecated-attr`

Disallow deprecated HTML attributes

[Bad](#html-deprecated-attr-bad) · [Good](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**Bad**

The paragraph uses the deprecated presentational `align` attribute.

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**Good**

The class and `text-align: center` declaration express the alignment through CSS.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [All rules](all.md)

### `html/deprecated-element`

Disallow deprecated HTML elements

[Bad](#html-deprecated-element-bad) · [Good](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**Bad**

The `center` element uses a deprecated HTML presentation element.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**Good**

A section and a styling class replace the deprecated element while preserving the content.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [All rules](all.md)

### `html/id-duplication`

Disallow duplicate element IDs

[Bad](#html-id-duplication-bad) · [Good](#html-id-duplication-good)

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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**Bad**

Both the input and help paragraph declare `id="email"`, so the label target is ambiguous.

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**Good**

The input keeps `email`; the help paragraph uses `email-help`, and aria-describedby refers to that distinct ID.

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [All rules](all.md)

### `html/no-consecutive-br`

Disallow consecutive &lt;br&gt; elements

[Bad](#html-no-consecutive-br-bad) · [Good](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**Bad**

Two consecutive break elements create spacing between blocks inside a single paragraph.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**Good**

Separate paragraphs express the two content blocks without repeated break elements.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [All rules](all.md)

### `html/no-dupe-style-properties`

Disallow duplicate properties in inline style attributes

[Bad](#html-no-dupe-style-properties-bad) · [Good](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**Bad**

Each static style repeats one property; `margin` and `MARGIN` also count as the same property.

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**Good**

The static style uses distinct color and background properties. Dynamic style bindings are outside this static-attribute check.

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [All rules](all.md)

### `html/no-duplicate-class`

Disallow duplicate class names in a static class attribute

[Bad](#html-no-duplicate-class-bad) · [Good](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**Bad**

The static class list repeats the `btn` token.

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**Good**

The class list keeps one `btn` token and the distinct `primary` token.

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [All rules](all.md)

### `html/no-duplicate-dt`

Disallow duplicate &lt;dt&gt; names in &lt;dl&gt;

[Bad](#html-no-duplicate-dt-bad) · [Good](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**Bad**

The same definition list repeats the `API` term for two descriptions.

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**Good**

One API term is followed by both descriptions, avoiding the repeated term.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [All rules](all.md)

### `html/no-empty-palpable-content`

Disallow empty elements that expect visible content

[Bad](#html-no-empty-palpable-content-bad) · [Good](#html-no-empty-palpable-content-good)

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
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**Bad**

The paragraph, list item, and table cell all have empty palpable content.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**Good**

Text fills the paragraph, interpolation supplies the list item, and aria-label explicitly names the otherwise empty cell.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [All rules](all.md)

### `html/require-datetime`

Require datetime attribute on &lt;time&gt; element

[Bad](#html-require-datetime-bad) · [Good](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**Bad**

The time element contains a human-readable date but no machine-readable datetime value.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**Good**

`datetime="2026-05-13"` supplies the corresponding machine-readable date.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [All rules](all.md)
