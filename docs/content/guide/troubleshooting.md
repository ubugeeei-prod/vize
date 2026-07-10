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

## Native Type Package Resolution

`vize check` resolves Vue and Vite type packages from the checked project before it uses bundled
fallbacks, so the project's own `vue`, `@vue/runtime-dom`, `@vue`, and `vite` versions drive the
generated virtual project. For unusual package-manager layouts, set `VIZE_VUE_PACKAGE`,
`VIZE_VUE_NAMESPACE_PACKAGE`, `VIZE_VUE_RUNTIME_DOM_PACKAGE`, or `VIZE_VITE_PACKAGE` to explicit
package roots. `VIZE_RUNTIME_NODE_MODULES` can also point at one or more `node_modules` roots as a
fallback search path.
