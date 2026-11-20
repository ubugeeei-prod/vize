# @vizejs/lint

The Vize linter on its own: the `vzlint` command and the lint Node API.

```bash
vp install -D @vizejs/lint
```

```bash
vzlint src
vzlint --fix src
```

```ts
import { lint } from "@vizejs/lint";
```

`vzlint` takes the same arguments as `vize lint` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/cli
