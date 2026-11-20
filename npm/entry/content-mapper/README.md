# @vizejs/content-mapper

The Vize TypeScript content mapper on its own. Declare it in `tsconfig.json` and TypeScript checks `.vue` files directly.

```bash
vp install -D @vizejs/content-mapper
```

Requires TypeScript 7.1 beta or later. Check `tsc --version` before configuring a content mapper.

```json
{
  "contentMappers": [
    {
      "package": "@vizejs/content-mapper",
      "extensions": [".vue"]
    }
  ]
}
```

```bash
tsc --runExternalCode --noEmit -p tsconfig.json
```

TypeScript starts `vize-content-mapper`, the same server as `vize content-mapper`, which reads the same
`vite.config.*` and `tsconfig.json`.

Generated code inlines Vize's helper declarations. When `vize` is also a direct dependency of the
project, it imports the shared declarations from `vize` instead.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/content-mapper
