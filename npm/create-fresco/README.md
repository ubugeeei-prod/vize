# @vizejs/create-fresco

Create a Vue SFC terminal app backed by `@vizejs/fresco`:

```sh
pnpm create @vizejs/fresco my-tui
cd my-tui
pnpm install
pnpm dev
```

The generated project includes strict TypeScript, `vite-node --watch` development, a Node 22 Vite
build, a controlled Fresco app lifecycle, and a small keyboard-driven SFC. The command never
installs dependencies itself. It refuses a non-empty target by default; `--force` overwrites only
the scaffold-owned paths and does not delete unrelated files.

```text
create-fresco [directory] [options]

--package-manager, --pm <name>  npm, pnpm, yarn, or bun
--force                        overwrite scaffold-owned files
--help, -h                     show help
--version, -v                  show version
```
