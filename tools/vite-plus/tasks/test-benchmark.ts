import { cacheInputs, testedPackages } from "../task-inputs.ts";
import {
  defineTasks,
  moonScript,
  noCacheTask,
  runInPackages,
  runInVscodeExtension,
  runTask,
  runTasks,
  task,
} from "../task-helpers.ts";
import { inTestbox } from "./testbox.ts";

const jsPackageTestCommand = runInPackages("test", testedPackages, {
  concurrencyLimit: 1,
});

const rustSourceCoverageJson = "target/llvm-cov/source-summary.json";
const rustBranchCoverageJson = "target/llvm-cov/source-branch-summary.json";
const rustSourceCoverageMinimums = "--min-lines 70 --min-functions 70 --min-regions 70";
const rustBranchCoverageMinimums =
  "--min-lines 55 --min-functions 70 --min-regions 55 --min-branches 40";
const rustSourceCoverageCommand = [
  "mkdir -p target/llvm-cov",
  [
    "cargo llvm-cov --workspace --json --summary-only",
    `--output-path ${rustSourceCoverageJson}`,
    "--fail-under-lines 70 --fail-under-functions 70 --fail-under-regions 70",
  ].join(" "),
  [
    "node tools/coverage/enforce-rust-source-coverage.mjs",
    `--json ${rustSourceCoverageJson}`,
    rustSourceCoverageMinimums,
  ].join(" "),
].join(" && ");
const rustBranchCoverageCommand = [
  "mkdir -p target/llvm-cov",
  [
    "cargo +nightly llvm-cov -p vize_carton -p vize_armature -p vize_atelier_core",
    "--branch --json --summary-only",
    `--output-path ${rustBranchCoverageJson}`,
    "--fail-under-lines 55 --fail-under-functions 70 --fail-under-regions 55",
  ].join(" "),
  [
    "node tools/coverage/enforce-rust-source-coverage.mjs",
    `--json ${rustBranchCoverageJson}`,
    rustBranchCoverageMinimums,
  ].join(" "),
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
  // `vp test` runs inside a Blacksmith Testbox; the underlying test:* tasks
  // stay local. See tools/vite-plus/tasks/testbox.ts.
  test: noCacheTask(
    inTestbox(runTasks("test:rust", "test:js", "test:scripts", "test:zed-extension:unit")),
  ),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  // Use the CI-profile native build instead of the release-profile one.
  // The release build was ~3m+ on GitHub Actions and was being immediately
  // overwritten by vite-plugin-vize's pretest hook, which also rebuilds in
  // dev profile (~1m20s). Building once in the CI profile saves both legs.
  "test:js": noCacheTask(`${runTask("build:native:test")} && ${jsPackageTestCommand}`),
  "test:scripts": noCacheTask(
    `${runTask("build:native:test")} && node --test --test-concurrency=1 tests/tooling/*.test.ts`,
  ),
  "test:vscode-extension:vsix": noCacheTask(
    runInVscodeExtension(
      "pnpm exec vsce package --no-dependencies --out dist/vize.vsix",
      "node ../../tools/vscode-vize/assert-vsix-package.mjs dist/vize.vsix",
    ),
  ),
  "test:vscode-extension:host": noCacheTask(
    runInVscodeExtension("pnpm exec vp pack", "pnpm run test:host"),
  ),
  "test:zed-extension:package": noCacheTask("vp run --workspace-root package:zed-extension"),
  "test:zed-extension:unit": task("cargo test --manifest-path editors/zed/Cargo.toml", {
    input: ["editors/zed/**"],
  }),
  "test:nvim-extension:headless": noCacheTask(
    "nvim --headless -u NONE --noplugin '+set runtimepath^=editors/nvim' '+luafile editors/nvim/test/vize_spec.lua' '+qa'",
  ),
  "test:nvim-extension:package": noCacheTask("vp run --workspace-root package:nvim-extension"),
  "test:vim-extension:headless": noCacheTask(
    "vim -Nu NONE -n -es -S editors/vim/test/vize_spec.vim",
  ),
  "test:vim-extension:package": noCacheTask("vp run --workspace-root package:vim-extension"),
  "test:helix-extension:package": noCacheTask("vp run --workspace-root package:helix-extension"),
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
  "coverage:source": task(rustSourceCoverageCommand, { input: cacheInputs.rust }),
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
  "bench:compare-tools": noCacheTask(
    "node bench/compare-tools.mjs --input bench/__in__ --out target/tool-benchmark-summary.md --json target/tool-benchmark-results.json --doc target/performance-blacksmith.md",
  ),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
});
