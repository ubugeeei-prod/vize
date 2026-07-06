# vim-vize

Vim integration for the Vize language server.

Vim does not include a built-in LSP client. This package provides filetype detection and a
`vim-lsp` server registration helper.

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'ubugeeei-prod/vize', { 'rtp': 'editors/vim' }

call vize#setup({'profile': 'recommended'})
```

The default profile is `recommended`, so Vim starts Vize with diagnostics, hover,
completion, definition, references, symbols, and ecosystem helpers enabled.

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
