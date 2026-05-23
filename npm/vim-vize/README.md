# vim-vize

Vim integration for the Vize language server.

Vim does not include a built-in LSP client. This package provides filetype detection and a
`vim-lsp` server registration helper.

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'ubugeeei/vize', { 'rtp': 'npm/vim-vize' }

call vize#setup({'profile': 'lint'})
```

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
