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

- React 18
- Vite
- Monaco Editor
- Prism for syntax highlighting
- WASM (Vize)
