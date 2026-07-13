---
title: Philosophy
---

# Philosophy

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. The design principles below describe the project's vision and direction.

Vize is more than a compiler — it is a design statement about how Vue.js tooling should work.

## Why Vize Exists

The JavaScript ecosystem has long relied on JavaScript-based tooling to compile, lint, format, and type-check JavaScript code. This creates a fundamental bottleneck: the tools that process your code are subject to the same runtime limitations as the code they process — garbage collection pauses, single-threaded execution, and dynamic dispatch overhead.

Vize takes a different approach. By rewriting the entire Vue.js toolchain in Rust, we eliminate these constraints at the architecture level. The result is not an incremental improvement — it is a categorical shift in what is possible.

## Design Principles

### 1. Unified Toolchain

Traditional Vue.js development requires assembling a constellation of separate tools: a compiler (`@vue/compiler-sfc`), a linter (eslint + eslint-plugin-vue), a formatter (prettier), a type checker (vue-tsc), and a component explorer (Storybook). Each tool often repeats source discovery, parsing, caching, and invalidation behind a different configuration surface.

Vize integrates those jobs around Atlas, a typed execution substrate with stable source identity, demand-selected providers, memoized products, and selective invalidation. It deliberately does **not** force every source through one parser or one AST. SFC containers, raw Vue templates, JS/TS modules, and JSX/TSX keep their appropriate owned representations; tools share only the products their source shape and requested root require.

```
@vue/compiler-sfc  +  eslint-plugin-vue  +  prettier  +  vue-tsc  +  Storybook
                              ↓
                            vize
```

### 2. Performance as a Feature

Speed is not a nice-to-have — it is a prerequisite for developer experience. When compilation takes seconds, developers lose flow. When linting takes minutes, developers disable it. When type checking takes too long, developers skip it.

Vize is designed so that every tool runs fast enough to be used interactively:

- **Compilation**: 15,000 SFC files in 498ms (multi-threaded)
- **Formatting**: Near-instant, even on large codebases
- **Linting**: Real-time feedback through the LSP
- **Type checking**: Incremental analysis without V8 overhead

This is achieved through native Rust execution, arena allocation, multi-threading with Rayon, and Atlas planning that omits unrequested compiler work and shares common dependencies. “Zero cost” in the Atlas design refers to compiler operation, not to the runtime cost of generated JavaScript.

### 3. Drop-in Compatibility

Vize does not ask you to rewrite your code or change your workflow. The Vite plugin is a drop-in replacement for `@vitejs/plugin-vue`. Your existing Vue components, `<script setup>`, scoped styles, and HMR all work without modification.

This principle extends to the broader ecosystem. Vize's Vite plugin is compatible with Nuxt, and the LSP integrates with VS Code through standard protocols. Adopting Vize should feel like upgrading your engine, not rebuilding your car.

### 4. Art as Architecture

Every Vize crate is named after a concept from the visual arts — painting, sculpture, and museum curation. This is not mere whimsy. The naming convention encodes a philosophy: **code is a creative medium**, and the tools that shape it should reflect the craft involved.

| Crate        | Art Origin                      | Role                                             |
| ------------ | ------------------------------- | ------------------------------------------------ |
| **Carton**   | Artist's portfolio case         | Shared utilities — the toolbox                   |
| **Atlas**    | Collection of maps              | Typed source and product execution substrate     |
| **Relief**   | Sculptural surface projection   | Authored Vue-template syntax                     |
| **Armature** | Skeleton supporting a sculpture | Vue-template parser                              |
| **Module**   | Organized body of work          | Owned JavaScript/TypeScript facts and CFG        |
| **Croquis**  | Quick gestural sketch           | Derived identity, scope, binding, and reactivity |
| **Flow**     | Movement through a composition  | Control, data, and effect graph                  |
| **Rendu**    | Rendered appearance             | Frontend-neutral render intent                   |
| **Atelier**  | Artist's workshop               | Target compiler — where output is produced       |
| **Vitrine**  | Glass display case              | Bindings — exposing the work                     |
| **Canon**    | Standard of ideal proportions   | Type checker — ensuring correctness              |
| **Patina**   | Aged surface indicating quality | Linter — polishing the surface                   |
| **Glyph**    | Carved symbol or letterform     | Formatter — shaping the text                     |
| **Maestro**  | Master conductor                | LSP — orchestrating the experience               |
| **Musea**    | Plural of museum                | Component gallery — exhibiting the work          |
| **Fresco**   | Wall painting technique         | TUI framework — painting the terminal            |

This naming system serves a practical purpose: it makes the crate hierarchy intuitive. When you see `vize_atelier_dom`, you immediately understand it is a _workshop_ that produces _VDOM output_. When you see `vize_patina`, you know it _polishes_ your code.

#### The Sculpture Analogy

The deepest analogy is between software compilation and sculpture, but it is a vocabulary of peer
responsibilities rather than a fixed pipeline:

1. **Armature and Relief** — Armature parses Vue-template text; Relief preserves what was authored,
   including tags, directives, comments, and exact locations. SFC decomposition and JS/TS parsing
   have their own owners.
2. **Module and Croquis** — Module records source-faithful JS/TS facts and CFG, while Croquis records
   derived Vue meaning such as identity, scopes, bindings, usage, and reactivity. Neither replaces
   the other.
3. **Atlas** — The atlas tells each recipe which typed products are reachable, executes shared work
   once, and invalidates affected products. It does not own a universal compiler representation.
4. **Rendu and Atelier** — Rendu expresses frontend-neutral render intent. The DOM, Vapor, and SSR
   ateliers are separate workshops that consume it when their output is requested.
5. **Vitrine** — Bindings place selected compile, lint, typecheck, and analysis products behind NAPI
   or WASM surfaces for JavaScript consumers.
6. **Musea** — Component works are exhibited, explored, and documented through the Musea tooling.

#### The Quality Crafts Analogy

The remaining crates follow a craftsmanship analogy:

- **Canon** (type checker) — In classical sculpture, the _canon_ was a standard of ideal human proportions. Polykleitos wrote the _Kanon_ defining mathematical ratios for the perfect figure. In Vize, the type checker enforces the "ideal proportions" of your code — types must be correct, props must match, emissions must conform.

- **Patina** (linter) — A _patina_ is the surface finish that develops on aged materials, indicating quality and care. A bronze sculpture with a rich patina has been well-maintained. In Vize, the linter examines the surface of your code, identifying problems that affect its quality.

- **Glyph** (formatter) — A _glyph_ is a carved symbol or letterform — think of the precise, consistent letterforms in a font. Each glyph has exact proportions and spacing. In Vize, the formatter ensures your code has consistent, precise proportions.

- **Maestro** (LSP) — A _maestro_ is the master conductor who orchestrates an ensemble into a unified performance. In Vize, the LSP server orchestrates all language features (completion, diagnostics, formatting, navigation) into a unified editor experience.

- **Fresco** (TUI) — A _fresco_ is a painting technique where pigment is applied to wet plaster, becoming part of the wall itself. In Vize, the TUI framework "paints" interfaces directly onto the terminal surface.

### 5. Vapor-First Thinking

Vue 3.6 introduces Vapor mode — a compilation strategy that generates fine-grained reactive code without the virtual DOM. Vize was designed with Vapor mode as a first-class compilation target from day one.

While `@vue/compiler-sfc` added Vapor support incrementally, Vize's `vize_atelier_vapor` was built alongside `vize_atelier_dom` from the beginning. In the production graph, DOM, Vapor, and SSR are peer providers over Rendu; `vize_atelier_core` remains a narrow legacy-compatible transform/emission helper, not the shared architecture or representation owner.

### 6. Developer Sovereignty

Vize is an **independent** toolchain. It is not controlled by the Vue.js core team, and it makes no claim to be the "official" way to build Vue applications. This is intentional.

By remaining independent, Vize can:

- Experiment with compilation strategies without the burden of backwards compatibility
- Move faster than an official project bound by governance processes
- Serve as a proving ground for ideas that may eventually influence the official toolchain
- Provide an alternative for developers who want maximum performance

At the same time, Vize tracks the official Vue.js specification closely. The goal is compatibility, not fragmentation.

### 7. Standing on the Shoulders of Oxidation

Vize does not exist in isolation. It is part of a broader movement to rewrite JavaScript tooling in systems languages — what the community calls "oxidation." Vize embraces and integrates with this ecosystem:

- **OXC** — Vize uses the [Oxidation Compiler](https://oxc.rs/) for JavaScript and TypeScript parsing. `vize_module` owns parser-lifetime-free module and CFG facts, while the SFC and JSX frontends project their Croquis and compiler facts from the same live OXC program before its allocator is dropped. Rather than reimplement a JS parser, Vize delegates to OXC's battle-tested implementation without making OXC's arena AST the universal graph product.
- **oxlint** — Vize is designed with [oxlint](https://oxc.rs/docs/guide/usage/linter) in mind. While `vize_patina` handles Vue-specific template linting, the broader JavaScript linting story is best served by oxlint's Rust-native rule engine. The two tools are complementary, not competing.
- **Corsa** — Vize's native TypeScript execution layer, built around [`corsa-bind`](https://github.com/ubugeeei/corsa-bind), represents the direction Vize is taking for JavaScript/TypeScript type checking without routing everything through a JavaScript-hosted compiler. `vize_canon` uses this stack for native diagnostics while continuing to provide Vue-specific template type analysis.
- **LightningCSS** — Vize uses [LightningCSS](https://lightningcss.dev/) for CSS parsing and transformation within `vize_atelier_sfc`, leveraging its Rust-native CSS processing for scoped styles.

There are still many unsolved challenges in this space — cross-tool AST interop, incremental analysis across language boundaries, and editor integration consistency. Vize aims to be a proving ground for solutions to these problems within the Vue.js ecosystem, contributing to the broader oxidation movement.

### 8. Collaboration with Vite+ and OXC

[Vite+](https://viteplus.dev/) and [OXC](https://oxc.rs) are **framework-agnostic** toolchains — they provide general-purpose JS/TS/CSS bundling, parsing, linting, and formatting capabilities that work across any framework. Vize is **Vue-specific** and is designed to **integrate with** these ecosystem tools rather than compete against them.

Vize directly depends on OXC for JavaScript/TypeScript parsing and LightningCSS for CSS processing within Vue SFCs. The Vize linter (Patina) and formatter (Glyph) handle Vue-specific concerns such as template directives, SFC structure, and component conventions. Production analysis recipes already reuse owned OXC-derived Module facts where applicable while keeping Vue-template syntax and semantics in their owning crates. Vize's Vite plugin (`@vizejs/vite-plugin`) is built on top of Vite and designed to be a drop-in replacement for `@vitejs/plugin-vue`, fully embracing the Vite ecosystem.

As the author of Vize, I ([@ubugeeei](https://github.com/ubugeeei)) want to be clear: **I have no adversarial intent toward any of these projects.** I am fully open to collaboration and believe that the best outcomes come from tools that complement each other. If there are changes needed on either side to enable better integration, I am ready to work together to make that happen.

## The Name

**Vize** (_/viːz/_) is derived from three words:

- **Vizier** — a wise counselor or advisor
- **Visor** — something that helps you see clearly
- **Advisor** — a guide that helps you make better decisions

Together, they describe a tool that _sees through your code_ and _advises you wisely_. The pronunciation rhymes with "breeze" — fast, effortless, and refreshing.
