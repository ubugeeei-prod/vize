# Vize builder matrix example

This example builds the same typed Vue SFC with Vize through three different hosts. It keeps the
application source shared so differences stay in the builder configuration instead of drifting
between copy-pasted demos.

| Builder | Command            | Vize adapter               | CSS path           |
| ------- | ------------------ | -------------------------- | ------------------ |
| esbuild | `vp run build`     | `@vizejs/unplugin/esbuild` | esbuild CSS bundle |
| webpack | `vp run build`     | `@vizejs/unplugin/webpack` | webpack native CSS |
| Bun     | `vp run build:bun` | `@vizejs/unplugin/bun`     | Bun CSS bundle     |

Run the Node-hosted matrix from the repository root:

```sh
vp run --filter @vizejs/example-builder-matrix build
```

With Bun installed, build the same source through Bun:

```sh
vp run --filter @vizejs/example-builder-matrix build:bun
```

Each command writes a self-contained preview under `dist/<builder>/index.html`. Serve `dist` with
any static file server. The package test builds esbuild and webpack concurrently and verifies both
JavaScript and scoped CSS output, so the checked-in configurations stay executable.

Vite has its own example in `npm/builder/vite/example`, while Rspack uses the dedicated example in
`npm/builder/rspack/example` because its loader and HMR contracts differ from webpack.
