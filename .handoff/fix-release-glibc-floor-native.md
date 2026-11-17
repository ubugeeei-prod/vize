# Handoff: Linux glibc floor (Vercel `GLIBC_2.35 not found`) and native `vize` startup

Agent: glibc-floor. Session ended mid-task because of a machine switch. Local scratch is gone; this file is the record.

## Branches

| Branch | Head | PR | State |
| --- | --- | --- | --- |
| `fix/release-glibc-floor` | `c426fb74ec` | #13242 | Open; all 124 checks were green; the coordinator **dequeued** it because of perf rules. It is the old full scope (bindings and CLI through zig at 2.17). Superseded by the branch below; close it or repoint it. |
| `bench/glibc-floor-recipes` | `8a1ace6a70` | none | #13242 plus the callgrind bench commit. Bench run **44077947551** was dispatched from it and **cancelled while queued** (it never started). |
| `fix/release-glibc-floor-native` | the commit adding this file | none | **PR A** (bindings only). Pushed without a PR, as instructed. |

## PR A (`fix/release-glibc-floor-native`): what it contains

- Native bindings (`@vizejs/native-linux-{x64,arm64}-gnu`, fresco too) are linked through a pinned `cargo-zigbuild` 0.24.1 (`.github/actions/setup-cargo-zigbuild`) at `<triple>.2.17`. The single source of the floor is `tools/support/github/glibc-floor.ts`. The gnu rows of `release-platforms.ts` set `cross_compile: true`, and `build-napi-package.ts` appends `.2.17`.
- The **CLI recipe is unchanged** from main (`build-release-cli` was reverted; `install-cross-compile-tools.ts` was restored).
- **Gate:** `verify-glibc-symbols.ts` reads the ELF in process (`tools/support/github/elf-dynamic-requirements.ts`). It fails on:
  - a symbol version above the floor, naming the symbols;
  - DT_RELR or GLIBC_ABI_DT_RELR;
  - GLIBC_PRIVATE or unknown versions;
  - libgcc_s newer than GCC 4.8;
  - non-base libraries.

  It runs in `build-release-native`, and the default `--max` is the floor.
- **Smoke:** `.github/actions/smoke-glibc-floor-containers` and `tools/commands/release/npm/smoke-glibc-floor-containers.ts`, in three modes:
  - packages: Amazon Linux 2023 and AlmaLinux 8, with `smoke-release-install --runtime-checks`;
  - binding: CentOS 7, with the unofficial Node 24 glibc-217 build;
  - cli: not used in PR A.

  `release.yml` gains a `smoke-glibc-floor` job that `tag` needs. `release-glibc-floor.yml` is the PR, weekly and dispatch gate (native jobs only).
- **Loader:** `npm/native/native-binding.js` reports:
  - the dlopen text;
  - "This system has glibc X; the binding asks for GLIBC_Y";
  - the 2.17 floor, plus either "packaging bug, report it" or "upgrade";
  - that musl cannot load in glibc processes (tested: `libc.so: cannot open shared object file`);
  - a docs link.

  When the platform package is missing, it names the package, the platform/glibc/Node version and the Node below `engines` case. The Node 20 case on AL2023 used to show the misleading npm-bug message.
- **Custody and harness adjustments:**
  - `tests/tooling/support/native-project-export-source.ts` rebuilds the original `native-binding.js` through a hash-pinned diagnostics block. **When you edit the diagnostics, update `loadDiagnosticsSha256`.**
  - `tools/support/compat/nuxt/source-native-entry.ts` (the Void source build) now pairs any loader the source changed instead of refusing it.
  - `.github/actionlint.yaml` gets the `blacksmith-32vcpu-ubuntu-{2204,2404}-arm` labels.
- **Docs:** `docs/content/stability.md` and `docs/content/ja/stability.md` gain a "Linux glibc floor" section (anchor `#linux-glibc-floor`) with a Vercel note. They say the gnu CLI archives are not covered yet and point to the musl archive.
- **Bench:** `.github/workflows/glibc-floor-bench.yml` and `tools/support/github/glibc-floor-bench.ts` give callgrind instruction counts plus before-`main` counts (a second run with `--toggle-collect=main`) and sizes for:
  - CLI: gnu-host (today), gnu-floor, musl-pie (today's musl), musl-static (non-PIE plus `--icf=all`);
  - binding: host and floor.

## Evidence already collected (for the PR A body)

- **Before (published 0.777.0):**
  - `objdump` max version: GLIBC_2.35 for both gnu `.node` files (`hypotf@2.35`, pthread/dl @2.34). The CLIs need 2.39 plus DT_RELR.
  - FAIL on amazonlinux:2023 x64/arm64 (Node 22/24), lambda nodejs:22/24, almalinux:8, ubuntu:20.04, debian bullseye.
  - PASS on bookworm and ubuntu 22.04.
  - Node 20 on AL2023 got the npm-bug message (the platform package was skipped because of engines).
- **After (#13242 CI run 44051803056, release recipe):**
  - Every gnu `.node` (native and fresco, x64 and arm64) reports GLIBC_2.17.
  - packages smoke PASS on AL2023 and Alma8 for x64 and arm64.
  - CentOS 7 binding load plus `compileSfc` PASS.
  - The local overlay matrix (0.777.0 JS + 2.17 binding) passes on every row above, including bullseye.
- **Wall bench (old, superseded, run 44055036214):** binding `compileSfc` 730×5 measured host 1.347 s vs floor 1.341 s. The floor `.node` is 76.5 MB vs 91.9 MB on the host.
- **Not done yet (needed before queueing):** instruction counts. The coordinator requires zero perf regression, measured by instructions.

## Next steps, in order (resume only after the coordinator lifts the CI pause)

1. Dispatch the bench: `gh workflow run glibc-floor-bench.yml --ref fix/release-glibc-floor-native -f runs=3`. It needs about 6 fat-LTO builds; the timeout is 240 min.
   - Read the step summary, or `gh run download <id> -n glibc-floor-bench` followed by `node tools/support/github/glibc-floor-bench.ts summarize <json>`.
   - The artifact includes the built binaries.
2. **PR A decision:** if the binding's floor instructions for `require(binding)` and `require + compileSfc` are ≤ host, and the size is ≤ host (it was smaller), open PR A from `fix/release-glibc-floor-native`.
   - Title: `fix(release): link the native bindings against glibc 2.17`.
   - Put the instruction and size tables in the body, add the evidence above, drive it green and queue it. Then close #13242 as superseded.
   - If the binding regresses: the host binding has no DT_RELR (Wild, RELA), so the relocation format is unchanged. Suspect zig's lld layout or the missing ICF. zig `cc` rejects `-Wl,--icf=all` and `-z pack-relative-relocs` ("unsupported linker arg").
3. **PR B (CLI, user decision "ネイティブにしたい"):** `vize` must start as a native executable, not through Node.
   - Today the npm launcher costs about 150 ms and 530M instructions; Node boot is ~310M, and loading the 79 MB addon adds ~70–90M.
   - The plan is per-platform npm packages `@vizejs/cli-<os>-<arch>` holding the native exe. Linux would get the static musl non-PIE build, but only if the bench shows musl-static ≤ gnu-host on every metric, including `vize lint <corpus>` throughput; musl memcpy may lose. macOS and Windows get native builds.
   - Required: an explicit, reported JS fallback; docs (en + ja); release artifacts; the gate and smoke in cli mode. Measure instructions plus quiet-runner wall for `--version`, single-file lint/check and LSP startup.
   - Command coverage: the Rust binary already covers every command `runCli` does (`vize_vitrine` `runCli` = `vize::cli::run_from_args`, and the TS/JS config loading lives in Rust, `vize_carton/src/config/loader/js.rs`).
     - JS-only: `vize init` and `vize setup` (`npm/cli/src/{init,setup}.ts`), and the `content-mapper` handshake (`npm/cli/src/content-mapper-launch.ts`).
     - The native exe must hand these to Node explicitly.
4. **PR B bin-linking evaluation (unfinished).** A local harness tested how each package manager links a bin; it lived in a now-lost scratch dir, and the script is reproduced below. Design notes from reading `cmd-shim`:
   - npm on Windows builds `.cmd` shims from the bin's shebang. With `#!/bin/sh` the shim calls `/bin/sh`, which is broken on Windows without Git bash. With no shebang, it runs the file directly, but Windows then needs a `.exe` name. So a single POSIX sh launcher bin cannot serve Windows.
   - esbuild-style postinstall replacement (copy the native exe over `bin/vize`) works only where `.bin` is a symlink and scripts run: npm, yarn1, yarn-berry `nodeLinker: node-modules`, and bun only with `trustedDependencies`. pnpm shims `exec node <bin>` for node-shebang bins, so it breaks after replacement unless pnpm links bins after build (to test). pnpm 10 blocks scripts by default (`onlyBuiltDependencies`). Yarn PnP runs bins through node, so it must fall back to JS. Windows keeps the JS launcher.
   - Candidate: a `#!/bin/sh` + JS polyglot bin (`":" //; <sh that resolves ../@vizejs/cli-$p/vize and nested node_modules, then exec>`) gives native start with zero install scripts on every POSIX PM. Yarn PnP then gets the JS path. Windows is unsolved (the shebang problem above).
   - The open question is a Windows strategy: either a postinstall that rewrites the `.cmd`/`.ps1` shims, or accept the JS launcher on Windows as a reported path. **This needs a user decision (see below).**

### PM harness (recreate under `target/` and run in `node:22-bookworm-slim` with `/pm` mounted)

- Mock packages:
  - `vizet` (bin `bin/vizet`, a JS launcher that prints `JS-LAUNCHER`, plus a `postinstall: node install.js` that copies `@vizet/cli-$platform-$arch/vizet` over `bin/vizet`);
  - `@vizet/cli-linux-{arm64,x64}` (`os`/`cpu`/`preferUnplugged`, holding a `zig cc -static` C program that prints `NATIVE argv0=`).
- `run.sh`:
  1. start verdaccio 6 on :4873 (anonymous publish for `vizet` and `@vizet/*`, proxying the rest) and publish the mocks;
  2. for each of npm, pnpm (default and `onlyBuiltDependencies: [vizet]`), yarn1, yarn4 node-modules, yarn4 PnP, bun (default and `trustedDependencies`): install `vizet`;
  3. record whether the install script ran, what `node_modules/.bin/vizet` runs (NATIVE vs JS-LAUNCHER), and the `.bin` entry type.

## Pending user / coordinator decisions

- **Windows native start for the npm `vize` bin.** Options:
  - a postinstall that rewrites the shims, which is fragile;
  - a reported JS launcher on Windows only.
- Whether Linux ships one static musl CLI per arch (no glibc split) for npm, depending on the bench.
- What to do with #13242: close it as superseded by PR A, or force-push PR A into it.

## Gotchas

- Wall time on this machine is not evidence; use CI callgrind. The callgrind output file's `summary:` line is the total. Parsing per-object costs from callgrind or `callgrind_annotate` did not add up, so the bench uses a `--toggle-collect=main` second run for the before-`main` count.
- zig `cc` 0.15 rejects `-Wl,--icf=all` and `-z pack-relative-relocs`. glibc 2.17's ld.so cannot process DT_RELR at all, so a floor-compatible gnu CLI loses both ICF and RELR. That is the coordinator's concern: the x64 CLI is +4% in size, and relocation is about 50% of the 3.1M instructions of `vize --version`.
- The Void build (`void-build.yml`) pins published `@vizejs/native` 0.474.0 and compares loader bytes. Any `native-binding.js` change needs the source-pairing fix that is already in PR A.
- The `public-typechecker-11317` custody hashes `native-binding.js`; update `loadDiagnosticsSha256` after any edit.
- `colima` (the Docker VM) has about 4 GB free; delete pulled images after container tests.
- The bench workflow can be dispatched from a branch even though the file isn't on main (`gh workflow run ... --ref <branch>` worked).
