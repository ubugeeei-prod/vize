## Summary

Two typecheck-qualification fixes for the Real Project Matrix. Both are maintainer-approved designs.

### 1. Per-app typecheck targets

vuetify's library and docs map `@/` to different directories. lx-music-desktop's renderer and lyric windows have their own tsconfigs. The registry gave each project one tsconfig: vuetify's root `tsconfig.json` has no `paths`, and lx-music's `jsconfig.json` has no `@root/*`. Both checkers therefore missed the same aliases (run 44107635850: vuetify 1616 TS2307, lx-music 4), and both runs were configuration-only.

`typecheckPerformance.targets: [{ id, tsconfig, corpusGlobs, baselineTsconfig?, configurationOnly? }]` turns each app into its own typecheck unit, with its own Vize run, isolation, vue-tsc baseline and qualification. The expansion is identical in TS (core-tool runner, validator, dependency-prepare isolation) and in the Rust divergence report.
- The first unit keeps the project id and artifact names. Later units are `<project>@<target>` and get their own summary entry. Every unit shares the project's single install.
- Compiler, lint, glyph and LSP coverage stay per project.
- A target can be registered `configurationOnly: { reason, issue }`. It then records that reason instead of comparing, and fails once it reaches semantic comparison, so the marker cannot go stale.
- An unmarked configuration-only unit used to abort the shard at the first one, which hid every later project's result. Now every unit reports first, and then the run still fails, in every budget mode.

Registry:
- vuetify: `lib` = packages/vuetify/tsconfig.json (src, dev); `docs` = packages/docs/tsconfig.json, registered `configurationOnly` (#13300). Its tsconfig has no project reference to the library, so under TypeScript's one-program semantics the library sources keep the docs `@/` paths. vue-tsc 6.0.3 also stops at TS1005 in 4 docs SFCs.
- lx-music-desktop: `renderer` = src/renderer/tsconfig.json, `lyric` = src/renderer-lyric/tsconfig.json.

### 2. Option notices do not block qualification

Deprecated or removed compiler options (TS5101, TS5102, TS5107, TS5108) are reported against the tsconfig, so the qualification counted them as unavailable input. They introduce no `any`. They no longer block qualification, and they stay in the compared diagnostics, so both checkers must still agree on them. Every other unavailable-input code, and any other diagnostic on a config file, still makes the run configuration-only. The TS and Rust qualifications pin the list with tests. vuetify `lib`, whose only diagnostic is its root config's removed `downlevelIteration`, now reaches semantic comparison.

## Evidence

- Local (Mac) with the matrix's own prepare step: vuetify `lib` and `docs` units run as separate Vize reports, and prepare isolates both target tsconfigs. The `@/` TS2307 in `lib` are gone.
- `node --test` on typecheck-targets (new), typechecker-configuration-qualification, fixture-tool-matrix-typecheck-target, typecheck-dependency-prepare(-steps) and vue-ecosystem-fixtures-typecheck: 34/34 pass. `rust-script --test` on the divergence report: 7/7 pass (2 new). `vize_tool_support` qualification tests: 2/2 (new). `vp check` and `tsc -p tools` / `tsc -p tests` are clean.
- A scoped Real Project Matrix run on vuetify and lx-music-desktop follows. The whole-project divergence report runs only on Linux.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
