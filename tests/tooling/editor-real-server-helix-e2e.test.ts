import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile } from "./support/github-workflows.ts";

function taskCommand(name: string): string {
  return (testAndBenchmarkTasks[name] as { command: string }).command;
}

test("the Helix real-server scenario has a task that runs its Rust Script launcher", () => {
  assert.equal(
    taskCommand("test:helix-extension:real-server"),
    "'rust-script' 'tools/commands/editors/helix/run-real-server.rs'",
  );
});

test("CI checks the package with a pinned official Helix before the server scenario", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const installAt = action.indexOf("- name: Install pinned Helix");
  const healthAt = action.indexOf("- name: Check the Helix package with the official CLI");
  const scenarioAt = action.indexOf("- name: Run the Helix scenario against the real server");

  assert.ok(installAt >= 0, "editor host CI must install the official Helix binary");
  assert.ok(healthAt > installAt, "Helix health checks must use the pinned binary");
  assert.ok(scenarioAt > healthAt, "the package must pass Helix health before the scenario");

  const install = action.slice(installAt, healthAt);
  assert.match(install, /HELIX_VERSION: \d+\.\d+\.\d+/);
  assert.match(install, /HELIX_SHA256: [0-9a-f]{64}/);
  assert.match(install, /sha256sum --check --strict/);
  assert.match(install, /helix-\$\{HELIX_VERSION\}-x86_64-linux\.tar\.xz/);

  const health = action.slice(healthAt, scenarioAt);
  assert.match(health, /XDG_CONFIG_HOME: \$\{\{ runner\.temp \}\}\/helix-config/);
  assert.match(health, /HELIX_RUNTIME: \$\{\{ runner\.temp \}\}\/helix\/runtime/);
  assert.match(health, /VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/);
  assert.match(health, /rust-script tools\/commands\/editors\/helix\/assert-helix-health\.rs/);

  const scenario = action.slice(scenarioAt);
  assert.match(scenario, /VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/);
  assert.match(scenario, /vp run --workspace-root test:helix-extension:real-server/);
});

test("the official Helix health guard checks both packaged language entries exactly", () => {
  const health = readRepoFile("tools", "commands", "editors", "helix", "assert-helix-health.rs");

  assert.match(health, /\["--health", language\]/);
  assert.match(health, /for language in \["vue", "art-vue"\]/);
  assert.match(health, /configured != vec!\[expected_server\.clone\(\)\]/);
  assert.match(health, /if !output\.status\.success\(\)/);
  assert.doesNotMatch(health, /configured\.contains\(/);
});

test("the Helix scenario pins complete responses for every advertised feature", () => {
  const scenario = readRepoFile("tools", "commands", "editors", "helix", "run-real-server.rs");
  const contract = readRepoFile("tools", "rust", "lsp_smoke.rs");

  for (const method of [
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/codeAction",
    "textDocument/semanticTokens/full",
    "textDocument/rename",
  ]) {
    assert.match(contract, new RegExp(method.replace("/", "\\/")));
  }

  assert.match(contract, /assert_json_eq\([\s\S]*expected_diagnostics\(\)/);
  assert.match(contract, /assert_json_eq\(&completion, expected_completion\(\)/);
  assert.match(contract, /assert_json_eq\(&hover, expected_hover\(\)/);
  assert.match(contract, /assert_json_eq\(&code_actions, expected_code_actions\(&uri\)/);
  assert.match(contract, /assert_json_eq\([\s\S]*"semantic tokens"/);
  assert.match(contract, /assert_json_eq\(&rename, expected_rename\(&uri\)/);
  assert.match(scenario, /run_editor_contract\(&repo, "helix", false\)/);
  assert.doesNotMatch(scenario, /textDocument\/formatting/);
  assert.doesNotMatch(contract, /\.includes\(|\.contains\(/);

  // This is the exact config Helix sends. Formatting is intentionally absent,
  // so the scenario verifies only the capabilities the package advertises.
  assert.match(scenario, /common::read_text\(repo\.join\("editors\/helix\/languages\.toml"\)\)/);
  assert.match(scenario, /"editor = true"/);
  assert.match(scenario, /"typecheck = true"/);
  assert.match(contract, /document_formatting_provider\.is_some_and/);
});
