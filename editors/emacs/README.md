# emacs-vize

Emacs Eglot integration for the Vize language server.

```elisp
(add-to-list 'load-path "/path/to/emacs-vize")
(require 'vize)
(vize-setup-eglot 'lint)
```

Profiles:

- `lint`: enables Vize lint diagnostics only.
- `recommended`: enables lint, typecheck, editor, and ecosystem features.
- `off`: starts Vize with no features enabled.

The package registers `vize-vue-mode`, `vize-art-vue-mode`, and common Vue major modes with
`eglot-server-programs`. Eglot starts `("vize" "lsp")` and passes `:initializationOptions` for
the selected profile.
