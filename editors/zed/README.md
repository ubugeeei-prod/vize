# Vize for Zed

Vue diagnostics and language support powered by Vize.

This extension expects the `vize` CLI to be available on `PATH`, or configured through Zed settings.

The extension also registers an `Art Vue` language for `*.art.vue`, so Vize can power hover,
completion, go-to-definition, and references there without relying on a separate Zed extension.

By default, the extension starts Vize with the recommended profile: lint, typecheck, editor
features, and ecosystem helpers. Override `initialization_options` if you need a narrower profile.

## Recommended Profile

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    },
    "Art Vue": {
      "language_servers": ["vize"]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "editor": true,
        "ecosystem": true,
        "lint": true,
        "typecheck": true
      }
    }
  }
}
```

## Lint Only

```json
{
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": false,
        "editor": false,
        "ecosystem": false
      }
    }
  }
}
```

## Narrow Editor Profile

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    },
    "Art Vue": {
      "language_servers": ["vize"]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": true,
        "definition": true,
        "references": true,
        "hover": true,
        "ecosystem": true
      }
    }
  }
}
```

`ecosystem` enables Vue Router route-name and file-route param completions, route-param diagnostics
for `useRoute()`, Vue I18n key completions, workspace key validation and inlay previews, Void Vue
route completions, and ecosystem lint diagnostics.

`optionsApi` resolves Vue 3 Options API template bindings (`data`, `computed`, `methods`, `props`,
`inject`) during typecheck and hover. It is opt-in and officially supported on Vue 3; leave it off
(the default) for `<script setup>`-only projects to keep analysis zero cost.

To make Vize the only Vue language server, replace the existing Vue server entry in your `language_servers` list with its disabled form, such as `"!server-id"`.

If you only want Vize on `*.art.vue`, keep your existing `Vue` language servers unchanged and
configure only `Art Vue`.

## Custom Binary

```json
{
  "lsp": {
    "vize": {
      "binary": {
        "path": "/path/to/vize",
        "arguments": ["lsp"]
      }
    }
  }
}
```

## Publishing

Zed extensions are published by adding this repository as a submodule to `zed-industries/extensions` and pointing the entry at `editors/zed`.
