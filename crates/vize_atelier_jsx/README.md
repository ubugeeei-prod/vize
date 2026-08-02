<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_atelier_jsx.svg" alt="vize_atelier_jsx logo" width="120" height="120" /><br>
  vize_atelier_jsx
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_jsx` parses and lowers Vue JSX/TSX into Vize's shared compiler representation.

## Key Entry Points

- `compile_jsx`
- `lower_source`
- `compile_to_vdom`
- `compile_to_vapor`
- `compile_to_ssr`

## `@vue/babel-plugin-jsx` compatibility mode

Vize compiles JSX/TSX with its own semantics by default: a block tree, lowered
`v-if` / `v-for`, and patch flags on every node. `@vue/babel-plugin-jsx` emits
bare `createVNode` calls instead. For a project migrating off the babel plugin,
`compiler.jsxCompat: "babel"` asks for the plugin's semantics instead.

```jsonc
// vize.json
{ "compiler": { "jsxCompat": "babel" } }
```

The switch is **opt-in and off by default** — turning it on by default would
silently change output for every existing Vize user — and it is a project-level
setting with no per-component directive form, because it changes the shape of
the emitted module rather than of a single render function.

Its scope is JSX/TSX compilation: the key is read by this crate's JSX entry
points and by the JSX bindings that wrap them. `.vue` SFCs do not go through the
JSX compiler, so `jsxCompat` does not change how they are compiled.

### Plugin option mapping

The plugin's options have no config-file spelling; each is a Vize option or API
field on a `compile_jsx_with_babel_*` entry point, and all of them are inert
unless `jsxCompat` is `"babel"`.

| `@vue/babel-plugin-jsx` | Vize option / API                           |
| ----------------------- | ------------------------------------------- |
| `transformOn`           | `BabelJsxOptions::transform_on`             |
| `pragma`                | `compile_jsx_with_babel_pragma`             |
| `mergeProps`            | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`       | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`     | `compile_jsx_with_babel_object_slots`       |
| any combination         | `compile_jsx_with_babel_customizations`     |

`optimize` has no equivalent: Vize's output is always optimized, which is what
the plugin's `optimize: true` produces. `resolveType` is not implemented yet.

### Where it does not apply

- **Vapor output.** `@vue/babel-plugin-jsx` is a vdom-era plugin with no Vapor
  output shape, so `jsxCompat: "babel"` combined with `jsxMode: "vapor"` is
  rejected with a diagnostic rather than silently ignored.
- **SSR output.** The plugin's options describe client vnode trees, so SSR
  compilation withholds the whole Babel lane and uses Vize's own SSR semantics.

### How compatibility is measured

`tests/babel_compat/` records the output of the real plugin (pinned `2.0.1`) for
every case in `fixtures/corpus.json` and snapshots it beside Vize's, with an
explicit per-row verdict. `tests/BABEL_COMPAT_INVENTORY.md` is the prose form of
that table, including the deferred rows and the global divergences (module
shape, block tree, patch flags, un-lowered control flow) that hold for nearly
every row.

## License

MIT
