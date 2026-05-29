<p align="center">
  <img src="./assets/readme-screenshot.png" alt="Vize" width="600" />
</p>

<p align="center">
  <strong>Unofficial High-Performance Vue.js Toolchain in Rust</strong>
</p>

<p align="center">
  <em>/viːz/: Named after Vizier + Visor + Advisor, a wise tool that sees through your code.</em>
</p>

<p align="center">
  <a href="https://vizejs.dev"><strong>Documentation</strong></a> ・
  <a href="https://vizejs.dev/play/"><strong>Playground</strong></a> ・
  <a href="https://github.com/sponsors/ubugeeei"><strong>Sponsor</strong></a>
</p>

> [!WARNING]
> Vize is under active development. It is not a completely production-ready toolchain yet; see the
> [production-readiness checklist](./docs/release/production-readiness.md),
> [support policy](./docs/release/support-policy.md), and [stability guide](./docs/content/stability.md)
> before adopting it in production.

> [!IMPORTANT]
> For day-to-day editor support, keep using the official Vue language tools (`vuejs/language-tools`)
> for now. Vize's VS Code extension, Zed extension, and `vize lsp` remain opt-in while the editor
> surface matures.

## What Is Vize?

Vize is an unofficial Rust-native Vue toolchain. It experiments with a shared parser, semantic
analysis, compiler, lint, type-checking, formatting, and editor pipeline for Vue single-file
components.

The main entry points today are:

- `@vizejs/vite-plugin` for Rust-native Vue SFC compilation in Vite.
- `vize` on npm for package scripts such as `build`, `fmt`, `lint`, `check`, and `ready`.
- the Rust `vize` binary for the full native CLI, LSP, profiling, and project-backed checking.
- `@vizejs/vite-plugin-musea` for Musea component gallery workflows.
- `oxlint-plugin-vize` for running Vize diagnostics inside Oxlint.

## Quick Start

Need `vp` first? Install Vite+ once from the [Vite+ install guide](https://viteplus.dev/guide/install).

```bash
vp install -D @vizejs/vite-plugin
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Add the npm CLI when you want package scripts:

```bash
vp install -D vize
vp exec vize lint src
vp exec vize check src
vp exec vize fmt --check src
```

Use the native binary when you need the full CLI:

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

## Documentation Map

| Need                                    | Read                                                                                            |
| --------------------------------------- | ----------------------------------------------------------------------------------------------- |
| First setup and package choices         | [Getting Started](./docs/content/getting-started.md)                                            |
| Vite integration                        | [Vite Plugin](./docs/content/guide/vite-plugin.md)                                              |
| CLI commands                            | [CLI Reference](./docs/content/guide/cli.md)                                                    |
| Linting, type checking, and diagnostics | [Static Analysis](./docs/content/guide/static-analysis.md)                                      |
| Shared config files                     | [Configuration](./docs/content/guide/configuration.md)                                          |
| Rule catalog                            | [Rules](./docs/content/rules/index.md)                                                          |
| Musea component gallery                 | [Musea](./docs/content/guide/musea.md)                                                          |
| Editor integration                      | [VS Code Integration](./docs/content/integrations/vscode.md)                                    |
| Architecture                            | [Architecture Overview](./docs/content/architecture/overview.md)                                |
| Language-engineering workflow           | [Language Engineering Practices](./docs/content/architecture/language-engineering-practices.md) |
| Production posture                      | [Production Readiness](./docs/release/production-readiness.md)                                  |

## Local Development

The primary local setup is `Nix + vp`:

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for repository setup, pull-request expectations, and
language-facing change-class guidance.

## Community

- [Governance](./GOVERNANCE.md)
- [Support](./SUPPORT.md)
- [Contributing](./CONTRIBUTING.md)
- [Sponsor](https://github.com/sponsors/ubugeeei)

## Credits

This project draws inspiration from
[Volar.js](https://github.com/volarjs/volar.js),
[vuejs/language-tools](https://github.com/vuejs/language-tools),
[eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue),
[eslint-plugin-vuejs-accessibility](https://github.com/vue-a11y/eslint-plugin-vuejs-accessibility),
[Lightning CSS](https://github.com/parcel-bundler/lightningcss),
[Storybook](https://github.com/storybookjs/storybook), and
[OXC](https://github.com/oxc-project/oxc).

Special thanks to:

- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI/CD runner infrastructure.
- [かっこかり](https://github.com/kakkokari-gtyih) for regular debugging and monitoring around
  Misskey, including many compiler-focused bug reports.
- [ushironoko](https://github.com/ushironoko) for compiler, linter, and CLI bug reports,
  reference implementations, and reproduction repositories.
- [dannote](https://github.com/dannote) for Elixier ecosystem feedback and pull requests,
  especially for CSS-facing APIs and fixes.
- Everyone who has mentioned, shared, tested, or amplified Vize across the community.

## License

[MIT](./LICENSE)
