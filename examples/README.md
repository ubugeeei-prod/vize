# Vize Examples

Example projects for trying Vize locally.

## Prerequisites

Run the following from the project root before using the examples:

```bash
nix develop
vp env install
vp install
vp run --workspace-root cli  # Enable the vize CLI command
```

Or build directly with Cargo:

```bash
cargo build --release
```

---

## CLI Examples

The `examples/cli/` directory contains sample Vue files for trying the CLI tools.

### File Structure

| File                  | Description                       |
| --------------------- | --------------------------------- |
| `src/App.vue`         | A correctly formatted Vue file    |
| `src/Unformatted.vue` | A Vue file that needs formatting  |
| `src/HasErrors.vue`   | A Vue file containing lint errors |

### Formatter (`vize fmt`)

```bash
# Check whether formatting is needed
vize fmt examples/cli/src/*.vue --check

# Print the formatted result without changing files
vize fmt examples/cli/src/Unformatted.vue

# Write changes to the file
vize fmt examples/cli/src/Unformatted.vue --write

# With options
vize fmt examples/cli/src/*.vue --single-quote --no-semi --print-width 80
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
vize lint examples/cli/src/*.vue

# Output as JSON
vize lint examples/cli/src/HasErrors.vue --format json

# Set a warning limit
vize lint examples/cli/src/*.vue --max-warnings 5

# Show only the summary
vize lint examples/cli/src/*.vue --quiet
```

**Options:**

| Option           | Description                    | Default |
| ---------------- | ------------------------------ | ------- |
| `--format`, `-f` | Output format (`text`/`json`)  | text    |
| `--max-warnings` | Warning limit                  | -       |
| `--quiet`, `-q`  | Show only the summary          | false   |
| `--fix`          | Auto-fix (not implemented yet) | false   |

### LSP Server (`vize lsp`)

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

| File                            | Description                          |
| ------------------------------- | ------------------------------------ |
| `src/components/Button.vue`     | Button component                     |
| `src/components/Button.art.vue` | Musea art file (variant definitions) |
| `vite.config.ts`                | Vite + Musea configuration           |

### Writing Art Files

`.art.vue` files define component variants:

```vue
<art title="Button" component="./Button.vue" category="Components" status="ready">
  <variant name="Default" default>
    <Button>Default Button</Button>
  </variant>
  <variant name="Primary">
    <Button variant="primary">Primary Button</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from "./Button.vue";
</script>
```

**`<art>` attributes:**

| Attribute   | Description                               |
| ----------- | ----------------------------------------- |
| `title`     | Component title (required)                |
| `component` | Path to the target component              |
| `category`  | Category                                  |
| `status`    | Status (`draft` / `ready` / `deprecated`) |

**`<variant>` attributes:**

| Attribute  | Description                          |
| ---------- | ------------------------------------ |
| `name`     | Variant name (required)              |
| `default`  | Mark as the default variant          |
| `skip-vrt` | Skip VRT (Visual Regression Testing) |

---

## Troubleshooting

### `vize` Command Not Found

```bash
# Enable the CLI from vp run
vp run --workspace-root cli

# Or use cargo run directly
cargo run --release -- fmt examples/cli/src/*.vue
```

### Native Binding Errors

If you use the Musea plugin, `@vizejs/native` must be built:

```bash
vp run --workspace-root build:native
```
