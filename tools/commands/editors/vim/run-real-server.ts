import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { runEntrypoint } from "../../../support/command-runtime.ts";
import {
  createScenarioWorkspace,
  removeScenarioWorkspace,
} from "../../../support/compat/editor-e2e/scenario-workspace.ts";
import {
  repositoryRoot,
  resolveRealServerPath,
} from "../../../support/compat/editor-e2e/real-vue-workspace.ts";

function appendFailureLog(filename: string): void {
  if (fs.existsSync(filename)) process.stderr.write(fs.readFileSync(filename, "utf8"));
}

function main(): number {
  const vimLsp = process.env.VIZE_TEST_VIM_LSP_PATH;
  if (!vimLsp)
    throw new Error(
      "VIZE_TEST_VIM_LSP_PATH must point at the pinned vim-lsp checkout used by the host test",
    );
  const vimLspPlugin = path.join(vimLsp, "plugin", "lsp.vim");
  if (!fs.existsSync(vimLspPlugin)) throw new Error(`vim-lsp plugin not found: ${vimLspPlugin}`);

  const scenario = createScenarioWorkspace("vize-vim-e2e");
  const errorPath = path.join(scenario.root, "vim-errors.log");
  const verbosePath = path.join(scenario.root, "vim-verbose.log");
  const serverPath = resolveRealServerPath();
  try {
    const pluginRoot = path.join(repositoryRoot, "editors", "vim");
    const result = spawnSync(
      process.env.VIZE_TEST_VIM_PATH ?? "vim",
      [
        "-Nu",
        "NONE",
        "-n",
        "-es",
        "-i",
        "NONE",
        `-V1${verbosePath}`,
        "-S",
        path.join(pluginRoot, "test", "vize_e2e_spec.vim"),
      ],
      {
        cwd: scenario.workspace,
        encoding: "utf8",
        env: {
          ...process.env,
          VIZE_E2E_ERROR_PATH: errorPath,
          VIZE_E2E_PLUGIN_ROOT: pluginRoot,
          VIZE_E2E_SERVER: serverPath,
          VIZE_E2E_WORKSPACE: scenario.workspace,
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    if (result.error !== undefined) throw new Error(`failed to run headless Vim: ${result.error}`);
    if (result.status !== 0) {
      appendFailureLog(errorPath);
      appendFailureLog(verbosePath);
      throw new Error(`headless Vim scenario failed with exit code ${result.status ?? 1}`);
    }
    console.log("vim real-server scenario passed");
  } finally {
    removeScenarioWorkspace(scenario);
  }

  const configuration = spawnSync(
    process.execPath,
    [path.join(repositoryRoot, "tools", "support", "compat", "vim-vize", "run-config-server.ts")],
    {
      cwd: repositoryRoot,
      env: { ...process.env, VIZE_SERVER_PATH: serverPath },
      stdio: ["ignore", "inherit", "inherit"],
    },
  );
  if (configuration.error !== undefined)
    throw new Error(`failed to run Vim configuration scenario: ${configuration.error}`);
  if (configuration.status !== 0) throw new Error("Vim configuration scenario failed");
  return 0;
}

runEntrypoint(main);
