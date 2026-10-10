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
- **Ecosystem Helpers** - Vue Router file-route params, Vue I18n catalogs, Nuxt, and Void Vue diagnostics and completions

## Installation

Install the extension from the VS Code Marketplace as `ubugeeei.vize`.

### From Marketplace

```bash
code --install-extension ubugeeei.vize
```

### Development

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then:

```bash
cd editors/vscode
vp install --ignore-workspace
vp build
vp exec vsce package --no-dependencies --out dist/vize.vsix
# Press F5 to launch Extension Development Host
```

## Requirements

- VS Code 1.75+
- A matching Vize language server binary. The extension auto-detects bundled, development, cached
  GitHub release, cargo, and `PATH` binaries, and downloads the matching GitHub release binary when
  no local binary matches the extension version. `vize.serverPath` is only needed for custom builds.

Do not set `vize.serverPath` to `node_modules/.bin/vize` from the npm package. That package is for
project scripts and NAPI-backed commands; the language server is the Rust `vize` executable started
with the `lsp` subcommand. Use a GitHub release binary, Nix, a local Cargo build, or the binary that
the extension auto-detects/downloads.

## Configuration

Open a Vue file in an existing Vite or TypeScript project to start Vize automatically.
`vite.config.*` or `tsconfig.json` is sufficient; no separate Vize config or editor feature switches
are required. The project defaults enable lint diagnostics, type checking, editor assistance,
ecosystem helpers, and formatting. Explicit `vize.enable: false` always keeps the extension disabled.

Configure project features through the top-level `vize` settings in your Vite config, for example:

```ts
export default {
  vize: {
    lsp: {
      hover: false,
      formatting: false,
    },
  },
};
```

Explicit VS Code feature settings override project settings. Settings left unset preserve the
project configuration and its defaults. Existing dedicated Vize configs retain their precedence
and feature behavior, including opt-in formatting. The extension also detects project config
files inside monorepo packages; open packages as separate workspace folders when they need
different language-server configurations.

For workspaces without a project config, setting only `vize.enable: true` uses the recommended
diagnostics, editor, and ecosystem profile. The setup commands apply that profile to the workspace
by writing explicit editor settings.

The status bar item opens `Vize: Show Status`, a small command hub for switching profiles, selecting the `vize` executable, restarting the server, opening settings, and showing logs. If the server cannot be found, the same flow lets you pick a local binary instead of hunting through settings.

If you want a lighter rollout, run `Vize: Enable Lint-Only Profile`, then opt into type checking or editor features after confirming it does not overlap with your existing Vue setup.

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.ecosystem.enable": false,
  "vize.formatting.enable": false
}
```

When you are ready to evaluate Vize editor assistance separately from `vuejs/language-tools`, use:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": true,
  "vize.editor.enable": true,
  "vize.ecosystem.enable": true
}
```

`vize.editor.enable` turns on completion, hover, definition, references, symbols, rename,
semantic tokens, links, folding ranges, inlay hints, and file rename handling. If you prefer
individual switches, make sure to include `vize.completion.enable`, `vize.hover.enable`, and
`vize.definition.enable` together when testing the core editor flow.

Use individual switches when you want to compare one Vize surface against an existing Vue editor
setup without changing the whole profile:

| Setting                        | Default | Use                                                                         |
| ------------------------------ | ------- | --------------------------------------------------------------------------- |
| `vize.lint.enable`             | `true`  | Vue-aware diagnostics.                                                      |
| `vize.diagnostics.enable`      | `false` | Deprecated alias for `vize.lint.enable`.                                    |
| `vize.typecheck.enable`        | `true`  | Type checking diagnostics and type-aware backend features.                  |
| `vize.editor.enable`           | `true`  | Main editor assistance bundle.                                              |
| `vize.ecosystem.enable`        | `true`  | Router, I18n, Nuxt, and Void Vue helpers.                                   |
| `vize.optionsApi.enable`       | `false` | Vue 3 Options API template bindings.                                        |
| `vize.legacyVue2.enable`       | `false` | Vue 2.7 and Nuxt 2 helpers.                                                 |
| `vize.completion.enable`       | `true`  | Completion provider; may overlap with `vuejs/language-tools`.               |
| `vize.signatureHelp.enable`    | `true`  | Signature help provider; may overlap with `vuejs/language-tools`.           |
| `vize.hover.enable`            | `true`  | Hover provider; may overlap with `vuejs/language-tools`.                    |
| `vize.definition.enable`       | `true`  | Go-to-definition provider; may overlap with `vuejs/language-tools`.         |
| `vize.references.enable`       | `true`  | Find-references provider; may overlap with `vuejs/language-tools`.          |
| `vize.documentSymbols.enable`  | `true`  | Document symbols provider.                                                  |
| `vize.workspaceSymbols.enable` | `true`  | Workspace symbols provider.                                                 |
| `vize.codeActions.enable`      | `true`  | Lint quick fixes and suppressions; requires lint diagnostics.               |
| `vize.rename.enable`           | `true`  | Rename provider; may overlap with `vuejs/language-tools`.                   |
| `vize.codeLens.enable`         | `true`  | Code lens provider.                                                         |
| `vize.formatting.enable`       | `false` | Formatting provider; keep off when another Vue formatter owns formatting.   |
| `vize.semanticTokens.enable`   | `true`  | Semantic token provider; may overlap with theme or Vue-token providers.     |
| `vize.documentLinks.enable`    | `true`  | Document link provider.                                                     |
| `vize.foldingRanges.enable`    | `true`  | Folding range provider.                                                     |
| `vize.inlayHints.enable`       | `true`  | Inlay hints provider.                                                       |
| `vize.fileRename.enable`       | `true`  | File rename edits for Vue imports.                                          |
| `vize.autoInsert.enable`       | `false` | Experimental automatic insertion for refs, interpolation, tags, and quotes. |

The table shows VS Code setting defaults. Unset settings defer to the project profile; in
Vite/TypeScript projects, formatting is enabled unless a project setting or explicit
`vize.formatting.enable: false` disables it. Choose `ubugeeei.vize` as your Vue default formatter
to use Vize for format-on-save.

`vize.ecosystem.enable` adds Vue Router route-name and file-route param completions, route-param
diagnostics for `useRoute()`, Vue I18n key completions, workspace key validation, inlay previews,
Void Vue route completions, and ecosystem lint diagnostics.

Vue 3 Options API support is opt-in. Set `vize.optionsApi.enable: true` to resolve `data`,
`computed`, `methods`, `props`, and `inject` template bindings in type checking and hover. It is
officially supported on Vue 3 and stays zero cost when left off for `<script setup>`-only projects.

Vue 2.7 / Nuxt 2 support is opt-in. Set `vize.legacyVue2.enable: true` to include Options API
template bindings and Nuxt 2 globals in type checking, completion, hover, definition, and references.

When paired with the `Vize Art` extension (`vize.vize-art`), the same editor capabilities also
apply to `*.art.vue` documents.

### Vue TSX and JSX

Opening a `.tsx` or `.jsx` file activates the extension, including in a workspace containing
no Vue SFCs. Enable Vue JSX checking in the project's `vize.config.json`:

```json
{
  "lsp": { "editor": true, "typecheck": true },
  "typeChecker": { "jsxTypecheck": true }
}
```

The same `typeChecker.jsxTypecheck` option works in Pkl configuration. It enables type
diagnostics, completion, hover, and navigation through the native TypeScript backend.
Leave it off for React projects: JSX documents are then withheld from the Vize server.
Restart the server after changing the project configuration. Older servers that do not
advertise JSX support also leave JSX documents with their existing language service.

## Commands

- `Vize: Show Status` - Open the Vize status and setup action hub
- `Vize: Enable Recommended Profile` - Enable lint, type checking, and editor assistance
- `Vize: Enable Lint-Only Profile` - Enable diagnostics while leaving editor navigation to existing tools
- `Vize: Select Language Server Executable` - Set `vize.serverPath` from a file picker
- `Vize: Disable Language Server` - Stop Vize for the current configuration target
- `Vize: Restart Language Server` - Restart the LSP server
- `Vize: Show Output Channel` - Show server logs

## License

MIT
