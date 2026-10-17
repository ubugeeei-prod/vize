# helix-vize

Helix `languages.toml` integration for the Vize language server.

Copy or merge `languages.toml` into:

```text
~/.config/helix/languages.toml
```

The default configuration lets the server load project settings:

```toml
[language-server.vize]
command = "vize"
args = ["lsp"]

```

The package registers Vize for `vue` and `art-vue`. The `art-vue` language uses a glob so
`*.art.vue` files do not get swallowed by the generic `.vue` suffix rule.

Vize reads `vue` settings from `vite.config.*` and `vueCompilerOptions` from `tsconfig.json`,
including nested projects. Add `[language-server.vize.config]` only to override individual
features for Helix; for example, `formatting = false` disables server formatting.

CI validates this file with the pinned official Helix binary (`hx --health vue` and
`hx --health art-vue`), then runs the advertised language-server features against one real
`vize lsp` binary.
