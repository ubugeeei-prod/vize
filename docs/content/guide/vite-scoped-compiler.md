# Scoped compiler settings

Use `entries[].compiler` when part of a project needs different compiler settings, for example a
directory whose whitespace must stay as written. Entries go in the shared configuration described in
[Configuration](./configuration.md), and [the Vite plugin's compiler options](./vite-plugin.md#compiler-options)
list the settings an entry can override.

```json
{
  "compiler": { "whitespace": "preserve" },
  "entries": [
    {
      "basePath": "src/condensed",
      "files": ["**/*.vue"],
      "ignores": ["generated/**"],
      "compiler": { "whitespace": "condense" }
    }
  ]
}
```

Entry bases resolve against the project root. Matching entries merge in declaration order over the top-level compiler settings. Later entries take precedence; explicit `vize({ ... })` plugin options take precedence over the shared configuration. File patterns and ignores support ordered `!` negations, directory scopes, braces, and hidden files.

The same scoped settings apply during precompilation, client and SSR loads, and HMR. Precompilation groups files with identical effective options, and disk cache identities include the configured scopes. Changing an entry's compiler settings cannot reuse output compiled with its previous settings.
