# nvim-vize

Neovim integration for the Vize language server.

```lua
require("vize").setup({
  profile = "recommended",
})
```

The plugin configures `vim.lsp.config("vize", ...)` with `cmd = { "vize", "lsp" }` and filetypes
for `vue` and `art-vue`. The default profile enables diagnostics, hover, completion,
definition, references, symbols, and ecosystem helpers.

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
