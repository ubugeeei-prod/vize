# @vizejs/compile

The Vize SFC and template compiler on its own: the `vzc` command (`vize build`) and the compile Node API.

```bash
vp install -D @vizejs/compile
```

```bash
vzc 'src/**/*.vue' -o dist
```

```ts
import { compile } from "@vizejs/compile";
```

`vzc` takes the same arguments as `vize build` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/cli
