# Vize Examples

Example projects for trying Vize locally.

## Prerequisites

Run the following from the project root before using the examples:

```bash
nix develop
vp env install
vp install
vp run --workspace-root build:native
vp run --filter './npm/vize' build
```

Or build directly with Cargo:

```bash
cargo build --release
```

---

## npm Package Command Examples

The `examples/cli/` directory contains sample Vue files for trying the `vize` npm package commands.
In application projects, prefer package scripts. In this workspace, use `vp exec vize ...` after
building the local package.

### File Structure

| File                  | Description                                         |
| --------------------- | --------------------------------------------------- |
| `src/App.vue`         | A correctly formatted Vue file                      |
| `src/Unformatted.vue` | A Vue file that needs formatting                    |
| `src/HasErrors.vue`   | A Vue file containing lint and security diagnostics |

### Formatter (`vize fmt`)

```bash
# Check whether formatting is needed
vp exec vize fmt examples/cli/src/*.vue --check

# Print the formatted result without changing files
vp exec vize fmt examples/cli/src/Unformatted.vue

# Write changes to the file
vp exec vize fmt examples/cli/src/Unformatted.vue --write

# With options
vp exec vize fmt examples/cli/src/*.vue --single-quote --no-semi --print-width 80
```

**Options:**

| Option           | Description                              | Default |
| ---------------- | ---------------------------------------- | ------- |
| `--check`        | Exit with an error if changes are needed | -       |
| `--write`, `-w`  | Write changes to the file                | -       |
| `--single-quote` | Use single quotes                        | false   |
| `--no-semi`      | Omit semicolons                          | false   |
| `--print-width`  | Maximum line length                      | 100     |
| `--tab-width`    | Indent width                             | 2       |
| `--use-tabs`     | Use tabs                                 | false   |

### Linter (`vize lint`)

```bash
# Show lint errors
vp exec vize lint examples/cli/src/*.vue

# Output as JSON
vp exec vize lint examples/cli/src/HasErrors.vue --format json

# Output as plain text for hooks and agents
vp exec vize lint examples/cli/src/HasErrors.vue --format plain

# Set a warning limit
vp exec vize lint examples/cli/src/*.vue --max-warnings 5

# Show only the summary
vp exec vize lint examples/cli/src/*.vue --quiet

# Print rule and file timing
vp exec vize lint examples/cli/src/*.vue --profile
```

`src/HasErrors.vue` intentionally includes missing `v-for` keys, a `v-if`/`v-for` conflict, a
static unsafe URL, and an obfuscated invalid anchor so the linter output demonstrates correctness,
accessibility, and security diagnostics together.
The SSR rule docs include extra boundary examples for `typeof window`, comments, and regex literals.

**Options:**

| Option           | Description                                                                      | Default |
| ---------------- | -------------------------------------------------------------------------------- | ------- |
| `--format`, `-f` | Output format (`text`/`ansi`/`plain`/`json`/`stylish`/`markdown`/`html`/`agent`) | text    |
| `--max-warnings` | Warning limit                                                                    | -       |
| `--quiet`, `-q`  | Show only the summary                                                            | false   |
| `--fix`          | Auto-fix (not implemented yet)                                                   | false   |

### Rust LSP Server (`vize lsp`)

The npm package does not provide the LSP server. Use the Rust binary when you need editor protocol
experiments from the command line.

```bash
# Start with stdio (for editor integration)
vize lsp

# Specify a TCP port
vize lsp --port 3000

# Enable debug logging
vize lsp --debug
```

**Editor configuration example (VS Code):**

`.vscode/settings.json`:

```json
{
  "vize.lsp.path": "/path/to/vize",
  "vize.lsp.args": ["lsp", "--debug"]
}
```

---

## JSX/TSX Example

`examples/jsx-tsx/` contains focused JSX/TSX source examples for the compiler, linter, type
checker, LSP, and formatter.

### Setup

```bash
vp run --filter './examples/jsx-tsx' check
vp run --filter './examples/jsx-tsx' lint
vp run --filter './examples/jsx-tsx' fmt
```

The workspace quality gate also includes this package, so `vp run check` covers these JSX/TSX
inputs alongside the other checked examples.

### File Structure

| File                           | Description                                               |
| ------------------------------ | --------------------------------------------------------- |
| `src/StatefulPanel.tsx`        | Destructured props, state, emits, slots, lists, and style |
| `src/AccessibleMedia.jsx`      | Accessible JSX lint sample with keyed list rendering      |
| `src/FormattedScriptBlock.vue` | `.vue` `<script setup lang="tsx">` formatter sample       |

---

## Vite + Musea Example

The `examples/vite-musea/` directory contains a sample component gallery built with Vite + Musea.

### Setup

```bash
cd examples/vite-musea
vp install
vp dev
```

### Usage

1. Start the development server with `vp dev`
2. Open `http://localhost:5173` in your browser
3. View the component gallery at `http://localhost:5173/__musea__`

### File Structure

| File                        | Description                                     |
| --------------------------- | ----------------------------------------------- |
| `src/components/Button.vue` | Button component with co-located Musea variants |
| `src/tokens.json`           | Design tokens shown in the Musea gallery        |
| `vite.config.ts`            | Vite + Musea configuration                      |

### Writing Art Files

Use `defineArt(source, options)` in root `<script setup>` to declare the target component and gallery metadata. The `<art>` block then focuses on variants:

```vue
<script setup lang="ts">
defineArt("./Button.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "form"],
  status: "ready",
});
</script>

<art>
  <variant name="Default" default>
    <Button>Default Button</Button>
  </variant>
  <variant name="Primary">
    <Button variant="primary">Primary Button</Button>
  </variant>
</art>
```

**`defineArt()` options:**

| Option        | Description                               |
| ------------- | ----------------------------------------- |
| `title`       | Component title                           |
| `description` | Component summary                         |
| `category`    | Category                                  |
| `tags`        | Search tags                               |
| `status`      | Status (`draft` / `ready` / `deprecated`) |

Legacy `<art title="..." component="...">` metadata attributes are still supported for compatibility, but new examples should prefer `defineArt()`.

**`<variant>` attributes:**

| Attribute  | Description                          |
| ---------- | ------------------------------------ |
| `name`     | Variant name (required)              |
| `default`  | Mark as the default variant          |
| `skip-vrt` | Skip VRT (Visual Regression Testing) |

---

## Oxlint + Vize Example

`examples/oxlint-vize/` contains the smallest runnable setup for executing Patina from Oxlint through `oxlint-plugin-vize`.

### Setup

Run this from the repository root:

```bash
vp install
vp run --filter './npm/vize-native' build
vp run --filter './npm/oxlint-plugin-vize' build
```

### Run

```bash
vp run --filter './examples/oxlint-vize' lint
```

This command intentionally exits non-zero because it includes `src/HasPatinaErrors.vue`. It mixes Oxlint core output with Patina output and uses the `stylish` formatter so the default code frame does not dominate the output. If you only want the success path:

```bash
vp run --filter './examples/oxlint-vize' lint:clean
```

If you want JSON output:

```bash
vp run --filter './examples/oxlint-vize' lint:json
```

To turn the long Patina `Help:` block back on:

```bash
vp run --filter './examples/oxlint-vize' lint:with-help
```

To probe `no-unused-vars` on a Vue SFC:

```bash
vp run --filter './examples/oxlint-vize' lint:unused-vars-probe
```

Current observed behavior in this repository: that probe reports `0` findings on `.vue`, even though the sample file contains an unused binding.

### Files

| File                         | Description                                                |
| ---------------------------- | ---------------------------------------------------------- |
| `.oxlintrc.json`             | Oxlint config enabling `vue` and `oxlint-plugin-vize`      |
| `.oxlintrc.unused-vars.json` | Dedicated probe config for `no-unused-vars` on a Vue SFC   |
| `src/HasPatinaErrors.vue`    | Sample SFC that intentionally triggers Patina diagnostics  |
| `src/Clean.vue`              | Clean success-case sample                                  |
| `src/UnusedVarProbe.vue`     | Probe file for current `no-unused-vars` behavior on `.vue` |
| `README.md`                  | Run instructions and current limitations                   |

---

## Troubleshooting

### `vp exec vize` Command Not Found

```bash
# Build the local native binding and npm package
vp run --workspace-root build:native
vp run --filter './npm/vize' build

# Or use the Rust CLI directly for CLI-only debugging
cargo run --release -- fmt examples/cli/src/*.vue
```

### Native Binding Errors

If you use the Musea plugin, `@vizejs/native` must be built:

```bash
vp run --workspace-root build:native
```
