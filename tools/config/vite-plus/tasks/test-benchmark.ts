import { cacheInputs, testedPackages } from "../task-inputs.ts";
import { testedPackageGroups } from "../tested-package-groups.ts";
import {
  assertVscodePackage,
  packageVscodeExtension,
  stageVscodeTypeScriptPlugin,
  steps,
  ciCommand,
  defineTasks,
  nodeCommand,
  nodeTool,
  noCacheTask,
  runInPackages,
  runInVscodeExtension,
  runPackageScriptDirectly,
  rustTool,
  runTask,
  runTasks,
  task,
  vscodeExtensionPackageBin,
} from "../task-helpers.ts";
import type { PackagePath } from "../task-helpers.ts";
import { inTestbox } from "./testbox.ts";

const localTestCommand = runTasks(
  "test:rust",
  "test:js",
  "test:scripts",
  "test:zed-extension:unit",
);
const toolingTestFiles = [
  "tests/tooling/*.test.ts",
  "tests/tooling/*.test.mjs",
  "tests/tooling/davinci/*.test.ts",
  "tests/tooling/davinci/*.test.mjs",
  "tests/tooling/release/*.test.ts",
];

// The Vite package reuses the successful native preparation. Keep the same
// package selection, dependency ordering, concurrency, and inherited env.
const preparedPackageTests = (packages: readonly PackagePath[]) => {
  const command = runInPackages("test", packages, { concurrencyLimit: 1 });
  return {
    ...command,
    program: "node",
    arguments: [
      "npm/native/scripts/test-preparation.mjs",
      command.program,
      ...(command.arguments ?? []),
    ],
  };
};
const nativePreparedPackageTests = (packages: readonly PackagePath[]) =>
  steps(runTask("build:native:test"), preparedPackageTests(packages));

const prepareToolingTests = steps(runTask("build:native:test"), ciCommand("verify-tool-layout"));
const plannedToolingTests = steps(
  prepareToolingTests,
  "node tools/support/compat/github/run-tooling-tests.mjs",
);
const nvimTest = {
  program: "nvim",
  arguments: ["--headless", "-u", "NONE", "--noplugin", "-l", "editors/nvim/test/vize_spec.lua"],
};

// Pull-request CI runs one group per job. A group keeps the full set's package
// order and builds its out-of-group dependencies' bundles before testing.
const jsPackageGroupTasks = Object.fromEntries(
  testedPackageGroups.map(({ name, packages, prebuilt }) => {
    const selected = testedPackages.filter((pkg) =>
      (packages as readonly PackagePath[]).includes(pkg),
    );
    const commands = steps(
      runTask("build:native:test"),
      ...(prebuilt.length > 0 ? [runPackageScriptDirectly("build", prebuilt)] : []),
      preparedPackageTests(selected),
    );
    return [`test:js:${name}`, noCacheTask(commands)];
  }),
);

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
const benchmarkDispatcher = (...args: string[]) => nodeCommand("benchmarks/dispatch", ...args);
const sourceCoverage = (...args: string[]) =>
  ["node", "tools/ci/commands/source-coverage.ts", ...args].join(" ");
const rustSourceCoverageCommand = steps(
  nodeTool("tasks/prepare-coverage-directory"),
  [
    // nextest runs the instrumented test binaries in parallel; libtest ran them one by one.
    // The full profile's groups apply, and eight threads leave the slower instrumented
    // real-TSGO tests the headroom that 32 threads did not.
    "cargo llvm-cov nextest --workspace --profile full --test-threads 8 --json --summary-only",
    `--output-path ${rustSourceCoverageJson}`,
  ].join(" "),
  sourceCoverage("--json", rustSourceCoverageJson, ...rustSourceCoverageMinimums),
);
const rustBranchCoverageCommand = steps(
  nodeTool("tasks/prepare-coverage-directory"),
  "cargo clean --target-dir target/llvm-cov-target",
  [
    "cargo +nightly llvm-cov -p vize_carton -p vize_armature -p vize_atelier_core",
    "--branch --json --summary-only",
    `--output-path ${rustBranchCoverageJson}`,
  ].join(" "),
  // Keep threshold enforcement in one place so failures still render the
  // complete metric table instead of stopping at cargo-llvm-cov's exit code.
  sourceCoverage("--json", rustBranchCoverageJson, ...rustBranchCoverageMinimums),
);

export const testAndBenchmarkTasks = defineTasks({
  test: noCacheTask(localTestCommand),
  "test:testbox": noCacheTask(inTestbox("test")),
  "test:rust": task("cargo test --workspace", { input: cacheInputs.rust }),
  "test:js": noCacheTask(nativePreparedPackageTests(testedPackages)),
  ...jsPackageGroupTasks,
  // The PR Oxlint qualification job builds the same source addon and the
  // bundles the oxlint package test packs, without the package tests.
  "build:oxlint-qualification": noCacheTask(
    steps(runTask("build:native:test"), runPackageScriptDirectly("build", ["./npm/linter/oxlint"])),
  ),
  "test:scripts": noCacheTask(
    steps(prepareToolingTests, {
      program: "node",
      arguments: ["--test", "--test-concurrency=1", ...toolingTestFiles],
      globs: toolingTestFiles,
      environment: { VIZE_TEST_REQUIRE_TSGO: "1" },
    }),
  ),
  "test:scripts:pr": noCacheTask(plannedToolingTests),
  "test:scripts:planned": noCacheTask(plannedToolingTests),
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
  "test:vscode-extension:download": noCacheTask(
    runInVscodeExtension("node test/download-vscode.mjs"),
  ),
  "test:zed-extension:package": noCacheTask("vp run --workspace-root package:zed-extension"),
  "test:zed-extension:unit": task("cargo test --manifest-path editors/zed/Cargo.toml", {
    input: ["editors/zed/**"],
  }),
  "test:zed-extension:real-server": noCacheTask(rustTool("editors/zed/run-real-server")),
  "test:nvim-extension:headless": noCacheTask(
    steps(nvimTest, { ...nvimTest, environment: { VIZE_TEST_ART_VUE_NATIVE_PARSER: "1" } }),
  ),
  "test:nvim-extension:package": noCacheTask("vp run --workspace-root package:nvim-extension"),
  // The headless Neovim end-to-end scenario against a real `vize lsp` (#3457).
  // It needs a built server binary, so CI runs it in the same job that builds
  // one for the VS Code host smoke.
  "test:nvim-extension:real-server": noCacheTask(nodeCommand("editors/neovim/run-real-server")),
  "test:vim-extension:headless": noCacheTask(
    "vim -Nu NONE -n -es -S editors/vim/test/vize_spec.vim",
  ),
  "test:vim-extension:real-server": noCacheTask(nodeCommand("editors/vim/run-real-server")),
  "test:vim-extension:package": noCacheTask("vp run --workspace-root package:vim-extension"),
  "test:helix-extension:package": noCacheTask("vp run --workspace-root package:helix-extension"),
  "test:helix-extension:real-server": noCacheTask(rustTool("editors/helix/run-real-server")),
  "test:emacs-extension:headless": noCacheTask(
    "emacs -Q --batch -l ert -l editors/emacs/test/vize-test.el -f ert-run-tests-batch-and-exit",
  ),
  "test:emacs-extension:real-server": noCacheTask(nodeTool("compat/emacs-vize/run-real-server")),
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
    env: [
      "VIZE_NUXT_CONFIG_ITERATIONS",
      "VIZE_NATIVE_SYNTAX_CAPTURE_DIR",
      "CORSA_PATH",
      "CORSA_EXECUTABLE",
      "TSGO_PATH",
      "TSGO_EXECUTABLE",
      "TSRS_PATH",
      "TSRS_EXECUTABLE",
    ],
    input: cacheInputs.rust,
  }),
  "coverage:source:branch": task(rustBranchCoverageCommand, { input: cacheInputs.rust }),
  "coverage:verbose": task("cargo run -p vize_test_runner --bin coverage -- -v", {
    input: cacheInputs.rust,
  }),
  "coverage:diff": task("cargo run -p vize_test_runner --bin coverage -- -vv", {
    input: cacheInputs.rust,
  }),
  "generate:rule-types": task(nodeCommand("generate/rule-types"), {
    input: [
      "tools/commands/generate/rule-types.ts",
      "tools/support/command-runtime.ts",
      "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
    ],
  }),
  "expected:generate": task(nodeTool("tasks/generate-expected")),
  "expected:generate:sfc": task(nodeTool("tasks/generate-expected", "--mode", "sfc")),
  "expected:generate:vdom": task(nodeTool("tasks/generate-expected", "--mode", "vdom")),
  "expected:generate:vapor": task(nodeTool("tasks/generate-expected", "--mode", "vapor")),
  snapshot: noCacheTask(runTasks("snapshot:test", "snapshot:review")),
  "snapshot:test": task("cargo insta test -p vize_atelier_sfc -- snapshot_tests"),
  "snapshot:review": noCacheTask("cargo insta review"),
  "snapshot:accept": noCacheTask("cargo insta accept"),
  bench: noCacheTask(benchmarkDispatcher("run")),
  "bench:quick": noCacheTask(benchmarkDispatcher("run", "1000")),
  "bench:generate": noCacheTask(benchmarkDispatcher("generate", "15000")),
  "bench:lint": noCacheTask(benchmarkDispatcher("lint")),
  "bench:fmt": noCacheTask(benchmarkDispatcher("fmt")),
  "bench:check": noCacheTask(benchmarkDispatcher("check")),
  "bench:vite": noCacheTask(benchmarkDispatcher("vite")),
  // Published, not enforcing: reports what @vizejs/vite-plugin-musea and
  // @vizejs/musea-nuxt cost a gallery build, and carries no baseline of its
  // own so 03-source-comparison.yml's fixed-baseline schedule (#3586) stays the repo's
  // single drift gate. Needs build:native:test, build:vite-plugin and
  // build:nuxt-stack, and says so when they are missing.
  "bench:musea": noCacheTask(benchmarkDispatcher("musea")),
  "bench:compare-tools": noCacheTask(
    "node tools/benchmarks/scripts/compare-tools.mjs --input tools/benchmarks/scripts/__in__ --out target/tool-benchmark-summary.md --json target/tool-benchmark-results.json --doc target/performance-blacksmith.md",
  ),
  "bench:all": noCacheTask(
    runTasks("bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite", "bench:musea"),
  ),
  "bench:rust": noCacheTask("cargo bench -p vize_atelier_sfc"),
});
