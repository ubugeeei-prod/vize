# @vizejs/typecheck

The Vize type checker on its own: the `vzcheck` command and the type-check Node API.

```bash
vp install -D @vizejs/typecheck
```

```bash
vzcheck
vzcheck -p tsconfig.app.json
```

```ts
import { typeCheck } from "@vizejs/typecheck";
```

`vzcheck` takes the same arguments as `vize check` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/type-checking
