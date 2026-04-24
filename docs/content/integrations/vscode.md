---
title: VS Code
---

# VS Code Integration

> **⚠️ Work in Progress:** Vize's editor support is still experimental.

> **Important:** For day-to-day Vue editor support, keep using the official Vue language tools
> (`vuejs/language-tools`) for now. Vize is designed for incremental opt-in evaluation.

The repository contains two experimental VS Code extensions:

- **Vize** — Vue language support backed by `vize lsp`
- **Vize Art** — syntax highlighting for Musea `*.art.vue` files

They are not published to the VS Code Marketplace yet. Install from a locally built VSIX or run the
extension development host while the editor packages stabilize.

Install both if you want `*.art.vue` to receive Vize hover, completion, go-to-definition, and
reference support in addition to syntax highlighting.

## Vize Extension

The Vize extension starts `vize lsp` and can opt into specific capability bundles.
When you open a Vue file with the extension still disabled, or with no capabilities enabled, the extension now offers a one-click recommended workspace setup so hover, jump, and diagnostics do not silently stay off.
That setup writes `vize.enable`, `vize.lint.enable`, `vize.typecheck.enable`, and `vize.editor.enable` for the current workspace.

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
| `vize.formatting.enable`     | Enable document formatting                         |
| `vize.definition.enable`     | Enable go-to-definition                            |
| `vize.references.enable`     | Enable references                                  |
| `vize.hover.enable`          | Enable hover                                       |
| `vize.codeActions.enable`    | Enable lint quick fixes                            |
| `vize.semanticTokens.enable` | Enable semantic tokens                             |
| `vize.trace.server`          | Trace LSP communication                            |

### What the Extension Uses

```text
VS Code
  ↕ Language Server Protocol
vize lsp (vize_maestro)
  → vize_armature
  → vize_croquis
  → vize_patina
  → vize_canon
  → vize_glyph
```

### Installing from Source or VSIX

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then:

```bash
git clone https://github.com/ubugeeei/vize.git
cd vize
cd npm/vscode-vize
vp install --ignore-workspace
vp build
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Vize Art Extension

`Vize Art` provides syntax highlighting for Musea `*.art.vue` files.

It recognizes:

- `<art>` metadata blocks
- `<variant>` blocks
- standard Vue `<template>`, `<script>`, and `<style>` sections

## Other Editors

`vize lsp` follows the Language Server Protocol and can be used by editors such as Neovim, Helix,
Zed, and Emacs.

Example Neovim setup:

```lua
require("lspconfig").vize.setup({
  cmd = { "vize", "lsp" },
  filetypes = { "vue" },
  init_options = {
    lint = true,
    typecheck = false,
    editor = false,
  },
})
```

When another TypeScript server such as tsgo owns project diagnostics, keep `typecheck = false` and
turn on only the Vue-specific capabilities you want to evaluate.
