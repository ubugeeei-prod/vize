# Scaffold a Fresco SFC App

`@vizejs/create-fresco` creates a strict Vue SFC terminal application without downloading or
executing an installer inside the generator:

```sh
pnpm create @vizejs/fresco my-tui
cd my-tui
pnpm install
pnpm dev
```

The development command runs `vite-node --watch src/main.ts`, so edits to the SFC restart the
terminal application through the Vite module graph. `pnpm check` runs `vue-tsc`; `pnpm build`
emits the Node 22 entry `dist/main.mjs`; and `pnpm start` executes that production entry.

## Executable delivery

`dist/main.mjs` is the single JavaScript launcher artifact. It can be exposed through a package
`bin` entry or a `#!/usr/bin/env node` wrapper. Fresco still loads its platform-specific native
addon from the installed `@vizejs/fresco-native` dependency, so this workflow is not a static,
dependency-free operating-system binary. Packagers that embed Node must also ship or extract the
matching native addon for each target platform.

The generator refuses filesystem roots, the home directory, files, and non-empty directories by
default. `--force` overwrites only the eight scaffold-owned files; it never removes unrelated
content.
