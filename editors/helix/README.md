# helix-vize

Helix `languages.toml` integration for the Vize language server.

Copy or merge `languages.toml` into:

```text
~/.config/helix/languages.toml
```

The default profile is recommended:

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

The package registers Vize for `vue` and `art-vue`. The `art-vue` language uses a glob so
`*.art.vue` files do not get swallowed by the generic `.vue` suffix rule.

CI validates this file with the pinned official Helix binary (`hx --health vue` and
`hx --health art-vue`), then runs the advertised language-server features against one real
`vize lsp` binary. Formatting is not advertised by this default profile; enable it explicitly
with `formatting = true` if Helix should use Vize as the document formatter.
