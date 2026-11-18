The lint divergence report now runs both providers on a mirror of each fixture. Nothing above the mirror provides packages, so imports resolve only from the fixture's own install.

**The leak.** An in-place fixture under `tests/_fixtures/_git/<id>` resolved imports by walking up to the repository's own `tests/node_modules`. That directory holds Vize's test dependencies: reka-ui, element-plus, naive-ui, primevue, vant, vue, vue-router, `@types/*` and others. Patina's type resolution therefore read package versions the fixture never installed. ESLint does not resolve modules, so only one side of the comparison moved.

Measured locally with `lint-divergence-report.ts --all-mapped-rules`: shadcn-vue's `script/require-default-prop` false positives drop from 2009 (schedule run 43880794806) to 124 once reka-ui's contracts no longer come from `tests/node_modules`.

**The fixture's own dependencies stay.** The matrix installs pinned dependencies for 15 of its 150 projects through `typecheck-dependency-prepare`:
- `typecheckPerformance`: vue-vben-admin, hoppscotch, element-plus, reka-ui, primevue, primevue-volt, primevue-showcase, vuetify, voicevox, elk, misskey, lx-music-desktop, vuestic-admin
- `typecheckSetup`: filebrowser (`frontend/`), mealie

Users have these packages, so Vize resolving them stays correct. The mirror links every installed `node_modules` directory, at any depth, to the fixture's own as a single symlink. Packages therefore resolve exactly as that install provides them, including the shadow links `typecheck-dependency-prepare` adds. A package's undeclared transitive dependency could still walk up from its real path, but pnpm and hoisted layouts provide declared dependencies locally. The other 135 projects have no install, so imported packages now stay unresolved, as they would in a fresh clone.

**The mirror**
- It lives under `os.tmpdir()/vize-lint-divergence`, and the run refuses to start if that directory has a `node_modules` ancestor. A mirror under `target/` would still see the repository root's `node_modules/@types`.
- Files are hard links, falling back to copies across devices. Symbolic links keep their authored targets. `.git` is skipped.
- The mirror is removed after each project.
- Absolute paths in the artifact, such as ESLint `filePath` and staged raw results, are moved back to the fixture path. Reports stay deterministic and keep naming the fixture.
- The report schema is unchanged: no field is added, so strict report parsing still accepts the artifact.

New test (`tests/tooling/lint-divergence-isolation.test.ts`) covers:
- a fixture inside a fake repository whose `tests/node_modules` has reka-ui: the mirror has no package ancestor, keeps the fixture's `node_modules` as a link to its own, preserves symlinks, and leaves the fixture untouched;
- path relocation;
- refusal of a mirror root inside a package tree.

The existing lint-divergence, raw-error and reference-coverage tooling tests pass unchanged (130 tests).

The frozen-reporter parity contract (`tests/tooling/support/lint-report-parity.ts`) compares the TypeScript successor with the two pinned Rust reporters. That comparison includes the subprocess transcript, and the Rust reporters lint in place. For the successor only, `runReporter` now treats a `vize lint` working directory that is a removed `<tmpdir>/vize-lint-divergence/<id>-XXXXXX` mirror as the fixture directory. Every other transcript field, every output file and every report must still match exactly.

The typecheck harness will get the same mirror in a follow-up. Its leak is symmetric, since vize check and vue-tsc both see the repository's `@types`.

Part of #12336.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
