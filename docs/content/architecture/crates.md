---
title: Crates
---

# Crate Reference

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Crate APIs are unstable and may change without notice.

Vize consists of 16 Rust crates organized in a workspace. Each crate is independently versioned and published to [crates.io](https://crates.io/). The crate names follow an [art terminology convention](./overview.md#naming-convention) — see [Philosophy](../philosophy.md) for the full rationale.

## Core Crates

### vize_carton

**The Artist's Toolbox** — Shared utilities and arena allocator used across all crates.

- **Arena allocation** — Bump allocator (`bumpalo`) for AST nodes. Allocation is O(1), deallocation is O(1) (drop the entire arena), and memory locality is excellent (nodes packed contiguously).
- **String interning** — Common string interning via `compact_str` to reduce memory usage and enable fast string comparison.
- **Shared types** — Bitflags, hash maps (`rustc-hash`, `xxhash-rust`), small vectors (`smallvec`), and other foundational types.
- **Error utilities** — Common error formatting and source location types.

Key dependencies: `bumpalo`, `compact_str`, `bitflags`, `phf`, `rustc-hash`, `smallvec`, `xxhash-rust`

### vize_relief

**The Sculptured Surface** — AST (Abstract Syntax Tree) definitions, error types, and compiler options.

- **Template AST nodes** — Elements, expressions, directives (`v-if`, `v-for`, `v-bind`, `v-on`, `v-slot`, etc.), text, comments, and interpolation.
- **Script AST integration** — Interfaces with OXC's JavaScript/TypeScript AST for `<script>` block analysis.
- **Compiler options** — Configuration types shared across all compilation backends (DOM, Vapor, SSR).
- **Diagnostic types** — Error and warning types with source location tracking (file, line, column, span).

### vize_armature

**The Structural Framework** — Parser and tokenizer for Vue SFC files.

- **HTML-like tokenizer** — Stream-based tokenizer that handles Vue-specific syntax extensions: directive shorthand (`@`, `:`, `#`), expression delimiters (`{{ }}`), and SFC block boundaries (`<template>`, `<script>`, `<style>`).
- **Recursive descent parser** — Produces the Relief AST from the token stream. Handles self-closing tags, dynamic components, `<Suspense>`, `<Teleport>`, `<KeepAlive>`, and other Vue built-ins.
- **Expression parsing** — Parses JavaScript/TypeScript expressions within template directives and interpolations.
- **Directive parsing** — Full support for all Vue directives: `v-if`/`v-else-if`/`v-else`, `v-for`, `v-bind`, `v-on`, `v-model`, `v-show`, `v-slot`, `v-pre`, `v-once`, `v-memo`, `v-cloak`, and custom directives.
- **Error recovery** — Produces partial AST on parse errors, enabling the LSP to provide diagnostics even for incomplete code.

### vize_croquis

**The Quick Sketch** — Semantic analysis layer.

- **Template expression validation** — Verifies that template expressions are valid JavaScript/TypeScript using OXC's parser (`oxc_parser`, `oxc_ast`).
- **Scope resolution** — Tracks variable scopes through `v-for`, `v-slot`, and `<script setup>` bindings.
- **Binding detection** — Classifies bindings by source: `setup` (Composition API), `data` (Options API), `props`, `inject`, and template-local (`v-for` variable, slot props).
- **Cross-block analysis** — Uses `dashmap` for concurrent analysis across multiple SFC blocks.

Key dependencies: `oxc_parser`, `oxc_ast`, `oxc_span`, `oxc_allocator`, `dashmap`

## Compilation Crates

### vize_atelier_core

**The Core Workshop** — Shared transforms and code generation utilities used by all compilation backends.

- **AST transforms** — Static hoisting (lifting constant nodes out of the render function), caching (memoizing stable subtrees), and tree flattening.
- **Code generation primitives** — JavaScript code builder with proper indentation, string escaping, and identifier generation.
- **Source map generation** — Maps generated JavaScript back to the original `.vue` source for debugging.
- **OXC integration** — Full OXC stack (`oxc_parser`, `oxc_ast`, `oxc_codegen`, `oxc_semantic`, `oxc_transformer`) for JavaScript/TypeScript code generation and transformation.

### vize_atelier_dom

**The DOM Workshop** — Generates code targeting Vue's virtual DOM runtime.

- **`createVNode`/`h` calls** — Generates virtual DOM node creation calls with proper types and children.
- **Patch flag optimization** — Computes and emits patch flags (`PatchFlags.TEXT`, `PatchFlags.CLASS`, etc.) so Vue's runtime diff algorithm can skip unnecessary comparisons.
- **Static hoisting** — Lifts static subtrees out of the render function into module-level constants, avoiding re-creation on every render.
- **Block tree optimization** — Uses `openBlock()`/`createBlock()` for optimized VDOM patching.

### vize_atelier_vapor

**The Vapor Workshop** — Generates code targeting Vue 3.6's Vapor mode.

- **Fine-grained reactivity** — Generates code that subscribes to individual reactive sources and updates specific DOM nodes directly, without a virtual DOM diff.
- **Direct DOM manipulation** — `document.createElement`, `textContent`, `setAttribute` — no virtual DOM abstraction.
- **No runtime overhead** — Vapor components have zero VDOM overhead. Only the reactive subscriptions and their effects exist at runtime.
- **Template-level optimization** — Analyzes the template to determine the minimal set of reactive effects needed.

### vize_atelier_sfc

**The SFC Workshop** — Orchestrates the compilation of complete Single File Components.

- **`<script setup>` compilation** — Transforms `<script setup>` into a standard `export default` with `setup()` function, handling `defineProps`, `defineEmits`, `defineExpose`, `defineSlots`, `defineModel`, `defineOptions`, and `withDefaults`.
- **Template compilation coordination** — Delegates to the appropriate backend (DOM, Vapor, SSR) based on compilation options.
- **`<style>` scoped CSS** — Generates scoped CSS using data attributes and processes CSS through LightningCSS (when the `native` feature is enabled).
- **HMR support** — Generates `__hmrId` and `__file` metadata for Vite's Hot Module Replacement.
- **Multi-block orchestration** — Coordinates the compilation of all SFC blocks into a single JavaScript module.

Key dependencies: all `vize_atelier_*` crates + `lightningcss` + full OXC stack

### vize_atelier_ssr

**The SSR Workshop** — Generates code optimized for server-side rendering.

- **String concatenation** — Generates `_push` calls that build HTML strings through concatenation, avoiding DOM creation overhead on the server.
- **Hydration markers** — Emits `<!--[-->` and `<!--]-->` markers that Vue's client-side runtime uses to match server-rendered HTML with client-side components.
- **SSR-specific optimizations** — Skips event handlers, client-only directives, and other browser-specific features.

## Tool Crates

### vize_canon

**The Standard of Correctness** — TypeScript type checker for Vue components.

- **Template type inference** — Infers types for template expressions based on `<script setup>` bindings and prop definitions.
- **Props type validation** — Verifies that component usage matches declared prop types (required props, type compatibility).
- **Emits type checking** — Validates that emitted events match `defineEmits` declarations.
- **Corsa-backed native checking** — Canon is designed to integrate with Vize's Corsa stack for JavaScript/TypeScript type checking while continuing to provide Vue-specific template analysis.

Key dependencies: all core crates + OXC + `dashmap` + `tokio` (async, with `native` feature)

### vize_patina

**The Quality Finish** — Vue.js linter with internationalized messages.

- **Template lint rules** — Vue-specific rules: valid directive usage, required `key` in `v-for`, no unused `v-for` variables, etc.
- **Accessibility rules** — WAI-ARIA best practices: `alt` text for images, ARIA attribute validity, focusable element roles.
- **Best practice rules** — Code quality rules: no `v-html` (XSS risk), no duplicate attributes, no template `key` on `<template v-for>`, etc.
- **i18n diagnostics** — All lint messages available in English, Japanese (日本語), and Chinese (中文).
- **Auto-fix** — Many rules support automatic fixing.
- **Complementary with oxlint** — Focuses on Vue-specific rules, designed to work alongside oxlint for JavaScript/TypeScript rules.

Key dependencies: `vize_relief`, `vize_armature`, `vize_croquis` + full OXC stack + `lightningcss`

### vize_glyph

**The Letterform** — Vue.js code formatter.

- **Template formatting** — Indentation, attribute alignment, self-closing tag normalization, expression formatting.
- **Script formatting** — JavaScript/TypeScript formatting using OXC's code generator.
- **Style formatting** — CSS formatting within `<style>` blocks.
- **Single-pass** — Formats all three SFC blocks in a single pass, maintaining consistent output.

Key dependencies: `vize_atelier_sfc` + OXC codegen

## Integration Crates

### vize_maestro

**The Master Conductor** — Language Server Protocol implementation.

- **Completions** — Auto-complete for component names, prop names, directive arguments, event names, and slot names.
- **Diagnostics** — Real-time compilation errors and lint warnings as you type.
- **Hover** — Type information, prop documentation, and component descriptions.
- **Go to definition** — Navigate to component definitions, prop declarations, and imported values.
- **Code actions** — Quick fixes for lint violations and common refactorings.
- **Rope-based text editing** — Uses `ropey` for efficient incremental text editing, supporting large files without re-parsing the entire document.

Key dependencies: `tower-lsp`, `tokio`, `ropey`, `dashmap`, `parking_lot`

### vize_vitrine

**The Glass Display Case** — Node.js (NAPI) and WebAssembly bindings.

The binding layer that exposes Vize's Rust crates to JavaScript consumers. It has two feature flags for two targets:

- **`napi` feature** — Node.js native addon via napi-rs. Provides `compileFile`, `compileBatch` (parallel via Rayon), `lintFile`, `formatFile`, and other APIs. Used by `@vizejs/vite-plugin` and `@vizejs/native`.
- **`wasm` feature** — WebAssembly via wasm-bindgen. Provides `compileSfc`, `lintSfc`, `formatSfc` for browser usage. Used by `@vizejs/wasm`.

Both targets expose the same compilation pipeline, ensuring consistent output.

### vize

**The CLI** — Command-line interface binary crate.

- **Command orchestration** — Routes `build`, `fmt`, `lint`, `check`, `musea`, `lsp` to the appropriate crates.
- **File discovery** — Uses `glob` and `ignore` for `.vue` file discovery, respecting `.gitignore` patterns.
- **Parallel execution** — Uses Rayon for multi-threaded file processing across all commands.
- **Argument parsing** — Uses `clap` for type-safe CLI argument parsing.

### vize_musea

**The Museum** — Component gallery (Storybook alternative).

- **`*.art.vue` parsing** — Parses art file syntax (`<art>`, `<variant>`) and extracts component metadata, variants, and documentation.
- **Gallery UI** — Built-in web UI for browsing, previewing, and interacting with components.
- **Design tokens** — Reads Style Dictionary-compatible JSON token files and displays them with visual previews.
- **Component documentation** — Extracts prop definitions, slot names, and event signatures from source components.

### vize_fresco

**The Terminal Painting** — Terminal User Interface framework.

- **Layout engine** — Uses `taffy` (flexbox) for terminal layout calculation.
- **Rendering** — Uses `crossterm` for terminal rendering with Unicode support.
- **Optional NAPI bindings** — Can be used from Node.js via the `napi` feature flag.

## npm Packages

| Package                     | Source Crate          | Description                                                |
| --------------------------- | --------------------- | ---------------------------------------------------------- |
| `vize`                      | `vize`                | Main CLI package (binary)                                  |
| `@vizejs/native`            | `vize_vitrine` (napi) | Node.js NAPI bindings                                      |
| `@vizejs/wasm`              | `vize_vitrine` (wasm) | WASM bindings for browser                                  |
| `@vizejs/vite-plugin`       | —                     | Vite plugin (drop-in replacement for `@vitejs/plugin-vue`) |
| `oxlint-plugin-vize`        | —                     | Oxlint JS plugin bridge for Vize Patina diagnostics        |
| `@vizejs/nuxt`              | —                     | Nuxt module (first-class Nuxt integration)                 |
| `@vizejs/vite-plugin-musea` | —                     | Musea Vite plugin                                          |
| `@vizejs/musea-mcp-server`  | —                     | MCP server for AI assistant integration                    |

`oxlint-plugin-vize` is being prepared for an alpha npm release. When published, it will pull platform-specific optional dependencies on `@vizejs/native-*`, so `pnpm add -D oxlint oxlint-plugin-vize@alpha` will be enough for most consumers.
