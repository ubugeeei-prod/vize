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

Saving a JavaScript, TypeScript, or JSON configuration/helper buffer inside the
running Vize workspace notifies the server to reload affected projects. Invalid
edits retain the last valid settings until the configuration is fixed. The pinned
`vim-lsp` client does not provide dynamic file watching: changes made outside Vim
require client watch support or restarting the language server.

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
