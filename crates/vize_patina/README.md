<p align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei/vize/main/crates/vize_patina/logo.svg" alt="vize_patina logo" width="120" height="120">
</p>

<h1 align="center">vize_patina</h1>

<p align="center">
  <strong>The Patina - Code quality checker for Vue SFCs</strong>
</p>

---

## Name Origin

**Patina** (/ˈpætɪnə/) refers to the greenish layer that forms on copper, bronze, and similar metals through oxidation over time. In the world of art and antiques, patina is highly valued - it serves as a mark of authenticity, age, and quality that cannot be easily faked.

In the art world, patina:

- **Authenticates** - Indicates genuine age and provenance
- **Enhances** - Adds character and beauty to surfaces
- **Protects** - Forms a protective layer over the base metal

Similarly, `vize_patina` provides:

- **Code linting** - Checks for common issues and anti-patterns
- **Quality assurance** - Ensures Vue SFC code meets standards
- **Best practices** - Enforces consistent coding conventions

## Features

- **Fast** - Written in Rust, runs in parallel
- **Configurable** - Enable/disable rules, set severity
- **Fixable** - Auto-fix support for many rules
- **Vue 3 focused** - Composition API, `<script setup>` support
- **Preset-based** - Default `happy-path` rules with optional `opinionated` mode and `incremental` opt-in adoption

## Usage

```rust
use vize_patina::{LintPreset, Linter};

let happy_path = Linter::new();
let opinionated = Linter::with_preset(LintPreset::Opinionated);

let result = opinionated.lint_sfc(source, "App.vue");
for diagnostic in result.diagnostics {
    println!("{}: {}", diagnostic.rule_name, diagnostic.message);
}
```

## Presets

- `happy-path` - Default preset focused on common Vue correctness, security, and accessibility checks
- `opinionated` - Enables stronger stylistic and structural rules in one shot
- `essential` - Minimal correctness-focused rule set
- `incremental` - Starts empty so a host can opt in rule-by-rule
- `nuxt` - Opinionated preset tuned for Nuxt auto-import conventions

## Rule Categories

| Category               | Description               |
| ---------------------- | ------------------------- |
| `art`                  | Art template syntax rules |
| `musea`                | Design token validation   |
| `essential`            | Prevent errors (Vue 3)    |
| `strongly-recommended` | Improve readability       |

## Part of the Vize Art Collection

`vize_patina` is part of the Vize compiler's art-themed crate collection:

| Crate              | Art Term                  | Role                         |
| ------------------ | ------------------------- | ---------------------------- |
| vize_carton        | Carton (Portfolio Case)   | Shared utilities & allocator |
| vize_atelier_core  | -                         | AST & Parser (core)          |
| vize_atelier_dom   | Atelier (Workshop)        | DOM compiler                 |
| vize_atelier_vapor | Atelier (Workshop)        | Vapor compiler               |
| vize_atelier_sfc   | Atelier (Workshop)        | SFC compiler                 |
| vize_vitrine       | Vitrine (Display Case)    | Bindings (Node.js/WASM)      |
| vize_canon         | Canon (Standard)          | Type checker                 |
| vize_glyph         | Glyph (Letterform)        | Formatter                    |
| **vize_patina**    | **Patina (Aged Surface)** | **Linter (this crate)**      |

## License

MIT License
