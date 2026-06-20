---
title: Troubleshooting
---

# Troubleshooting

## Template Syntax Modes

Vize defaults `compiler.templateSyntax` to `"standard"`. Standard mode accepts recoverable template
syntax problems, reports warnings, and rewrites them to valid output.

A common migration case is self-closing syntax on non-void HTML elements:

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` and `<span />` are not valid self-closing HTML elements. Standard mode rewrites them as
empty elements, equivalent to `<div></div>` and `<span></span>`, and emits a warning. Strict mode
reports them as errors. Quirks mode keeps them as self-closing leaves without a warning.

Prefer writing explicit end tags:

```vue
<template>
  <div></div>
  <span></span>
</template>
```

Choose a mode explicitly when migrating:

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      templateSyntax: "standard",
    }),
  ],
};
```

Use `"strict"` to fail on invalid syntax, or `"quirks"` when a project relies on Vue accepting those
tags as self-closing leaves. Valid void elements such as `<input />`, `<img />`, `<br />`, and
`<meta />` do not need quirks.
