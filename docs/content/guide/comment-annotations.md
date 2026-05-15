---
title: Comment Annotations
---

# Comment Annotations

Vize provides comment-based annotations to control linting, diagnostics, and codegen behavior. There are two annotation systems depending on where they are used:

- **`<!-- @vize:xxx -->`** — HTML comments in `<template>` (Patina linter directives)
- **`// @vize forget: reason`** — JS comments in `<script>` (cross-file analysis suppression)

All `@vize:` template directives are **stripped from build output** — they never appear in production code.

## Template Directives (`@vize:`)

Used inside `<template>` as HTML comments. These control Patina (the built-in linter) behavior.

### `@vize:expected`

Expect a diagnostic on the next line. If no diagnostic is produced, this is a no-op. Similar to `@ts-expect-error`.

```vue
<template>
  <ul>
    <!-- @vize:expected -->
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
```

### `@vize:ignore-start` / `@vize:ignore-end`

Suppress all diagnostics within a region.

```vue
<template>
  <!-- @vize:ignore-start -->
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <!-- @vize:ignore-end -->
</template>
```

### `@vize:level(warn|error|off)`

Override the severity of diagnostics on the next line.

```vue
<template>
  <!-- @vize:level(warn) -->
  <img src="/photo.png" />

  <!-- @vize:level(off) -->
  <li v-for="item in items">{{ item }}</li>
</template>
```

| Value   | Effect               |
| ------- | -------------------- |
| `warn`  | Downgrade to warning |
| `error` | Upgrade to error     |
| `off`   | Suppress entirely    |

### `@vize:todo`

Emit a TODO warning.

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

Emit a FIXME error.

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

Emit a deprecation warning.

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

Documentation comment. No lint effect.

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

Mark a node to be stripped in production builds, kept in development.

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### Summary

| Directive                | Effect                             | Severity |
| ------------------------ | ---------------------------------- | -------- |
| `@vize:expected`         | Expect diagnostic on next line     | —        |
| `@vize:ignore-start/end` | Suppress all diagnostics in region | —        |
| `@vize:level(...)`       | Override next-line severity        | —        |
| `@vize:todo <msg>`       | Emit TODO                          | Warning  |
| `@vize:fixme <msg>`      | Emit FIXME                         | Error    |
| `@vize:deprecated <msg>` | Emit deprecation notice            | Warning  |
| `@vize:docs <text>`      | Documentation (no lint effect)     | —        |
| `@vize:dev-only`         | Strip in production                | —        |

## Script Suppression (`@vize forget`)

Used inside `<script>` as JS comments. Suppresses cross-file analysis warnings (Croquis) on the next line.

### Syntax

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

A **reason is required** — you must explain why the suppression is needed.

### Example

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

Without the annotation, Vize would warn that destructuring a reactive `inject()` return value breaks reactivity tracking.

### Rules

| Rule            | Description                                                          |
| --------------- | -------------------------------------------------------------------- |
| Reason required | `// @vize forget` without a reason is an error                       |
| Colon required  | Must use `// @vize forget: <reason>` (colon before reason)           |
| Next line only  | Applies to the next non-comment, non-empty line                      |
| No orphans      | A suppression at the end of a file with no code after it is an error |

### Multiple Suppressions

Each `@vize forget` applies independently to the next code line:

```vue
<script setup>
import { inject } from "vue";

// @vize forget: one-time read for display name
const { name } = inject("user");

// @vize forget: static config value
const { theme } = inject("config");
</script>
```

### Skipping Comments

The suppression targets the next **code** line, skipping over comments and blank lines:

```vue
<script setup>
// @vize forget: read-only access
// This comment is skipped
const { count } = inject("state");
</script>
```

### Common Reasons

| Reason                       | When to Use                        |
| ---------------------------- | ---------------------------------- |
| `intentionally non-reactive` | Value doesn't need to be reactive  |
| `read-only access`           | Only reading, not tracking changes |
| `legacy code`                | Known issue, will refactor later   |
| `third-party integration`    | Required by external library       |

### Invalid Examples

```typescript
// @vize forget
const { count } = inject("state");
// ^ Error: requires a reason

// @vize forget because I said so
const { count } = inject("state");
// ^ Error: requires a colon before the reason

// @vize forget:
const { count } = inject("state");
// ^ Error: reason cannot be empty
```
