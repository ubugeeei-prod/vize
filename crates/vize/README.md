<p align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei/vize/main/crates/vize/logo.svg" alt="vize logo" width="120" height="120">
</p>

<h1 align="center">vize</h1>

<p align="center">
  <strong>High-performance Vue.js toolchain in Rust</strong>
</p>

---

## Name Origin

**Vize** (/viːz/) is named after Vizier + Visor + Advisor — a wise tool that sees through your code. This crate serves as the unified gateway to all Vize functionality, bringing together all the art-themed crates into a single, powerful command-line tool.

## Features

- High-performance native Vue SFC compilation
- Parallel processing with configurable thread count
- Multiple output formats (JS, JSON, stats)
- SSR mode support
- TypeScript/JSX transpilation options
- `.gitignore` aware file discovery

## Installation

```bash
cargo install vize
```

## Commands

### LSP Server

```bash
vize lsp              # stdio mode (for VS Code)
vize lsp --port 9527  # TCP mode (for debugging)
```

### Lint

```bash
vize lint src/**/*.vue
vize lint --fix src/
vize lint --profile src/  # Show parse, rule hook, Croquis, and type-aware timing
```

### Compile

```bash
vize src/**/*.vue     # Compile all .vue files
vize -o ./build       # Output to custom directory
vize --ssr            # Enable SSR mode
vize --profile        # Show parse, transform, codegen, and I/O profile
vize -j 4             # Set thread count
```

### Type Check and Format Profiles

```bash
vize check --profile src/       # Show Virtual TS, Croquis, and Corsa timing profile
vize fmt --check --profile src/ # Show formatter timing profile
```

Profile reports include hot files plus internal operation timings for compiler phases, linter rule
hooks, Croquis analysis, Virtual TS generation, and Corsa probes.

## Part of the Vize Art Collection

`vize` is the command-line interface for the Vize compiler's art-themed crate collection:

| Crate              | Art Term                | Role                         |
| ------------------ | ----------------------- | ---------------------------- |
| vize_carton        | Carton (Portfolio Case) | Shared utilities & allocator |
| vize_atelier_core  | -                       | AST & Parser (core)          |
| vize_atelier_dom   | Atelier (Workshop)      | DOM compiler                 |
| vize_atelier_vapor | Atelier (Workshop)      | Vapor compiler               |
| vize_atelier_sfc   | Atelier (Workshop)      | SFC compiler                 |
| vize_vitrine       | Vitrine (Display Case)  | Bindings (Node.js/WASM)      |
| vize_canon         | Canon (Standard)        | Type checker                 |
| vize_glyph         | Glyph (Letterform)      | Formatter                    |
| vize_patina        | Patina (Aged Surface)   | Linter                       |
| **vize**           | **-**                   | **CLI (this crate)**         |

## License

MIT License
