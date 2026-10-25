# @vizejs/babel-plugin

Compile Vue JSX and TSX with the shared Vize compiler inside Babel.

```sh
pnpm add -D @babel/core @vizejs/babel-plugin
```

The native Vize compiler is a normal package dependency. Babel is the host peer;
the application supplies its Vue runtime. Vapor output requires a Vue build that
provides the Vapor runtime APIs.

```js
// babel.config.mjs
export default {
  plugins: [["@vizejs/babel-plugin", { jsxMode: "vdom" }]],
  sourceMaps: true,
};
```

Use `@babel/preset-typescript` for type stripping and the usual Babel plugins for
decorators, language targets, or CommonJS output. The plugin enables JSX parsing
and TSX parsing for `.tsx` filenames. It leaves plain `.ts` parsing to the host.

## Options

| Option         | Default    | Behavior                                                                                                       |
| -------------- | ---------- | -------------------------------------------------------------------------------------------------------------- |
| `jsxMode`      | `"vdom"`   | Default client renderer: `"vdom"` or `"vapor"`. Component mode directives retain their native meaning.         |
| `ssr`          | `false`    | Generate SSR output for the selected client renderer.                                                          |
| `jsxCompat`    | `"native"` | Select native or `"babel"` compatibility semantics. Incompatible combinations surface the compiler diagnostic. |
| `scopedStyles` | `"error"`  | `"metadata"` lets a bundler consume extracted CSS explicitly.                                                  |

Keep client and server builds on the same renderer setting. Unsupported native
source shapes surface as compilation errors.

## Babel composition

The compiler runs in a `pre` hook. It serializes the current AST, including edits
from earlier `pre` hooks, and compiles that file once. Babel's normal visitors then
process the generated module. Program edits and the host's TypeScript, decorator,
and module transformations keep their place in that pipeline.

Files without JSX return before loading the native addon. JSX-producing visitor
transforms belong in an earlier Babel pass; introducing new JSX after native
compilation produces an explicit error.

Use Babel `overrides`, `only`, and `ignore` to select Vue files in mixed projects.
An explicit foreign JSX runtime or factory is a configuration error.

## Source maps

Enable Babel `sourceMaps` normally. The plugin composes the native map with
Babel's current-AST map and any upstream input map, then lets Babel's normal
generator produce the final map. It does not replace the parser or generator.
A missing requested native map produces an error.

When calling `transformFromAstSync`, pass the original source to retain its
source content in the map.

## Extracted styles

Babel does not emit stylesheets. By default, extracted scoped JSX CSS produces
an error. A bundler can opt in and emit the returned styles:

```js
import * as babel from "@babel/core";
import vize from "@vizejs/babel-plugin";

const result = babel.transformSync(source, {
  filename: "Card.tsx",
  plugins: [[vize, { scopedStyles: "metadata" }]],
});

const styles = result.metadata.vize.jsx.scopedStyles;
// Each entry contains scopeId and already-scoped CSS.
```

The same metadata contains the configured default mode, SSR flag, native pass
count, and compiler warnings. Compiler warnings are also emitted as Node warnings.
