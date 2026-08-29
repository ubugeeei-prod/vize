import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { testAndBenchmarkTasks } from "../../tools/vite-plus/tasks/test-benchmark.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

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

// The `env:` entries a named composite-action step passes, with YAML quoting
// removed, so tests can assert the values a step actually exports instead of the
// formatting the workflow happens to use.
function stepEnvironment(action: string, stepName: string): Record<string, string> {
  const stepAt = action.indexOf(`- name: ${stepName}\n`);
  assert.ok(stepAt >= 0, `missing composite-action step: ${stepName}`);
  const step = action.slice(stepAt).split(/\n {4}- name: /)[0];
  const environment = /^ {6}env:\n((?: {8}\S.*\n?)*)/m.exec(step)?.[1] ?? "";

  return Object.fromEntries(
    environment
      .split("\n")
      .filter((line) => line.trim().length > 0)
      .map((line) => {
        const [, key, value] = /^ {8}([\w.-]+): *(.*?) *$/.exec(line) ?? [];
        assert.ok(key, `unparsed env entry in ${stepName}: ${line}`);
        return [key, value.replace(/^(["'])(.*)\1$/, "$2")];
      }),
  );
}

// Packs `editors/vim` the way `package:vim-extension` does — optionally
// dropping entries first — and runs the packaging guard over the archive, so
// tests can assert what the guard accepts and rejects instead of its source.
function runVimPackageGuard(omit: string[] = []): { status: number; output: string } {
  const workdir = fs.mkdtempSync(path.join(os.tmpdir(), "vim-vize-package-"));
  try {
    const editors = path.join(workdir, "editors");
    fs.cpSync(path.join(root, "editors", "vim"), path.join(editors, "vim"), { recursive: true });
    for (const entry of omit) {
      fs.rmSync(path.join(editors, entry));
    }

    const archive = path.join(workdir, "vim-vize-extension.tar.gz");
    execFileSync("tar", ["-czf", archive, "-C", editors, "vim"], {
      env: { ...process.env, COPYFILE_DISABLE: "1", LC_ALL: "C", LANG: "C" },
    });

    const guard = spawnSync(
      process.execPath,
      [path.join(root, "tools", "vim-vize", "assert-vim-package.mjs"), archive],
      { encoding: "utf8" },
    );
    return { status: guard.status ?? 1, output: `${guard.stdout ?? ""}${guard.stderr ?? ""}` };
  } finally {
    fs.rmSync(workdir, { force: true, recursive: true });
  }
}

test("the Neovim real-server scenario has a task that runs its Rust Script launcher", () => {
  assert.equal(
    taskCommand("test:nvim-extension:real-server"),
    "'rust-script' 'tools/commands/editors/neovim/run-real-server.rs'",
  );
});

test("the Vim real-server scenario has a task that runs its Rust Script launcher", () => {
  assert.equal(
    taskCommand("test:vim-extension:real-server"),
    "'rust-script' 'tools/commands/editors/vim/run-real-server.rs'",
  );
});

test("CI runs all real-server editor scenarios from one built server binary", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

  // One `cargo build` feeds all scenarios; each must consume that binary.
  assert.match(action, /run: cargo build --profile ci -p vize/);
  const serverPathUses =
    action.match(/VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/g) ?? [];
  assert.equal(
    serverPathUses.length,
    6,
    "every real-server scenario and host health check must consume the one built binary",
  );
  assert.match(action, /vp run --workspace-root test:vscode-extension:host-real/);
  // The pin is the value the step passes, not how YAML happens to quote it.
  assert.equal(
    stepEnvironment(action, "Run VS Code host smoke against the real server")
      .VIZE_TEST_VSCODE_VERSION,
    "1.107.1",
    "the VS Code host smoke step must pin the editor build it downloads",
  );
  assert.match(action, /vp run --workspace-root test:nvim-extension:real-server/);
  assert.match(action, /vp run --workspace-root test:vim-extension:real-server/);
  assert.match(action, /vp run --workspace-root test:zed-extension:real-server/);
  assert.match(action, /vp run --workspace-root test:helix-extension:real-server/);

  // The scenario needs a Neovim with a Lua LSP client, pinned and checksummed.
  assert.match(action, /NVIM_VERSION: v\d+\.\d+\.\d+/);
  assert.match(action, /NVIM_SHA256: [0-9a-f]{64}/);
  assert.match(action, /sha256sum --check --strict/);
});

test("CI hydrates the exact pinned create-vue revision before the packaged host", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const hydrateAt = action.indexOf("- name: Hydrate pinned create-vue fixture");
  const hostAt = action.indexOf("- name: Run VS Code host smoke against the real server");

  assert.ok(hydrateAt >= 0, "editor host CI must hydrate create-vue");
  assert.ok(hostAt > hydrateAt, "create-vue must be pinned before the host starts");
  const hydration = action.slice(hydrateAt, hostAt);
  assert.match(
    hydration,
    /git submodule update --init --force --depth 1 -- tests\/_fixtures\/_git\/create-vue/,
  );
  assert.match(hydration, /git rev-parse HEAD:tests\/_fixtures\/_git\/create-vue/);
  assert.match(hydration, /git -C tests\/_fixtures\/_git\/create-vue rev-parse HEAD/);
});

test("CI runs both headless editor specs", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");

  assert.match(taskCommand("test:nvim-extension:headless"), /VIZE_TEST_ART_VUE_NATIVE_PARSER=1/);
  assert.match(action, /vp run --workspace-root test:nvim-extension:headless/);
  assert.match(action, /vp run --workspace-root test:vim-extension:headless/);
  assert.match(action, /command -v vim/);
  assert.match(action, /vim --version \| grep -F '\+timers'/);
  assert.match(action, /sudo apt-get install -y vim-nox/);
});

test("CI pins vim-lsp before running the Vim real-server scenario", () => {
  const action = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const installAt = action.indexOf("- name: Install pinned vim-lsp");
  const scenarioAt = action.indexOf("- name: Run the Vim scenario against the real server");

  assert.ok(installAt >= 0, "editor host CI must install vim-lsp");
  assert.ok(scenarioAt > installAt, "vim-lsp must be pinned before the scenario starts");

  const install = action.slice(installAt, scenarioAt);
  assert.match(install, /VIM_LSP_COMMIT: [0-9a-f]{40}/);
  assert.match(
    install,
    /git -C "\$\{RUNNER_TEMP\}\/vim-lsp" fetch --depth 1 origin "\$\{VIM_LSP_COMMIT\}"/,
  );
  assert.match(install, /test "\$\(git rev-parse HEAD\)" = "\$\{VIM_LSP_COMMIT\}"/);

  const scenario = action.slice(scenarioAt);
  assert.match(scenario, /VIZE_TEST_VIM_LSP_PATH: \$\{\{ runner\.temp \}\}\/vim-lsp/);
  assert.match(scenario, /VIZE_SERVER_PATH: \$\{\{ github\.workspace \}\}\/target\/ci\/vize/);
  assert.match(scenario, /vp run --workspace-root test:vim-extension:real-server/);
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
  assert.match(assertion, /"nvim\/test\/ref_surface_hover\.lua"/);
  assert.match(assertion, /\^nvim\\\/test\\\/ref_surface_hover\\\.lua\$/);
  assert.match(assertion, /\^nvim\\\/test\\\/vize_e2e_\(\?:expected\|spec\)\\\.lua\$/);
});

test("the packaged Vim archive ships the real-server scenario", () => {
  const packaged = runVimPackageGuard();
  assert.equal(packaged.status, 0, packaged.output);

  for (const scenarioFile of ["vim/test/vize_e2e_expected.vim", "vim/test/vize_e2e_spec.vim"]) {
    const withoutScenario = runVimPackageGuard([scenarioFile]);
    assert.notEqual(
      withoutScenario.status,
      0,
      `the packaging guard accepted an archive without ${scenarioFile}`,
    );
    assert.match(withoutScenario.output, /missing required file/);
    assert.ok(
      withoutScenario.output.includes(scenarioFile),
      `the packaging guard must name the missing ${scenarioFile}`,
    );
  }
});

test("the Vim real-server scenario pins complete host responses", () => {
  const scenario = readRepoFile("editors", "vim", "test", "vize_e2e_spec.vim");

  for (const method of [
    "textDocument/completion",
    "textDocument/hover",
    "textDocument/codeAction",
    "textDocument/formatting",
    "textDocument/semanticTokens/full",
    "textDocument/rename",
  ]) {
    assert.match(scenario, new RegExp(method.replace("/", "\\/")));
  }
  assert.match(scenario, /assert_equal\(s:expected\.diagnostics, s:diagnostics\(s:uri\)\)/);
  assert.match(scenario, /assert_equal\(a:expected, l:response\.result, a:method\)/);
  assert.doesNotMatch(scenario, /\.includes\(|\.contains\(/);
  assert.doesNotMatch(scenario, /\/dev\/(?:stdout|stderr)/);
});

test("the Neovim real-server scenario pins completion and hover responses", () => {
  const scenario = readRepoFile("editors", "nvim", "test", "vize_e2e_spec.lua");
  const expected = readRepoFile("editors", "nvim", "test", "vize_e2e_expected.lua");

  assert.match(scenario, /textDocument\/completion/);
  assert.match(scenario, /textDocument\/hover/);
  assert.match(scenario, /sorted_matching_labels\(items, expected\.completion_include\)/);
  assert.match(scenario, /sorted_matching_labels\(items, expected\.completion_exclude\)/);
  assert.match(scenario, /assert_eq\(result, expected\.hover, "script binding hover"\)/);
  assert.match(scenario, /ref_surface_hover\.run/);
  assert.doesNotMatch(scenario, /\.includes\(|\.contains\(/);

  assert.match(expected, /completion_include = \{ "Child", "total" \}/);
  assert.match(expected, /completion_exclude = \{ "count", "v-if" \}/);
  assert.match(expected, /const total: "3"/);
  assert.match(expected, /ref_surface_hovers = \{/);
  assert.match(expected, /character = 11, line = 3/);
});

test("the Neovim real-server scenario drains post-save diagnostics before rename", () => {
  const scenario = readRepoFile("editors", "nvim", "test", "vize_e2e_spec.lua");

  // A buffer-wide synchronous request waits for every attached client and can
  // turn a disappearing client into a full timeout. This scenario starts one
  // known Vize client, so requests must target that client directly.
  assert.match(scenario, /client:request_sync\(method, params, 120000, bufnr\)/);
  assert.doesNotMatch(scenario, /vim\.lsp\.buf_request_sync/);

  // `:write` schedules didChange/didSave diagnostics. Do not race a Corsa
  // rename against those passes: wait until Vize publishes the current buffer
  // version, then continue with semantic tokens and rename.
  assert.match(scenario, /counts\[result\.version\] = \(counts\[result\.version\] or 0\) \+ 1/);
  assert.match(scenario, /\(version_counts\[buffer_version\] or 0\) >= 2/);
  assert.match(
    scenario,
    /vim\.cmd\("write"\)[\s\S]*?wait_for_post_save_diagnostics\(bufnr, uri, published_version_counts\)/,
  );

  const waitAt = scenario.indexOf("step_format_on_save(");
  const semanticTokensAt = scenario.indexOf("step_semantic_tokens(", waitAt);
  const renameAt = scenario.indexOf("step_rename(", semanticTokensAt);
  assert.ok(waitAt >= 0 && semanticTokensAt > waitAt && renameAt > semanticTokensAt);
});

test("the VS Code real-server suite drives the packaged type-aware scorecard", () => {
  const scenario = readRepoFile("editors", "vscode", "test", "suite", "real-scenario.cjs");
  const expected = readRepoFile("editors", "vscode", "test", "suite", "real-scenario-expected.cjs");

  assert.match(scenario, /vscode\.executeCodeActionProvider/);
  assert.match(scenario, /vscode\.executeDefinitionProvider/);
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

  assert.match(expected, /templateTag: \[/);
  assert.match(expected, /componentContractTemplateDefinitions/);
  assert.match(expected, /_Component usage_/);
  assert.match(expected, /range: \[7, 3, 7, 16\]/);
});
