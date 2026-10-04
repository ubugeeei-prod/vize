# Sort imports in Vue scripts

Enable import sorting with `formatter.sortImports` in `vize.config.*`:

```ts
import { defineConfig } from "vize";

export default defineConfig({
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

An empty object selects Oxfmt's default groups and sorting policy. Omit the
option or set it to `false` to preserve authored import order. Both `<script>`
and `<script setup>`, including JSX/TSX blocks, use the native Oxc import sorter.
Direct `vize fmt` applies the same policy to standalone JS/TS/JSX/TSX files.

Vite+ shares `fmt.sortImports` with Vue formatting automatically. Set
`fmt.vize.sortImports` to override it, including `false` to disable sorting only
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
