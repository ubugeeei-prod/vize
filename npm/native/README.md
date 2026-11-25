# @vizejs/native

Native Node.js bindings for the Vize Vue compiler via NAPI.

## Features

- **Maximum Performance** - Native Rust speed in Node.js
- **Multi-threaded** - Parallel compilation support
- **Low Overhead** - Direct NAPI bindings, no IPC

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the package:

```bash
vp install @vizejs/native
```

## Usage

```ts
import { compile, compileFiles } from "@vizejs/native";

// Single file
const result = compile(source, { filename: "App.vue" });

// Multiple files (parallel)
const results = compileFiles(["src/App.vue", "src/components/Button.vue"], { threads: 4 });
```

### Template tree

`parseTemplateAst` returns the full authored tree of a template: elements, text, comments,
interpolations, attributes and directives in place, typed as `TemplateRootNode` and friends. Node
kinds and fields follow `@vue/compiler-core` (`type: "ELEMENT"`, `tagType`, `props`, `rawName`,
`exp`, `arg`, `modifiers`). Every `loc` counts UTF-16 code units with 1-based lines and columns, so
`source.slice(loc.start.offset, loc.end.offset)` is the node's text. Parser diagnostics are
returned in `errors` and `warnings` instead of being thrown. The `vize` package exports the same
function.

```ts
import { parseTemplateAst } from "@vizejs/native";

const { ast, errors, warnings } = parseTemplateAst("<Transition><!-- note --><div /></Transition>");
```

## Platform Support

| Platform | Architecture | Status |
| -------- | ------------ | ------ |
| Linux    | x64, arm64   | ✓      |
| macOS    | x64, arm64   | ✓      |
| Windows  | x64          | ✓      |

## License

MIT
