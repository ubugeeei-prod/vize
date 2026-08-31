import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile } from "./support/github-workflows.ts";

function taskCommand(name: string): string {
  return (testAndBenchmarkTasks[name] as { command: string }).command;
}

test("the Zed real-server scenario has a task that runs its Rust Script launcher", () => {
  assert.equal(
    taskCommand("test:zed-extension:real-server"),
    "'rust-script' 'tools/commands/editors/zed/run-real-server.rs'",
  );
});

test("CI validates Zed with the pinned official extension CLI before the real-server scenario", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const installAt = action.indexOf("- name: Install pinned Zed extension CLI");
  const validateAt = action.indexOf("- name: Validate the Zed extension");
  const scenarioAt = action.indexOf("- name: Run the Zed scenario against the real server");

  assert.ok(installAt >= 0, "editor host CI must install the official Zed extension CLI");
  assert.ok(validateAt > installAt, "Zed validation must use the pinned CLI");
  assert.ok(scenarioAt > validateAt, "Zed validation must pass before the server scenario");

  const install = action.slice(installAt, validateAt);
  assert.match(install, /ZED_EXTENSION_CLI_SHA: [0-9a-f]{40}/);
  assert.match(install, /ZED_EXTENSION_CLI_SHA256: [0-9a-f]{64}/);
  assert.match(install, /sha256sum --check --strict/);
  assert.match(install, /x86_64-unknown-linux-gnu\/zed-extension/);

  const validate = action.slice(validateAt, scenarioAt);
  assert.match(validate, /working-directory: editors\/zed/);
  assert.match(validate, /--source-dir \./);
  assert.match(validate, /--scratch-dir "\$\{RUNNER_TEMP\}\/zed-extension-scratch"/);
  assert.match(validate, /--output-dir "\$\{RUNNER_TEMP\}\/zed-extension-output"/);

  const scenario = action.slice(scenarioAt);
  assert.match(scenario, /VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/);
  assert.match(scenario, /vp run --workspace-root test:zed-extension:real-server/);
});

test("the Zed real-server scenario pins complete extension-contract responses", () => {
  const scenario = readRepoFile("tools", "commands", "editors", "zed", "run-real-server.rs");
  const contract = readRepoFile("tools", "support", "editors", "lsp_smoke.rs");

  for (const method of [
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/codeAction",
    "textDocument/formatting",
    "textDocument/semanticTokens/full",
    "textDocument/rename",
  ]) {
    assert.match(contract, new RegExp(method.replace("/", "\\/")));
  }

  assert.match(contract, /assert_json_eq\([\s\S]*expected_diagnostics\(\)/);
  assert.match(contract, /assert_json_eq\(&completion, expected_completion\(\)/);
  assert.match(contract, /assert_json_eq\(&hover, expected_hover\(\)/);
  assert.match(contract, /assert_json_eq\(&code_actions, expected_code_actions\(&uri\)/);
  assert.match(contract, /assert_json_eq\(&response, expected_formatting\(\)/);
  assert.match(contract, /assert_json_eq\([\s\S]*"semantic tokens"/);
  assert.match(contract, /assert_json_eq\(&rename, expected_rename\(&uri\)/);
  assert.doesNotMatch(contract, /\.includes\(|\.contains\(/);

  // These are the exact recommended defaults returned by the extension. The
  // scenario then opts into formatting through the same initialization option
  // available to Zed users, matching the established Neovim scenario.
  assert.match(scenario, /run_editor_contract\(&common::repo_root\(\)\?, "zed", true\)/);
  assert.match(contract, /"editor": true/);
  assert.match(contract, /"typecheck": true/);
  assert.match(contract, /options\["formatting"\] = json!\(true\)/);
});
