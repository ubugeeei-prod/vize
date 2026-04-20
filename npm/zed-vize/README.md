# Vize for Zed

Opt-in Vue diagnostics and language support powered by Vize.

This extension expects the `vize` CLI to be available on `PATH`, or configured through Zed settings.

## Enable Lint First

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## Add Type Checking

```json
{
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": true
      }
    }
  }
}
```

## Evaluate Editor Features

Use this only when you are ready to let Vize overlap with the existing Vue language server.

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": true,
        "definition": true,
        "references": true,
        "hover": true
      }
    }
  }
}
```

To make Vize the only Vue language server, replace the existing Vue server entry in your `language_servers` list with its disabled form, such as `"!server-id"`.

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

Zed extensions are published by adding this repository as a submodule to `zed-industries/extensions` and pointing the entry at `npm/zed-vize`.
