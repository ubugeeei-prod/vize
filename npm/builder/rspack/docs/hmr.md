# Source maps and style hot updates

[Package overview](../README.md)

With `eval-source-map`, a style edit changes the full SFC source embedded in the
component's source map. Rspack can therefore replace the component module even
when its generated JavaScript has not changed.

The SFC loader preserves component state when the SFC source changes but its
generated JavaScript and imported runtime values remain identical. Previous
values live in each browser module's `module.hot.data`, independently of compiler
caches and other clients. CSS Modules retain writable mapping objects so references
captured by `useCssModule()` observe class additions, changes, and deletions.

Changed JavaScript, including template and script edits, reloads the component.
Vapor, custom elements, custom blocks and side-effect-only imports also use
component reload. An uninitialized circular import makes dependency comparison
unavailable and uses reload on later updates; it does not prevent initial loading.
Production and SSR output omit client HMR code.

## Browser regression tests

`test:hmr` uses a real dev server, Chromium and temporary SFC fixtures with
`eval-source-map` enabled. It checks state retention for scoped styles, default
and named CSS Modules, `useCssModule()` mappings, circular component imports,
script dependency updates, component reload, page identity and browser errors.
JavaScript and TypeScript fixtures cover Native CSS automatic rules and manually
routed CssExtract. Automatic CssExtract scoped rules require a separate loader
ordering correction; the manual configuration is in [Manual rules](./manual-rules.md).

Dependencies come from an isolated directory to select the Rspack major:

```sh
# Run from npm/builder/rspack after building the workspace native package.
hmr_runtime=$(mktemp -d)
npm install --prefix "$hmr_runtime" --legacy-peer-deps \
  @rspack/core@2.2.2 @rspack/dev-server@2.2.1 \
  vue@3.5.42 playwright@1.62.1 css-loader@7.1.2
node "$hmr_runtime/node_modules/playwright/cli.js" install chromium
VIZE_HMR_RUNTIME="$hmr_runtime" pnpm test:hmr
```

The Rspack 1 combination uses `@rspack/core@1.7.12` and
`@rspack/dev-server@1.2.1` in a separate directory. `--legacy-peer-deps`
accommodates css-loader 7.1.2's optional peer range, which predates Rspack 2.
Tests load the current plugin's `dist` and do not cover tarball installation.
These selected versions do not establish coverage of every allowed peer version.
