---
title: HTML Rules
---

# HTML Rules

These rules cover HTML validity and semantic markup inside Vue templates. They are separate from
Vue-specific directive rules and from accessibility rules so HTML conformance checks can be enabled
or explained on their own.

## `html/id-duplication`

Reports duplicate static IDs inside one template.

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

Good:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

Reports deprecated HTML elements.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <center>Profile</center>
</template>
```

Good:

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

Reports deprecated HTML attributes.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

Good:

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

Reports consecutive `<br>` elements used for layout.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

Good:

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

Requires machine-readable `datetime` values on `<time>`.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

Good:

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## `html/no-duplicate-dt`

Reports duplicate `<dt>` terms inside the same `<dl>`.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

Good:

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

## `html/no-empty-palpable-content`

Reports empty elements that are expected to expose visible or otherwise perceivable content.
Elements with text, child content, `aria-label`, `aria-labelledby`, `v-html`, or `v-text` are
accepted.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

Good:

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```
