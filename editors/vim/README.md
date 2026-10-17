# vim-vize

Vim integration for the Vize language server.

Vim does not include a built-in LSP client. This package provides filetype detection and a
`vim-lsp` server registration helper.

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'ubugeeei-prod/vize', { 'rtp': 'editors/vim' }

call vize#setup()
```

By default, the server reads `vue` settings from `vite.config.*` and `vueCompilerOptions`
from `tsconfig.json`, including nested projects. Named profiles and explicit
`initialization_options` override project features.

Profiles:

- `lint`: enables Vize lint diagnostics only.
- `recommended`: enables lint, typecheck, editor, and ecosystem features.
- `off`: starts Vize with no features enabled.

Custom command:

```vim
call vize#setup({
      \ 'cmd': ['/path/to/vize', 'lsp'],
      \ 'initialization_options': {'lint': v:true, 'references': v:true},
      \ })
```
