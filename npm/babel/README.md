# @vizejs/babel-plugin

Compile Vue JSX and TSX with the shared Vize compiler inside Babel.

```sh
pnpm add -D @babel/core @vizejs/babel-plugin
```

The native Vize compiler is a normal package dependency. Babel is the host peer;
the application supplies its Vue runtime. The current Vapor preview targets
Vue 3.6.0-rc.11 and its build containing the Vapor runtime APIs. Select that
build through the application's normal module resolution configuration and
use `createVaporApp` or `createVaporSSRApp` for the client entry.

The runtime conformance matrix uses Vue 3.5.44 for VDOM. Historical 3.6.0-rc.9
fixtures remain an unsupported ABI frontier with their failed and reference
packets retained separately.

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

## Component authoring

Vapor and setup SSR output use component objects. A block arrow keeps setup
state inside the component; declare props with a named identifier and an inline
object type:

```tsx
import { ref } from "vue";

export const Counter = (props: { label: string }) => {
  const count = ref(0);
  const increment = () => {
    count.value++;
  };
  return <button onClick={increment}>{props.label + ":" + count.value}</button>;
};
```

The preview rejects destructured props in these component objects. Use their
bindings as JSX component tags or direct exports; calls, callable aliases,
introspection, and escaping component values require a separate callable
representation and produce a diagnostic. Nested JSX that an attribute expression
cannot lower also produces a diagnostic.

VDOM client output supports expression arrows. SSR output for an expression
functional component is an explicit unsupported role; use a block setup component
for the paired client/server build.

## Babel composition

The compiler runs in a `pre` hook. It serializes the current AST, including edits
from earlier `pre` hooks, and compiles that file once. Babel's normal visitors then
process the generated module. Program edits and the host's TypeScript, decorator,
and module transformations keep their place in that pipeline.

Files without JSX return before loading the native addon. JSX-producing visitor
transforms belong in an earlier Babel pass; introducing new JSX after native
compilation produces an explicit error.
The final generation boundary also checks for JSX introduced by later `post`
hooks. That guard returns control to Babel without claiming code generation;
other generator overrides keep Babel's normal collision rules.

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
