---
title: VS Code
---

# VS Code Integration

> [!WARNING]
> Vize's editor support is experimental. If you use Vue - Official (`vuejs/language-tools`), start
> with the [lint-only profile](#recommended-starting-point) so both extensions can run side by side,
> or disable Vue - Official in the workspace before enabling Vize's full profile.

The repository contains two experimental VS Code extensions:

- **Vize** — Vue language support backed by `vize lsp`
- **Vize Art** — syntax highlighting for Musea `*.art.vue` files

Install them from the VS Code Marketplace:

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

Install both if you want `*.art.vue` to receive Vize hover, completion, go-to-definition, and
reference support in addition to syntax highlighting.

## Vize Extension

The Vize extension starts `vize lsp` and can opt into specific capability bundles.
When you open a Vue file with the extension still disabled, or with no capabilities enabled, the extension now offers a one-click recommended workspace setup so hover, jump, and diagnostics do not silently stay off.
That setup writes `vize.enable`, `vize.lint.enable`, `vize.typecheck.enable`, and `vize.editor.enable` for the current workspace.
If you manually set only `vize.enable: true`, Vize also uses that recommended diagnostics and
editor profile instead of starting an empty language server.
The Vize status bar item opens `Vize: Show Status`, which gives you the profile switcher, server
binary picker, restart action, settings, and logs from one place.

### Recommended Starting Point

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

This enables lint diagnostics first while leaving navigation, completion, and formatting to your
existing Vue tooling.

### Common Settings

| Setting                      | Purpose                                            |
| ---------------------------- | -------------------------------------------------- |
| `vize.enable`                | Enable the extension and language server           |
| `vize.serverPath`            | Override the `vize` executable path                |
| `vize.lint.enable`           | Enable lint diagnostics                            |
| `vize.typecheck.enable`      | Enable type-aware diagnostics and backend features |
| `vize.editor.enable`         | Enable the editor assistance bundle                |
| `vize.completion.enable`     | Enable completion                                  |
| `vize.formatting.enable`     | Enable document formatting                         |
| `vize.definition.enable`     | Enable go-to-definition                            |
| `vize.references.enable`     | Enable references                                  |
| `vize.hover.enable`          | Enable hover                                       |
| `vize.codeActions.enable`    | Enable lint quick fixes                            |
| `vize.semanticTokens.enable` | Enable semantic tokens                             |
| `vize.trace.server`          | Trace LSP communication                            |

### Useful Commands

| Command                                   | Purpose                                             |
| ----------------------------------------- | --------------------------------------------------- |
| `Vize: Show Status`                       | Open the status and setup action hub                |
| `Vize: Enable Recommended Profile`        | Enable lint, type checking, and editor assistance   |
| `Vize: Enable Lint-Only Profile`          | Enable diagnostics while keeping other tools in use |
| `Vize: Select Language Server Executable` | Set `vize.serverPath` from a file picker            |
| `Vize: Disable Language Server`           | Stop Vize for the current configuration target      |
| `Vize: Restart Language Server`           | Restart the language server                         |
| `Vize: Show Output Channel`               | Show extension and LSP logs                         |

### What the Extension Uses

```text
VS Code
  ↕ Language Server Protocol
vize lsp
  → parser and semantic analysis
  → lint rules
  → type checker
  → formatter
```

### Installing from Source or VSIX

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then:

```bash
git clone https://github.com/ubugeeei-prod/vize.git
cd vize
cd editors/vscode
vp install -- --ignore-workspace
vp pack
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Vize Art Extension

`Vize Art` provides syntax highlighting for Musea `*.art.vue` files.
Its Marketplace extension ID is `vize.vize-art`.

It recognizes:

- `<art>` metadata blocks
- `<variant>` blocks
- standard Vue `<template>`, `<script>`, and `<style>` sections

## Other Editors

> [!TIP]
> The [Language Server guide](../guide/language-server.md) has setup steps for Zed, Neovim, Vim, Helix, and Emacs.

`vize lsp` follows the Language Server Protocol and can be used by editors such as Neovim, Helix,
Zed, and Emacs.

Example Neovim setup with the bundled plugin:

```lua
-- editors/nvim from the Vize repository, Neovim 0.11+
require("vize").setup({
  init_options = {
    lint = true,
    typecheck = true,
    editor = true,
  },
})
```

`editor = true` is the easiest way to test hover, completion, jump, references, and symbols
together. When another TypeScript server such as tsgo owns project diagnostics, keep
`typecheck = false` and turn on only the Vue-specific capabilities you want to evaluate.
