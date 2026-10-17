# Sort imports in Vue scripts

Sort Vue script imports consistently with the rest of a Vite+ project. Use this when you want stable import groups, while keeping side-effect initialization order unless you deliberately opt in.

Keep shared options in `vue.formatter` in `vite.config.*`; the CLI and editor read the same settings. See [Configuration](./configuration.md).

## Configure import sorting

Enable import sorting with `vue.formatter.sortImports` in your existing Vite+ configuration:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    sortImports: {
      internalPattern: ["@/", "~/"],
      groups: ["builtin", "external", "internal", ["parent", "sibling", "index"]],
      newlinesBetween: true,
      order: "asc",
    },
  },
});
```

An empty object selects Oxfmt's default groups and sorting policy. Set the
option to `false` to preserve authored import order. Omitting it also preserves
the order when no shared `fmt.sortImports` policy is configured. Both `<script>`
and `<script setup>`, including JSX/TSX blocks, use the native Oxc import sorter.
Direct `vize fmt` applies the same policy to standalone JS/TS/JSX/TSX files.

Vite+ shares `fmt.sortImports` with Vue formatting automatically. Set
`vue.formatter.sortImports` to override it, including `false` to disable sorting only
for Vue. Oxfmt still receives its original setting.

Supported options are `groups`, `customGroups`, `internalPattern`, `order`,
`ignoreCase`, `newlinesBetween`, `partitionByNewline`, `partitionByComment`, and
`sortSideEffects`. Group entries can be names, arrays of names, or
`{ newlinesBetween: boolean }` boundaries. Custom groups accept `groupName`,
`elementNamePattern`, `selector`, and `modifiers` as in Oxfmt.

Side-effect imports retain their authored order by default. Set
`sortSideEffects: true` only when rearranging initialization is intentional.
`partitionByNewline: true` requires `newlinesBetween: false` and cannot be
combined with boundary overrides. Invalid combinations fail before writing
files. `--no-config` ignores the configured sorting policy.

Native `formatSfc` and WASM `formatSfc` / `formatScript` also accept
`sortImports` in their options. The existing Rust `FormatOptions` remains source
compatible; use `resolve_sort_imports` with `GlyphFormatter::with_sort_imports`
or `format_script_with_sort_imports` for the additive Rust API.

[Configure line endings](./formatter-line-endings.md) with the same `endOfLine`
modes across CLI, editor, Node and WASM formatting.

[Configure property quotes](./formatter-property-quotes.md) for script object keys.

[Configure JSX attribute quotes](./formatter-jsx-quotes.md) independently of JavaScript strings.

For space and tab indentation, see [Indentation width](./formatter-indent-width.md).

The CLI and editor read the same `vue.formatter` settings from `vite.config.*`. See [Configuration](./configuration.md) for discovery and migration from removed dedicated configuration files.
