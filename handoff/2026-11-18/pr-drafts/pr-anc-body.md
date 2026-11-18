## Summary

These tests pin that Vize's mirror resolves packages and `types` entries through the ancestor `node_modules` chain the same way TypeScript does. The Real Project Matrix keeps fixtures under the repository, where ancestor `node_modules` and `@types` are visible to both `vize check` and vue-tsc. That leak stays symmetric only while this property holds. Full fixture isolation for the typecheck and LSP surfaces is deferred until they are green, as recorded on #13286.

- **Canon unit test** (`package_node_modules/tests/ancestor_chain.rs`). An app at `apps/web` has its own `dup` and `@types/shared-globals`. The workspace root also has `dup`, `@types/shared-globals`, `@types/far-globals` and `above-only`. The flattened mirror root must link:
  - the nearest install for each duplicate name;
  - the farther `@types/far-globals`, so a nearer partial `@types` scope must not hide it;
  - `above-only`, which is installed only above the app.

  Mutation check: walking the chain farthest-first fails the test.
- **CLI parity test** (`check_ancestor_node_modules_cli.rs`).
  - The same layout is written under the system temp dir. The test asserts that no ancestor there provides packages.
  - Each of `types: ["far-globals", "shared-globals"]`, `["*"]` and `[]` runs tsgo directly on `src/probe.ts`, and the test asserts TypeScript's exact `(line, column, code)` list.
  - It then asserts `vize check` reports that same list for `src/probe.ts` and, shifted by the `<script>` line, for `src/Probe.vue`.
  - The two `wrong*` probe lines fail only if the nearest `dup` and `@types/shared-globals` were read.
  - `[]` pins that the mirror adds no type packages TypeScript would not load.

Test-only change; no product code moves.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
