---
title: Troubleshooting
---

# Troubleshooting

## Strict HTML Parsing vs Vue Parser Quirks

Vize keeps `compiler.vueParserQuirks` disabled by default. In strict mode, the template parser treats
HTML syntax according to HTML tree-construction behavior instead of silently accepting every Vue
compiler edge case.

A common migration case is self-closing syntax on non-void HTML elements:

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` and `<span />` are not valid self-closing HTML elements. Strict mode treats the `/` as an
ignored self-closing flag on a start tag, so the parser may report a missing end tag and the following
nodes can become children of the open element.

Prefer writing explicit end tags:

```vue
<template>
  <div></div>
  <span></span>
</template>
```

If you are migrating existing templates that rely on Vue accepting those tags as self-closing leaves,
enable parser quirks:

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      vueParserQuirks: true,
    }),
  ],
};
```

This keeps invalid non-void HTML self-closing tags as leaf elements in the DOM, SSR, and Vapor
compiler paths. Valid void elements such as `<input />`, `<img />`, `<br />`, and `<meta />` do not
need quirks.

Use quirks as a compatibility switch, not as the preferred style for new code. Keeping strict mode on
makes parser diagnostics and HTML lint rules easier to reason about.
