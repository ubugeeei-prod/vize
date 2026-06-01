# helix-vize

Helix `languages.toml` integration for the Vize language server.

Copy or merge `languages.toml` into:

```text
~/.config/helix/languages.toml
```

The default profile is lint-only:

```toml
[language-server.vize]
command = "vize"
args = ["lsp"]

[language-server.vize.config]
lint = true
```

The package registers Vize for `vue` and `art-vue`. The `art-vue` language uses a glob so
`*.art.vue` files do not get swallowed by the generic `.vue` suffix rule.
