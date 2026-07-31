---
title: Getting Started
---

# Getting Started

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use.
> APIs and package boundaries may change without notice.

Vize (_/viːz/_) is a Rust-native Vue.js toolchain. It brings compilation, linting, formatting,
type checking, editor diagnostics, and component exploration into one workspace while keeping each
capability available through focused packages and commands.

| Need                                              | Recommended entry point     |
| ------------------------------------------------- | --------------------------- |
| Compile Vue SFCs in Vite                          | `@vizejs/vite-plugin`       |
| Compile Vue SFCs in Nuxt                          | `@vizejs/nuxt`              |
| Lint, format, and type-check from project scripts | `vize`                      |
| Combine Vize diagnostics with Oxlint              | `oxlint-plugin-vize`        |
| Explore and test components                       | `@vizejs/vite-plugin-musea` |
| Evaluate editor features                          | VS Code, Zed, or `vize lsp` |

## Set Up an Existing Project

Run the interactive initializer from the project root:

```bash
vpx vize init
```

`vpx` is included with [Vite+](https://viteplus.dev/guide/install). Install Vite+ first if the
command is not available in your shell.

`vize init` detects Vite, Vite+, or Nuxt; the package manager; TypeScript; the active lint command;
and existing Vize configuration before it writes anything. You choose which parts to configure:

- the Vite plugin or Nuxt module
- the Oxlint plugin, in the configuration file the active lint command reads
- `vize fmt` and `vize check` project scripts
- shared `vize.config.*` settings
- a VS Code extension recommendation

Preview every proposed file and dependency change without writing it:

```bash
vpx vize init --dry-run
```

For CI or another non-interactive environment, select the features explicitly:

```bash
vpx vize init --yes --lint --bundler --fmt --typecheck --editor
```

See [Project Setup](./guide/init.md) for detection rules, all flags, idempotency guarantees, and the
cases where the initializer deliberately refuses to edit a file.

## Choose a Manual Path

Prefer a manual setup when you need to preserve an established configuration or adopt one Vize
surface at a time:

- [Vite Plugin](./guide/vite-plugin.md) — native Vue SFC compilation in Vite
- [Nuxt Integration](./integrations/nuxt.md) — the supported path through Nuxt's Vite pipeline
- [Package Scripts and CLI](./guide/cli.md) — `vize build`, `fmt`, `lint`, `check`, `ready`, and the
  full Rust CLI

Vite is the recommended bundler integration. The unplugin and Rspack packages remain experimental;
their current scope is documented in [Other Bundlers](./guide/unplugin.md).

## Continue with the Focused Guides

Getting Started is intentionally an orientation page. Use the focused guides as the source of truth
for configuration and integration details:

- [Configuration](./guide/configuration.md) — `vize.config.*`, compiler options, type-checking, and
  Musea settings
- [Static Analysis](./guide/static-analysis.md) — the lint and type-checking model
- [Rule Documentation](./rules/index.md) — concrete diagnostics and examples
- [Oxlint Plugin](./guide/oxlint.md) — presets, settings, and the configuration file each command
  actually reads
- [VS Code and Other Editors](./integrations/vscode.md) — the opt-in editor profile and LSP setup
- [JSX & TSX](./guide/jsx.md) — Vue components authored outside `.vue` SFCs
- [Musea](./guide/musea.md) — component examples, documentation, tokens, a11y, and VRT

For day-to-day Vue editor support, keep using the official
[`vuejs/language-tools`](https://github.com/vuejs/language-tools) while Vize's editor integration is
experimental.
