# @vizejs/turbopack-plugin

Native Vue SFC compilation for Next.js 16.4.3 and its Turbopack loader API.

Not yet published to npm: it is an experimental workspace package until it joins
the release.

```sh
npm install @vizejs/turbopack-plugin vue
```

```js
// next.config.mjs
import { withVize } from "@vizejs/turbopack-plugin";

export default withVize({
  // Your existing Next.js configuration.
});
```

The package compiles `.vue` modules through Vize's native compiler. Imported
components are Vue component definitions; mount them in a Vue application or in
a Vue island owned by your host application. They do not become React components.

`withVize` preserves host settings, adds `.vue` resolution and supplies Vue's
compile-time feature flags. Explicit `compiler.define` values take precedence.
For an existing Turbopack configuration, matching loader rules remain in order.
Avoid applying another SFC compiler to the same `.vue` request.

For custom configuration composition, `vizeTurbopack(options)` returns just the
`turbopack` configuration. Configure Vue's feature flags yourself in that case.

## Styles and preprocessing

Plain CSS, scoped styles, CSS Modules and named CSS Modules use Turbopack's CSS
pipeline. Relative `@import` and `url()` dependencies retain their stylesheet
location, including external `<style src>` files. Template image imports work
with both string URLs and the host's image metadata objects.

Configure preprocessors using ordinary host loaders. Vize scopes their resulting
CSS before Turbopack handles CSS Modules, assets and output.

```sh
npm install --save-dev sass sass-loader
```

```js
import { withVize } from "@vizejs/turbopack-plugin";

export default withVize(
  {},
  {
    sourceMap: true,
    styleLoaders: {
      scss: [{ loader: "sass-loader", options: { sourceMap: true } }],
    },
  },
);
```

Loader options must be serializable. Additional loaders in each array follow
normal webpack right-to-left execution order. The integration rejects style
languages without a configured pipeline.

## Server rendering and updates

Server requests compile server render functions; browser requests compile Vue
render functions. The source identity and scope attribute agree across both.
The integration uses Turbopack's ESM hot API and Vue's hot runtime. Template and
style updates retain component state when their ownership remains compatible;
script or imported-value changes reload the component.

For a server-rendered Vue island inside a React host, retain the initial HTML
after Vue hydrates it. Vue then owns the island's descendants. The
[consumer fixture](./test/fixtures/app/client.jsx) demonstrates this boundary.

## Options and current scope

- `root`: project root used for stable scope identities; defaults to the host root.
- `sourceMap`: emit authored source maps; defaults to `true`.
- `hotReload`: enable development updates; defaults to `true`.
- `compilerOptions`: additional native SFC compiler options.
- `styleLoaders`: host preprocessing pipelines keyed by style language.

The initial integration targets stable Vue virtual DOM output with inline
JavaScript/TypeScript scripts and HTML templates. Authored Vapor markers produce
an explicit error. External script/template files, template/script preprocessors, custom
block loaders and macro module loaders produce explicit errors. They are not
silently emitted as valid JavaScript.

Source maps retain authored spans through module and style assembly. Generated
imports and hot runtime code have no authored location. An edit that exceeds the
shared alignment budget returns no map rather than inventing a location.
