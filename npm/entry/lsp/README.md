# @vizejs/lsp

The Vize language server on its own: `vzlsp` speaks the Language Server Protocol over stdio.

```bash
vp install -D @vizejs/lsp
```

```bash
vzlsp
```

`vzlsp` takes the same arguments as `vize lsp` and reads the same `vite.config.*` and `tsconfig.json`.

This package is one focused entry point into [Vize](https://github.com/ubugeeei-prod/vize). It runs the same release as the `vize` package; install `vize` instead when you use several tools. Guide: https://vizejs.dev/guide/language-server
