import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile } from "./support/github-workflows.ts";

// Guardrails for the #3457 real-server editor scenarios. A scenario nobody
// executes is not coverage, so these assert the wiring that makes CI run them,
// not the scenario contents themselves.

function taskCommand(name: string): string {
  return (testAndBenchmarkTasks[name] as { command: string }).command;
}

// Composite-action steps in declaration order, so tests can assert what a step
// actually runs and that it runs before the step depending on it. Steps written
// as `run: |` blocks report `|`; the callers below only need single-line steps.
function actionSteps(action: string): Array<{ name: string; run: string }> {
  return action
    .split(/\n {4}- name: /)
    .slice(1)
    .map((chunk) => {
      const [name, ...rest] = chunk.split("\n");
      return { name, run: /^ {6}run: (.*)$/m.exec(rest.join("\n"))?.[1] ?? "" };
    });
}

test("the Neovim real-server scenario has a task that runs its Node launcher", () => {
  assert.equal(
    taskCommand("test:nvim-extension:real-server"),
    "node tools/nvim-vize/run-real-server.mjs",
  );
});

test("CI runs both real-server editor scenarios from one built server binary", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

  // One `cargo build` feeds both scenarios; both must consume that binary.
  assert.match(action, /run: cargo build --profile ci -p vize/);
  assert.deepEqual(
    action.match(/VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/g),
    [
      "VIZE_SERVER_PATH: ${{ github.workspace }}/target/ci/vize",
      "VIZE_SERVER_PATH: ${{ github.workspace }}/target/ci/vize",
    ],
  );
  assert.match(action, /vp run --workspace-root test:vscode-extension:host-real/);
  assert.match(action, /vp run --workspace-root test:nvim-extension:real-server/);

  // The scenario needs a Neovim with a Lua LSP client, pinned and checksummed.
  assert.match(action, /NVIM_VERSION: v\d+\.\d+\.\d+/);
  assert.match(action, /NVIM_SHA256: [0-9a-f]{64}/);
  assert.match(action, /sha256sum --check --strict/);
});

test("CI runs both headless editor specs", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

  assert.match(taskCommand("test:nvim-extension:headless"), /VIZE_TEST_ART_VUE_NATIVE_PARSER=1/);
  assert.match(action, /vp run --workspace-root test:nvim-extension:headless/);
  assert.match(action, /vp run --workspace-root test:vim-extension:headless/);
  assert.match(action, /command -v vim/);
});

test("CI executes the packaged Emacs ERT suite", () => {
  const steps = actionSteps(readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml"));

  assert.equal(
    taskCommand("test:emacs-extension:headless"),
    "emacs -Q --batch -l ert -l editors/emacs/test/vize-test.el -f ert-run-tests-batch-and-exit",
  );

  // The interpreter is only guaranteed by the emacs-nox fallback, so the setup
  // step and its position ahead of the ERT task are one contract.
  const setupIndex = steps.findIndex((step) => step.name === "Ensure Emacs is available");
  const runIndex = steps.findIndex((step) => step.name === "Run the packaged Emacs ERT suite");

  assert.equal(
    steps[setupIndex]?.run,
    "command -v emacs || (sudo apt-get update && sudo apt-get install -y emacs-nox)",
  );
  assert.equal(steps[runIndex]?.run, "vp run --workspace-root test:emacs-extension:headless");
  assert.ok(setupIndex < runIndex, "Emacs setup must run before the ERT suite");
});

test("check.yml keeps the editor real-server job in the required set", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(
    workflow,
    /\n  editor-host-smoke:\n[\s\S]*?uses: \.\/\.github\/actions\/vscode-host-smoke\n/,
  );
  assert.match(workflow, /\n      - editor-host-smoke\n/);
});

test("the packaged Neovim archive ships the real-server scenario", () => {
  const assertion = readRepoFile("tools", "nvim-vize", "assert-nvim-package.mjs");

  assert.match(assertion, /"nvim\/test\/vize_e2e_expected\.lua"/);
  assert.match(assertion, /"nvim\/test\/vize_e2e_spec\.lua"/);
  assert.match(assertion, /\^nvim\\\/test\\\/vize_e2e_\(\?:expected\|spec\)\\\.lua\$/);
});

test("the Neovim real-server scenario pins completion and hover responses", () => {
  const scenario = readRepoFile("editors", "nvim", "test", "vize_e2e_spec.lua");
  const expected = readRepoFile("editors", "nvim", "test", "vize_e2e_expected.lua");

  assert.match(scenario, /textDocument\/completion/);
  assert.match(scenario, /textDocument\/hover/);
  assert.match(scenario, /sorted_matching_labels\(items, expected\.completion_include\)/);
  assert.match(scenario, /sorted_matching_labels\(items, expected\.completion_exclude\)/);
  assert.match(scenario, /assert_eq\(result, expected\.hover, "script binding hover"\)/);
  assert.doesNotMatch(scenario, /\.includes\(|\.contains\(/);

  assert.match(expected, /completion_include = \{ "Child", "total" \}/);
  assert.match(expected, /completion_exclude = \{ "count", "v-if" \}/);
  assert.match(expected, /const total: "3"/);
  assert.match(expected, /character = 11, line = 3/);
});

test("the VS Code real-server suite drives all five scorecard steps", () => {
  const scenario = readRepoFile("editors", "vscode", "test", "suite", "real-scenario.cjs");

  assert.match(scenario, /vscode\.executeCodeActionProvider/);
  assert.match(scenario, /vscode\.executeFormatDocumentProvider/);
  assert.match(scenario, /vscode\.provideDocumentSemanticTokens/);
  assert.match(scenario, /vscode\.executeDocumentRenameProvider/);
  assert.match(scenario, /document\.save\(\)/);

  // Step 1 and step 3 are the two steps whose value lives in what is asserted
  // rather than in the call itself: diagnostics must be compared against the
  // authored-span expectation, and the save must be followed by a check that
  // the document really came back formatted.
  assert.match(
    scenario,
    /assert\.deepEqual\(sortDiagnostics\(diagnostics\)\.map\(describeDiagnostic\), expected\.diagnostics\)/,
  );
  assert.match(
    scenario,
    /const saved = await document\.save\(\);[\s\S]*?assert\.equal\(document\.getText\(\), expected\.formattedSource,/,
  );

  // Rule for this suite: complete-response assertions only.
  assert.doesNotMatch(scenario, /\.includes\(|\.contains\(/);
});
