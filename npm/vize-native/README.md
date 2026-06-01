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

## Platform Support

| Platform | Architecture | Status |
| -------- | ------------ | ------ |
| Linux    | x64, arm64   | ✓      |
| macOS    | x64, arm64   | ✓      |
| Windows  | x64          | ✓      |

## License

MIT
