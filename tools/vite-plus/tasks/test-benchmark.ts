import { cacheInputs, testedPackages } from "../task-inputs.ts";
import {
  defineTasks,
  moonScript,
  noCacheTask,
  runInPackages,
  runInVscodeExtension,
  rustTool,
  rustToolFromVscodeExtension,
  runTask,
  runTasks,
  task,
  vscodeExtensionPackageBin,
} from "../task-helpers.ts";
import { inTestbox } from "./testbox.ts";

const stageVscodeTypeScriptPlugin = rustToolFromVscodeExtension(
  "editors/vscode/sync-typescript-plugin",
  "stage",
);
const injectVscodeTypeScriptPlugin = rustToolFromVscodeExtension(
  "editors/vscode/sync-typescript-plugin",
  "inject",
  "dist/vize.vsix",
);
const assertVscodePackage = rustToolFromVscodeExtension(
  "editors/vscode/assert-vsix-package",
  "dist/vize.vsix",
);
const packageVscodeExtension = [
  stageVscodeTypeScriptPlugin,
  `${vscodeExtensionPackageBin("@vscode/vsce", "vsce")} package --no-dependencies --out dist/vize.vsix`,
  injectVscodeTypeScriptPlugin,
].join(" && ");
const localTestCommand = runTasks(
  "test:rust",
  "test:js",
  "test:scripts",
  "test:zed-extension:unit",
);

const jsPackageTestCommand = runInPackages("test", testedPackages, {
  concurrencyLimit: 1,
});

const rustSourceCoverageJson = "target/llvm-cov/source-summary.json";
const rustBranchCoverageJson = "target/llvm-cov/source-branch-summary.json";
const rustSourceCoverageMinimums = [
  "--min-lines",
  "70",
  "--min-functions",
  "70",
  "--min-regions",
  "70",
];
const rustBranchCoverageMinimums = [
  "--min-lines",
  "55",
  "--min-functions",
  "70",
  "--min-regions",
  "55",
  "--min-branches",
  "40",
];
const rustSourceCoverageCommand = [
  "mkdir -p target/llvm-cov",
  [
    "cargo llvm-cov --workspace --json --summary-only",
    `--output-path ${rustSourceCoverageJson}`,
    "--fail-under-lines 70 --fail-under-functions 70 --fail-under-regions 70",
  ].join(" "),
  moonScript(
    "enforce_rust_source_coverage",
    "--json",
    rustSourceCoverageJson,
    ...rustSourceCoverageMinimums,
  ),
].join(" && ");
const rustBranchCoverageCommand = [
  "mkdir -p target/llvm-cov",
  "cargo clean --target-dir target/llvm-cov-target",
  [
    "cargo +nightly llvm-cov -p vize_carton -p vize_armature -p vize_atelier_core",
    "--branch --json --summary-only",
    `--output-path ${rustBranchCoverageJson}`,
  ].join(" "),
  // Keep threshold enforcement in one place so failures still render the
  // complete metric table instead of stopping at cargo-llvm-cov's exit code.
  moonScript(
    "enforce_rust_source_coverage",
    "--json",
    rustBranchCoverageJson,
    ...rustBranchCoverageMinimums,
  ),
].join(" && ");

/**
 * Test, snapshot, coverage, and benchmark tasks.
 *
 * These commands validate observable behavior across Rust compiler crates,
 * JavaScript packages, browser-facing playground flows, and generated fixture
 * output. Snapshot and benchmark entries live beside normal tests because they
 * are both part of the same feedback loop: keep correctness visible and keep
 * performance regressions difficult to miss.
 */
export const testAndBenchmarkTasks = defineTasks({
  test: noCacheTask(localTestCommand),
  "test:testbox": noCacheTask(inTestbox(localTestCommand)),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  // Use the CI-profile native build instead of the release-profile one.
  // The release build was ~3m+ on GitHub Actions and was being immediately
  // overwritten by vite-plugin-vize's pretest hook, which also rebuilds in
  // dev profile (~1m20s). Building once in the CI profile saves both legs.
  "test:js": noCacheTask(`${runTask("build:native:test")} && ${jsPackageTestCommand}`),
  "test:scripts": noCacheTask(
    `${runTask("build:native:test")} && rust-script tools/rust/verify-layout.rs && VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1 tests/tooling/*.test.ts tests/tooling/*.test.mjs`,
  ),
  "test:vscode-extension:vsix": noCacheTask(
    runInVscodeExtension(packageVscodeExtension, assertVscodePackage),
  ),
  "test:vscode-extension:host": noCacheTask(
    runInVscodeExtension(
      stageVscodeTypeScriptPlugin,
      `${vscodeExtensionPackageBin("vite-plus", "vp")} pack`,
      "pnpm run test:host",
    ),
  ),
  "test:vscode-extension:host-real": noCacheTask(
    runInVscodeExtension(
      packageVscodeExtension,
      assertVscodePackage,
      "node test/run-extension-host-real.mjs",
    ),
  ),
  "test:zed-extension:package": noCacheTask("vp run --workspace-root package:zed-extension"),
  "test:zed-extension:unit": task("cargo test --manifest-path editors/zed/Cargo.toml", {
    input: ["editors/zed/**"],
  }),
  "test:zed-extension:real-server": noCacheTask(rustTool("editors/zed/run-real-server")),
  "test:nvim-extension:headless": noCacheTask(
    "nvim --headless -u NONE --noplugin '+set runtimepath^=editors/nvim' '+luafile editors/nvim/test/vize_spec.lua' '+qa' && VIZE_TEST_ART_VUE_NATIVE_PARSER=1 nvim --headless -u NONE --noplugin '+set runtimepath^=editors/nvim' '+luafile editors/nvim/test/vize_spec.lua' '+qa'",
  ),
  "test:nvim-extension:package": noCacheTask("vp run --workspace-root package:nvim-extension"),
  // The headless Neovim end-to-end scenario against a real `vize lsp` (#3457).
  // It needs a built server binary, so CI runs it in the same job that builds
  // one for the VS Code host smoke.
  "test:nvim-extension:real-server": noCacheTask(rustTool("editors/neovim/run-real-server")),
  "test:vim-extension:headless": noCacheTask(
    "vim -Nu NONE -n -es -S editors/vim/test/vize_spec.vim",
  ),
  "test:vim-extension:real-server": noCacheTask(rustTool("editors/vim/run-real-server")),
  "test:vim-extension:package": noCacheTask("vp run --workspace-root package:vim-extension"),
  "test:helix-extension:package": noCacheTask("vp run --workspace-root package:helix-extension"),
  "test:helix-extension:real-server": noCacheTask(rustTool("editors/helix/run-real-server")),
  "test:emacs-extension:headless": noCacheTask(
    "emacs -Q --batch -l ert -l editors/emacs/test/vize-test.el -f ert-run-tests-batch-and-exit",
  ),
  "test:emacs-extension:package": noCacheTask("vp run --workspace-root package:emacs-extension"),
  "test:playground": task(runInPackages("test:browser", ["./playground"]), {
    input: cacheInputs.jsChecks,
  }),
  "test:e2e": noCacheTask(runTasks("test:e2e:dev", "test:e2e:preview")),
  "test:e2e:dev": task(runInPackages("test:dev", ["./tests"]), { input: cacheInputs.e2e }),
  "test:e2e:preview": task(runInPackages("test:preview", ["./tests"]), {
    input: cacheInputs.e2e,
  }),
  "test:e2e:vrt": task(runInPackages("test:vrt", ["./tests"]), { input: cacheInputs.e2e }),
  "test:vue": task("cargo test -p vize_test_runner", { input: cacheInputs.rust }),
  coverage: task("cargo run -p vize_test_runner --bin coverage", { input: cacheInputs.rust }),
  "coverage:all": noCacheTask(runTasks("coverage", "coverage:source")),
  "coverage:source": task(rustSourceCoverageCommand, {
    env: ["VIZE_NUXT_CONFIG_ITERATIONS"],
    input: cacheInputs.rust,
  }),
  "coverage:source:branch": task(rustBranchCoverageCommand, { input: cacheInputs.rust }),
  "coverage:verbose": task("cargo run -p vize_test_runner --bin coverage -- -v", {
    input: cacheInputs.rust,
  }),
  "coverage:diff": task("cargo run -p vize_test_runner --bin coverage -- -vv", {
    input: cacheInputs.rust,
  }),
  "generate:rule-types": task(moonScript("generate_rule_types"), {
    input: cacheInputs.rust,
  }),
  "expected:generate": task(moonScript("generate_expected")),
  "expected:generate:sfc": task(moonScript("generate_expected", "--mode", "sfc")),
  "expected:generate:vdom": task(moonScript("generate_expected", "--mode", "vdom")),
  "expected:generate:vapor": task(moonScript("generate_expected", "--mode", "vapor")),
  snapshot: noCacheTask(runTasks("snapshot:test", "snapshot:review")),
  "snapshot:test": task("cargo insta test -p vize_atelier_sfc -- snapshot_tests"),
  "snapshot:review": noCacheTask("cargo insta review"),
  "snapshot:accept": noCacheTask("cargo insta accept"),
  bench: noCacheTask(moonScript("bench", "run")),
  "bench:quick": noCacheTask(moonScript("bench", "run", "1000")),
  "bench:generate": noCacheTask(moonScript("bench", "generate", "15000")),
  "bench:lint": noCacheTask(moonScript("bench", "lint")),
  "bench:fmt": noCacheTask(moonScript("bench", "fmt")),
  "bench:check": noCacheTask(moonScript("bench", "check")),
  "bench:vite": noCacheTask(moonScript("bench", "vite")),
  // Published, not enforcing: reports what @vizejs/vite-plugin-musea and
  // @vizejs/musea-nuxt cost a gallery build, and carries no baseline of its
  // own so benchmark.yml's fixed-baseline schedule (#3586) stays the repo's
  // single drift gate. Needs build:native:test, build:vite-plugin and
  // build:nuxt-stack, and says so when they are missing.
  "bench:musea": noCacheTask(moonScript("bench", "musea")),
  "bench:compare-tools": noCacheTask(
    "node bench/compare-tools.mjs --input bench/__in__ --out target/tool-benchmark-summary.md --json target/tool-benchmark-results.json --doc target/performance-blacksmith.md",
  ),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite", "bench:musea"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
});
