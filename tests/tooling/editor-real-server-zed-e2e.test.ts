import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
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
  assert.match(install, /bash \.github\/actions\/vscode-host-smoke\/download-artifact\.sh/);
  assert.match(install, /"\$\{ZED_EXTENSION_CLI_SHA256\}"/);
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

  // The packaged adapter sends no feature overrides; the server discovers project settings.
  assert.match(scenario, /run_editor_contract\(&common::repo_root\(\)\?, "zed"\)/);
  assert.match(contract, /let options = json!\(\{\}\)/);
  assert.doesNotMatch(contract, /options\["formatting"\]/);
});

test("protocol replay retains complete stderr and forwards only failed process output", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-replay-stderr-"));
  const diagnostic = Buffer.from([0x70, 0x61, 0x6e, 0x69, 0x63, 0x0a, 0xff]);
  const helper = pathToFileURL(path.resolve("tests/editor-conformance/support/lsp-client.ts")).href;
  const server = `
    const fs = require('node:fs');
    let input = Buffer.alloc(0);
    let answered = false;
    process.stderr.write(Buffer.from(${JSON.stringify([...diagnostic])}));
    process.stdin.on('data', chunk => {
      input = Buffer.concat([input, chunk]);
      if (!answered && input.includes(Buffer.from('shutdown'))) {
        answered = true;
        const body = JSON.stringify({jsonrpc:'2.0',id:1,result:null});
        process.stdout.write('Content-Length: '+Buffer.byteLength(body)+'\\r\\n\\r\\n'+body);
      }
    });
    process.stdin.on('end', () => {
      fs.writeFileSync(process.argv[1], input);
      process.exit(Number(process.argv[2]));
    });
  `;
  try {
    for (const code of [0, 101]) {
      const transcript = path.join(directory, `${code}.jsonl`);
      const input = path.join(directory, `${code}.input`);
      const controller = `
        import { LspClient } from ${JSON.stringify(helper)};
        const client = new LspClient(process.execPath, ['-e', ${JSON.stringify(server)}, ${JSON.stringify(input)}, '${code}'], {
          cwd: ${JSON.stringify(directory)},
          env: {...process.env, VIZE_CONFORMANCE_TRANSCRIPT: ${JSON.stringify(transcript)}}
        }, () => null);
        const response = await client.request('shutdown', undefined);
        client.notify('exit', undefined);
        console.log(JSON.stringify({response, exitCode: await client.exit()}));
      `;
      const observed = spawnSync(process.execPath, ["--input-type=module", "-e", controller], {
        timeout: 5_000,
      });
      assert.ifError(observed.error);
      assert.equal(observed.status, 0);
      assert.deepEqual(JSON.parse(observed.stdout.toString()), { response: null, exitCode: code });
      assert.deepEqual(fs.readFileSync(`${transcript}.stderr.log`), diagnostic);
      assert.deepEqual(observed.stderr, code === 0 ? Buffer.alloc(0) : diagnostic);
      const messages = [
        { jsonrpc: "2.0", id: 1, method: "shutdown" },
        { jsonrpc: "2.0", method: "exit" },
      ].map((message) => {
        const body = JSON.stringify(message);
        return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;
      });
      assert.deepEqual(fs.readFileSync(input), Buffer.from(messages.join("")));
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
