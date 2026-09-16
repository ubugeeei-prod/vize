# Vize Playground

A modern WASM-powered playground for testing Vize.

## Features

- Real-time compilation
- Monaco Editor with Vue syntax highlighting
- Split-pane view (Template / Output)
- Multiple output views (Code, AST, Helpers)
- Compiler options (Mode, Hoist Static, Cache Handlers)
- VDom / Vapor mode toggle
- Beautiful dark theme

## Development

```bash
# Enter the dev shell and install dependencies (from project root)
nix develop
vp env install
vp install

# Run development server
vp run --filter './playground' dev

# Build WASM (from project root)
vp run --workspace-root build:wasm

# Build for production
vp run --filter './playground' build
```

## Tech Stack

- Vue 3 SFCs with Vapor compilation
- Vite
- Monaco Editor
- Prism for syntax highlighting
- WASM (Vize)

## Experimental Features

The source panel in Atelier, Canon, and Croquis includes an Experimental section.
All flags are off by default. Selections are remembered separately per tool;
choosing an example enables its flag and replaces that tool's source.

| Tool    | Available flags                                      |
| ------- | ---------------------------------------------------- |
| Atelier | In-tag comments, patterned templates, Self component |
| Canon   | In-tag comments, strict slot children                |
| Croquis | In-tag comments                                      |

Atelier applies the selected flags to VDOM, SSR, and Vapor outputs. Canon emits
strict slot child contracts into Virtual TS and Monaco checks them in the browser.
It does not run the CLI's project-wide Corsa checker. Reserved server-script
semantics and patterned-template exhaustiveness are not advertised as supported.
