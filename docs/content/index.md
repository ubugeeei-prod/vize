---
layout: entry
title: Vize
description: High-Performance Vue.js Toolchain in Rust. Compile, lint, format, type-check, and explore Vue components.
hero:
  name: Vize
  text: High-Performance Vue.js Toolchain in Rust
  tagline: "/viːz/ — A wise tool that sees through your code. Compile, lint, format, type-check, and explore Vue components — all powered by Rust. ⚠️ Not yet production-ready."
  image:
    src: /logo.svg
    alt: Vize Logo
  actions:
    - theme: brand
      text: Get Started
      link: getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: Playground
      link: https://vizejs.dev/play
features:
  - title: Start with Vite+
    details: Add the Vue compiler and native check tasks to your existing app, with one vite.config.ts.
    link: getting-started.md
  - title: Migrate existing tools
    details: Compare what changes, move supported options, and copy the exact before/after diffs.
    link: guide/migration.md
  - title: Configure a check
    details: Find where compiler, lint, formatting, and type-checking settings belong.
    link: guide/configuration.md
  - title: Understand a lint rule
    details: Look up diagnostics and compare bad and good Vue examples.
    link: rules/all.md
  - title: Explore components
    details: Browse UI examples, copy the imports, and try component behavior.
    link: guide/ui/index.md
  - title: Preview your own components
    details: Write Musea art files and open a gallery alongside your app.
    link: guide/musea.md
---

## What's in the toolchain

Every tool parses a Vue file once and shares the result. Adopt them one at a time.

| Tool              | Alternative to               | Command                       | Guide                                       |
| ----------------- | ---------------------------- | ----------------------------- | ------------------------------------------- |
| Compiler          | `@vitejs/plugin-vue`         | `vp dev`, `vp build`          | [Vite plugin](./guide/vite-plugin.md)       |
| Linter            | `eslint-plugin-vue`          | `vize lint`                   | [Rules](./rules/index.md)                   |
| Formatter         | Prettier for `.vue`          | `vize fmt`                    | [Formatting](./guide/formatter.md)          |
| Type checker      | `vue-tsc`                    | `tsc --runExternalCode`       | [Type Checking](./guide/type-checking.md)   |
| Language server   | Vue - Official               | `vize lsp`                    | [Language Server](./guide/language-server.md) |
| Vapor compiler    | —                            | `vue.compiler.vapor`          | [Vapor Mode](./guide/vapor.md)              |
| Component gallery | Storybook                    | `vize musea`                  | [Musea](./guide/musea.md)                   |
| App health report | —                            | `vize doctor`                 | [Doctor](./guide/doctor.md)                 |

## Add Vize to your app

Start with [Getting Started](./getting-started.md), then use the
[migration guide](./guide/migration.md) to replace your Vue compiler and compare
checks before changing CI. Vite+ projects keep integration settings in
`vite.config.ts`; standalone CLI and LSP workflows read the same `vue:` settings and `tsconfig.json`.

Vize is under active development. The [support status](./stability.md) describes
known limits; compare diagnostics and build output on your own project before adoption.

## Author

![ubugeeei](https://github.com/ubugeeei.png)

**[ubugeeei](https://github.com/ubugeeei)** is a software engineer based in Tokyo, working across Vue, Rust, design, and language tooling.

He is part of the [Vue.js Core Team](https://vuejs.org/about/team.html), [Vue.js Japan User Group](https://github.com/vuejs-jp) Core Staff, a [Vite+](https://github.com/voidzero-dev/vite-plus) Core Contributor, and Chief Engineer at [mates-dev](https://github.com/mates-dev).

He is also the creator of [chibivue](https://github.com/chibivue-land/chibivue), [Vize](https://github.com/ubugeeei-prod/vize), and [Ox Content](https://github.com/ubugeeei/ox-content).

- GitHub: [github.com/ubugeeei](https://github.com/ubugeeei)
- X (Twitter): [@ubugeeei](https://x.com/ubugeeei)
- Blog: [wtrclred.io](https://wtrclred.io)
- chibivue.land: [chibivue.land](https://chibivue.land)

## Sponsor

Vize is a free and open-source project licensed under MIT. Developing and maintaining a full toolchain — compiler, linter, formatter, type checker, LSP, component gallery, and WASM bindings — is a significant effort that requires sustained focus and dedication.

If Vize saves you time, improves your development experience, or you believe in the vision of a high-performance Vue.js toolchain, please consider sponsoring the project:

- CI/CD runner infrastructure is sponsored by [Blacksmith](https://www.blacksmith.sh/).
- [GitHub Sponsors](https://github.com/sponsors/ubugeeei)

Your support helps fund continued development, infrastructure costs, and ensures Vize remains free for everyone. Every contribution — no matter the size — makes a real difference.
