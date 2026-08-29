import assert from "node:assert/strict";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
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
  const health = readRepoFile("tools", "helix-vize", "assert-helix-health.mjs");

  assert.match(health, /\["--health", language\]/);
  assert.match(health, /for \(const language of \["vue", "art-vue"\]\)/);
  assert.match(health, /assert\.deepEqual\(\s*configuredServers,\s*\[expectedServer\]/);
  assert.match(health, /assert\.equal\(result\.status, 0/);
  assert.doesNotMatch(health, /\.includes\(|\.contains\(/);
});

test("the Helix scenario pins complete responses for every advertised feature", () => {
  const scenario = readRepoFile("tools", "helix-vize", "run-real-server.mjs");

  for (const method of [
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/codeAction",
    "textDocument/semanticTokens/full",
    "textDocument/rename",
  ]) {
    assert.match(scenario, new RegExp(method.replace("/", "\\/")));
  }

  assert.match(scenario, /assert\.deepEqual\(diagnostics\.diagnostics, expectedDiagnostics\)/);
  assert.match(scenario, /assert\.deepEqual\(completion, expectedCompletion\)/);
  assert.match(scenario, /assert\.deepEqual\(hover, expectedHover\)/);
  assert.match(scenario, /assert\.deepEqual\(codeActions, expectedCodeActions\(uri\)\)/);
  assert.match(scenario, /assert\.deepEqual\(semanticTokens, expectedSemanticTokens\)/);
  assert.match(scenario, /assert\.deepEqual\(rename, expectedRename\(uri\)\)/);
  assert.doesNotMatch(scenario, /textDocument\/formatting/);
  assert.doesNotMatch(scenario, /\.includes\(|\.contains\(/);

  // This is the exact config Helix sends. Formatting is intentionally absent,
  // so the scenario verifies only the capabilities the package advertises.
  assert.match(scenario, /toml\.parse\(fs\.readFileSync\(helixConfigPath, "utf8"\)\)/);
  assert.match(scenario, /assert\.deepEqual\(helixServer\.config, helixRecommendedOptions\)/);
  assert.match(scenario, /assert\.equal\(capabilities\.documentFormattingProvider, undefined\)/);
});
