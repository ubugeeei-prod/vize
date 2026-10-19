---
title: Language Server
description: Use vize lsp for Vue diagnostics, completion, navigation, and formatting in VS Code, Zed, Neovim, Vim, Helix, and Emacs.
---

# Language Server

`vize lsp` is a [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
server for Vue. It brings Vize's lint rules, type checker, and formatter into your editor, together
with completion and navigation for `.vue` and Musea `.art.vue` files.

> [!WARNING]
> Editor support is experimental. Try it in one workspace first. If you rely on Vue - Official
> (`vuejs/language-tools`) today, start with the [lint-only profile](#profiles) so the two do not
> report the same problems twice.

## Features

| Area        | What you get                                                                                   |
| ----------- | ---------------------------------------------------------------------------------------------- |
| Diagnostics | [Lint rules](../rules/index.md) and [type errors](./type-checking.md) as you type, with quick fixes |
| Completion  | Components, props, events, directives, and script bindings in templates                        |
| Navigation  | Hover, go to definition, find references, document and workspace symbols                       |
| Editing     | Rename, file rename updates, signature help, inlay hints, auto-insert, folding                 |
| Formatting  | [Vize formatter](./formatter.md) for `.vue` files                                              |
| Ecosystem   | Vue Router route names and params, Vue I18n keys, Nuxt, and Void Vue helpers                   |

## Profiles

Every editor integration starts the same server with one of three profiles. Choose by who owns Vue
language features in your editor:

| Profile       | Turns on                                  | Use it when                                              |
| ------------- | ----------------------------------------- | -------------------------------------------------------- |
| `recommended` | lint, type checking, editor features, ecosystem | Vize is your only Vue language server              |
| `lint`        | lint diagnostics only                     | You keep Vue - Official for types, completion, and navigation |
| `off`         | nothing                                   | You want to switch Vize off without uninstalling it      |

You can also enable single features, for example only `hover` and `references`. Each editor
section below shows where those switches go.

## Get the `vize` binary

The VS Code extension finds or downloads the server for you. Other editors run `vize lsp` from
your `PATH`. Install the CLI globally, or point the editor at a binary from
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases):

```bash
npm install -g vize
vize --version
```

The server reads the same project settings as the CLI: `vite.config.*` and `tsconfig.json`. See
[Configuration](./configuration.md).

## VS Code

Install the extensions from the Marketplace:

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

Open a `.vue` file and accept the setup prompt, or run **Vize: Show Status** to pick a profile.
In a Vite+ project, `vp run editor:setup` writes the workspace settings for you; see
[Vite+ editor setup](./vite-plus-editor.md). Every setting and command is listed in
[VS Code](../integrations/vscode.md).

## Zed

Install the Vize extension for Zed. From a clone of the Vize repository, `vize ide zed --install`
copies `editors/zed` into Zed for you. Then select the server in your Zed `settings.json`:

```json
{
  "languages": {
    "Vue.js": { "language_servers": ["vize", "..."] },
    "Art Vue": { "language_servers": ["vize"] }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": true,
        "editor": true,
        "ecosystem": true
      }
    }
  }
}
```

## Neovim

Neovim 0.11 or newer. Add `editors/nvim` from the Vize repository with your plugin manager, then:

```lua
require("vize").setup({ profile = "recommended" })
```

The plugin registers the server through `vim.lsp.config("vize", ...)`. Pass `init_options` to pick
single features:

```lua
require("vize").setup({
  init_options = { lint = true, hover = true, references = true },
})
```

## Vim

Vim needs [vim-lsp](https://github.com/prabirshrestha/vim-lsp):

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'ubugeeei-prod/vize', { 'rtp': 'editors/vim' }

call vize#setup({'profile': 'recommended'})
```

## Helix

Merge this into `~/.config/helix/languages.toml`. The complete file, including the `art-vue`
language, is `editors/helix/languages.toml` in the repository.

```toml
[language-server.vize]
command = "vize"
args = ["lsp"]

[language-server.vize.config]
editor = true
ecosystem = true
lint = true
typecheck = true
```

## Emacs

Vize works with Eglot:

```elisp
(add-to-list 'load-path "/path/to/vize/editors/emacs")
(require 'vize)
(vize-setup-eglot 'recommended)
```

## Other editors

Any LSP client can run the server over stdio. Use TCP when the client cannot spawn a process:

```bash
vize lsp
vize lsp --port 9527
vize lsp --debug
```

Send the profile switches (`lint`, `typecheck`, `editor`, `ecosystem`, or single features such as
`hover`) as `initializationOptions`.

## Troubleshooting

| Symptom                                    | Check                                                                          |
| ------------------------------------------ | ------------------------------------------------------------------------------ |
| No diagnostics at all                      | The profile is `off`, or no feature is enabled. In VS Code run **Vize: Show Status** |
| The same type error appears twice          | Vue - Official also checks types. Use the `lint` profile or disable one of them |
| Globals such as `$t` are reported missing  | The declaration file is outside `tsconfig.json` `include`; see [Type checking](./type-checking.md#global-types-and-auto-imports) |
| The server does not start                  | `vize --version` fails in the editor's environment. Set the full binary path   |

Start the server with `--debug`, or use **Vize: Show Output Channel** in VS Code, and include the
log when you [report a problem](./testing.md).
