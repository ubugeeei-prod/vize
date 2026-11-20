# @vizejs/compile-jsx

The Vize JSX/TSX compiler on its own. `jsxMode: "vapor"` (or a `"use vue:vapor"` directive) compiles a component to Vapor.

```bash
vp install -D @vizejs/compile-jsx
```

```ts
import { compileJsx } from "@vizejs/compile-jsx";
```

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/jsx
