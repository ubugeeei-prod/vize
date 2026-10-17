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

- `lint`: enables Vize lint diagnostics only.
- `recommended`: enables lint, typecheck, editor, and ecosystem features.
- `off`: starts Vize with no features enabled.

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
