# SFC hot updates

[Package overview](../README.md)

Development builds use Rspack's `module.hot` and Vue's HMR runtime. Set the SFC
loader's `hotReload: false` to disable injection. Production and SSR builds omit
client HMR code.

| Change                                                                                   | Behavior                                                                                            |
| ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Style content; component JavaScript unchanged                                            | Preserve component state, including when `eval-source-map` changes the SFC module's embedded source |
| Template; script, setup shape, style structure and imported runtime values unchanged     | Update the render function and preserve component state                                             |
| CSS Module content                                                                       | Update the shared class bindings and rerender                                                       |
| Script or script dependency                                                              | Reload the component; local state resets                                                            |
| Style block structure, compilation mode, setup shape, or imported runtime values changed | Reload the component                                                                                |

Update history belongs to each browser module's HMR lifecycle. Compiler caches
do not decide which previous version a browser has applied. External script and
template files participate in the same comparison after the loader resolves
their contents. External style files remain style-module dependencies.

State preservation requires compiler hash metadata and a compatible VDOM render
function. Vapor, custom elements, custom blocks, and modules with side-effect-only
imports use the conservative component reload path. A template edit that changes
the generated setup function also reloads the component. JSX/TSX entry modules
are outside this SFC HMR contract.

### Browser regression tests

`test:hmr` runs a real dev server and Chromium against temporary fixtures. It
checks DOM text, local component state, computed CSS, CSS Module bindings, page
identity, and browser errors through repeated edits. Fixtures cover JavaScript
and TypeScript script setup, external Options API scripts/templates/styles,
script dependencies, and removal of style blocks. Source maps remain enabled.

Verified combinations use Vue 3.5.42 and Playwright 1.62.1:

| Rspack | Dev server | CSS routing                                                        |
| ------ | ---------- | ------------------------------------------------------------------ |
| 1.7.12 | 1.2.1      | Native CSS with `experiments.css`; automatic and manual CssExtract |
| 2.2.2  | 2.2.1      | Native CSS; automatic and manual CssExtract                        |

These versions record browser acceptance coverage, not a guarantee for every
release allowed by the peer range. The test uses an isolated dependency directory
so the workspace's Rspack version cannot replace the selected major:

```sh
# Run from npm/builder/rspack after building the workspace native package.
hmr_runtime=$(mktemp -d)
npm install --prefix "$hmr_runtime" --legacy-peer-deps \
  @rspack/core@2.2.2 @rspack/dev-server@2.2.1 \
  vue@3.5.42 playwright@1.62.1 css-loader@7.1.2
node "$hmr_runtime/node_modules/playwright/cli.js" install chromium
VIZE_HMR_RUNTIME="$hmr_runtime" pnpm test:hmr
```

For Rspack 1, select `@rspack/core@1.7.12` and `@rspack/dev-server@1.2.1` in a
separate dependency directory. `--legacy-peer-deps` accommodates css-loader
7.1.2's optional Rspack peer declaration, which predates Rspack 2. Tests load the
current plugin build from `dist`; they do not exercise npm tarball packaging.
