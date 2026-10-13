# Build, CI and development commands in TypeScript

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) decision
records the first slice of retiring MoonBit repository tooling. Fourteen
MoonBit command packages that build, package and stage artifacts are now
strictly typed TypeScript that Node runs directly through its native type
stripping. Their arguments, environments, spawned commands, outputs, files and
exit codes are unchanged. The release and publish commands are a separate
slice of the same issue.

## Location

The ports live in `tools/support`, which `verify-tool-layout.rs` already
accepts for TypeScript and which groups support modules by use case:

- `tools/support/github/` holds the nine former `tools/moon/cmd/github/*`
  packages. Workflows call them as `node tools/support/github/<name>.ts`,
  like the existing GitHub helpers in `tools/support/compat/github`.
- `tools/support/tasks/` holds the five root Vite+ task commands
  (`build-vitrine-wasm`, `build-vize-wasm-package`, `dev-app`,
  `generate-expected` and `postprocess-types`). Tasks reach them through the
  new `nodeTool` helper beside `rustTool`.
- `tools/support/command-runtime.ts` replaces `tools/moon/lib` for these
  commands. The shared `tools/moon/lib`, `moon.mod`, `setup-moonbit` and
  `moonScript` stay until the release and publish slice lands.

Each file name is the kebab-case form of its former package name.
`tools/commands` stays the Rust Script surface. All ports and their tests are
in `tsconfig.tooling-migration.json`, so `check:repo` compiles them strictly.

## Preserved behavior

A differential harness ran each original MoonBit executable and its port in
identical sandboxes with recording fake commands. It compared exit codes,
stdout, stderr, every spawned command with its arguments, working directory
and complete environment, and the resulting file tree with modes. All 91
scenarios agreed. The harness found and the ports keep these details:

- Copied artifact paths are listed in shortlex order (length first), which
  is MoonBit's `String` ordering.
- Path joins normalize, and an absolute right-hand side is appended rather
  than replacing the base, as in MoonBit's `@path.join`.
- `trim` removes only tab, line feed, carriage return and space.
- Directory creation fails on an existing target. Only a missing parent
  triggers the parent walk, which splits on the last separator.
- `@process.run` with `extra_env` appended those variables after the
  inherited block. Cargo and Node read the first entry, so an inherited
  `RUSTFLAGS` in `github/build-vitrine-wasm` or `usage_target` in `dev-app`
  keeps precedence. Commands that passed a complete environment still
  override, including the WASM panic profile and the native build token.
- File copies create files with the original modes, keep the modes of
  existing files and flush data before closing, as `@fs.write_file` did.
- A signalled child reports its signal number, as on Linux.

Only messages that nothing consumes differ. Usage lines name the new
`node` invocation. Unexpected errors print their Node message on stderr
instead of a MoonBit error value on stdout, and still exit with status 1.
The library's `opendir` diagnostic for a missing artifact directory is gone;
the `No <label> .node files found after build.` refusal remains.

## Tests

The `native_build_token_wbtest.mbt` laws are now
`tests/tooling/native-build-token.test.ts` with the same three names. Native
Smoke's selected producer runs that file with `node --test`. Every existing
test that drove these commands through `runMoonScript` now runs the port, and
new tests cover the branches no test reached: argument errors, failed stages,
SDK discovery, bindgen installation, missing libraries, existing outputs and
every `git` exit path of the snapshot committer. `config-generation.test.ts`
still reproduces the checked-in generated declarations byte for byte.

The Native Smoke and Docs WASM custody tests keep authenticating their
original reviewed bytes. Each reverses this change exactly before comparing
the original digest, or records the re-reviewed producer and job digests.

## Workflows

A job drops `setup-moonbit` only when no MoonBit command remains in it:

- Docs build (`build-docs`, `build-playground`), Deploy docs, the full and
  PR playground jobs, the CI wall-time Docs job, pkg.pr.new, the npm
  bootstrap handoff and the release package composite action;
- Native Smoke's selected producer;
- the release CLI, WASM package, native build and package smoke jobs.

Node must run before any of these commands. The WASM jobs now set up Vite+
before building WASM. The release CLI and WASM package jobs, which used no
Node setup before the build, install the pinned Node from `package.json` with
`actions/setup-node`. The WASM cache keys hash the TypeScript builder and its
imports instead of the MoonBit package.

`release-npm-fresco-native` keeps `setup-moonbit`, because it still runs the
MoonBit `publish_npm_package`.

## Unfinished

The release and publish commands, `tools/moon`, `moon.mod`, the
`setup-moonbit` action and the `moonScript` helper remain until that slice
merges. Deploy docs still runs the site assembler from the completed build's
checkout. A docs build of a commit from before this change that completes
afterwards therefore fails at that step, and the next build deploys. The
French, Chinese and Portuguese source guides still describe MoonBit commands;
the English guide no longer does.
