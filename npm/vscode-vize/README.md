# Vize - VS Code Extension

Vue Language Support powered by Vize - A high-performance language server for Vue SFC.

> For day-to-day Vue editor support, keep using the official Vue language tools (`vuejs/language-tools`) for now.
> This extension is still experimental and should be evaluated separately from your primary editor setup.

## Features

- **Diagnostics** - Real-time error detection
- **Completion** - Vue directives, components, Composition API
- **Hover** - Type information and documentation
- **Go to Definition** - Navigate template to script
- **Find References** - Cross-file reference search
- **Rename** - Safe identifier renaming
- **Semantic Highlighting** - Vue-specific syntax colors
- **Code Lens** - Reference counts

## Installation

### From VS Code Marketplace

Search "Vize" in VS Code Extensions.

### From VSIX

```bash
code --install-extension dist/vize.vsix
```

### Development

```bash
cd npm/vscode-vize
pnpm install --ignore-workspace
pnpm run build
# Press F5 to launch Extension Development Host
```

## Requirements

- VS Code 1.75+
- `vize` CLI installed (`cargo install vize`)

## Configuration

Vize is disabled by default. Start with lint-only mode, then opt into type checking or editor features after confirming it does not overlap with your existing Vue setup.

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

When you are ready to evaluate Vize editor assistance separately from `vuejs/language-tools`, use:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": true,
  "vize.definition.enable": true,
  "vize.references.enable": true,
  "vize.hover.enable": true
}
```

## Commands

- `Vize: Restart Language Server` - Restart the LSP server
- `Vize: Show Output Channel` - Show server logs

## License

MIT
