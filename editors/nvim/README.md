# nvim-vize

Neovim integration for the Vize language server.

```lua
require("vize").setup()
```

The plugin configures `vim.lsp.config("vize", ...)` with `cmd = { "vize", "lsp" }` and filetypes
for `vue` and `art-vue`. By default, the server reads `vue` settings from `vite.config.*`
and `vueCompilerOptions` from `tsconfig.json`, including nested projects.
Named profiles and explicit `init_options` override project features.

Profiles:

- `lint`: enables lint diagnostics; disables type checking, editor features,
  ecosystem helpers, formatting, and cross-file analysis, including project-enabled features.
- `recommended`: enables lint, typecheck, editor, and ecosystem features.
- `off`: starts Vize with every feature explicitly disabled, including project-enabled features.

Default setup still inherits project settings. Custom partial overrides change only their named features.

Custom configuration:

```lua
require("vize").setup({
  cmd = { "/path/to/vize", "lsp" },
  init_options = {
    lint = true,
    hover = true,
    references = true,
  },
})
```
