---
title: VS Code
---

# VS Code Integration

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Extension features may be incomplete or change without notice.

> **Important:** For day-to-day Vue editor support, keep using the official Vue language tools (`vuejs/language-tools`) for now.
> The Vize VS Code extension and `vize lsp` are still experimental and may not yet match the stability or coverage of the official toolchain.

Vize provides two VS Code extensions for an enhanced development experience. Together, they provide language intelligence for `.vue` files and syntax support for `.art.vue` story files.

## Vize — Vue Language Support

Full language support for Vue files powered by Vize's LSP (Maestro).

### Features

| Feature              | Description                                                                                     |
| -------------------- | ----------------------------------------------------------------------------------------------- |
| **IntelliSense**     | Auto-completion for component names, props, events, slots, directives, and template expressions |
| **Diagnostics**      | Real-time compilation errors and lint warnings as you type (powered by Armature + Patina)       |
| **Go to definition** | Navigate to component definitions, prop declarations, and imported values                       |
| **Hover**            | Type information, prop documentation, and component descriptions on hover                       |
| **Code actions**     | Quick fixes for lint violations (auto-fix) and common refactorings                              |
| **Formatting**       | Format `.vue` files using Glyph (template, script, and style blocks)                            |

### How It Works

The extension starts `vize lsp` as a language server process. The LSP implementation (`vize_maestro`) orchestrates all language features through the same Rust-native crates used by the CLI:

```
VS Code Editor
  ↕ Language Server Protocol (JSON-RPC)
vize lsp (vize_maestro)
  → vize_armature (parsing)
  → vize_croquis (semantic analysis)
  → vize_patina (linting)
  → vize_glyph (formatting)
  → vize_canon (type checking)
```

The LSP uses the same Rust-native crates as the CLI, but the editor experience is still catching up.
For now, treat editor diagnostics and type-aware features as experimental rather than as a strict mirror of `vize lint` and `vize check`.

### Configuration

The extension can be configured through VS Code's settings:

```json
// .vscode/settings.json
{
  "vize.lintLocale": "en",
  "vize.formatOnSave": true
}
```

## Vize Art — Story File Support

Syntax highlighting for `*.art.vue` files used by Musea.

### Features

| Feature                 | Description                                                                |
| ----------------------- | -------------------------------------------------------------------------- |
| **Syntax highlighting** | Full syntax highlighting for `<art>`, `<variant>`, and standard Vue blocks |
| **File icon**           | Custom file icon for `.art.vue` files in the explorer                      |
| **Language detection**  | Automatic language mode detection for art files                            |

### What Art Files Look Like

With the extension installed, art files get proper syntax highlighting:

```art-vue
<script setup lang="ts">
import MyButton from './MyButton.vue'
</script>

<art title="MyButton" component="./MyButton.vue"
     category="Components" status="ready" tags="button, ui">
  <variant name="Default" default>
    <MyButton>Click me</MyButton>
  </variant>
  <variant name="Outlined">
    <MyButton outlined>Click me</MyButton>
  </variant>
</art>
```

The `<art>` and `<variant>` elements are highlighted as Vue-specific syntax, while the component usage within variants gets standard Vue template highlighting.

## Installation

### From VS Code Marketplace

Search for "Vize" in the VS Code extensions marketplace to find both extensions:

- **Vize** — Language support (LSP)
- **Vize Art** — Story file syntax highlighting

If you need the most stable Vue editing experience today, prefer the official Vue language tools (`vuejs/language-tools`) and evaluate Vize's extension separately.

### Building from Source

```bash
# Clone the repository
git clone https://github.com/ubugeeei/vize.git
cd vize

# Build the LSP extension
cd npm/vscode-vize && pnpm install --ignore-workspace && pnpm build

# Build the Art extension
cd npm/vscode-art && pnpm install --ignore-workspace && pnpm build
```

### Using with Other Editors

The Vize LSP (`vize lsp`) follows the standard Language Server Protocol and can be used with any LSP-compatible editor:

- **Neovim** — via nvim-lspconfig
- **Helix** — via `languages.toml` configuration
- **Zed** — via extension or manual LSP configuration
- **Emacs** — via lsp-mode or eglot

Example Neovim configuration:

```lua
-- nvim-lspconfig
require('lspconfig').vize.setup({
  cmd = { 'vize', 'lsp' },
  filetypes = { 'vue' },
})
```
