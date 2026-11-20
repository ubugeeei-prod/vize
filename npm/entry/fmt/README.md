# @vizejs/fmt

The Vize formatter on its own: the `vzfmt` command and `formatSfc`.

```bash
vp install -D @vizejs/fmt
```

```bash
vzfmt --check src
vzfmt --write src
```

```ts
import { formatSfc } from "@vizejs/fmt";
```

`vzfmt` takes the same arguments as `vize fmt` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/formatter
