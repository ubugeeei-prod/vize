import { spawnSync } from "node:child_process";
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

function main(): number {
  const scenario = createScenarioWorkspace("vize-nvim-e2e");
  try {
    const pluginRoot = path.join(repositoryRoot, "editors", "nvim");
    const specPath = path.join(pluginRoot, "test", "vize_e2e_spec.lua");
    const result = spawnSync(
      process.env.VIZE_TEST_NVIM_PATH ?? "nvim",
      [
        "--headless",
        "-u",
        "NONE",
        "--noplugin",
        "-n",
        "-i",
        "NONE",
        "-c",
        `lua vim.opt.runtimepath:prepend(${JSON.stringify(pluginRoot)})`,
        "-c",
        `lua dofile(${JSON.stringify(specPath)})`,
        "-c",
        "qall!",
      ],
      {
        cwd: scenario.workspace,
        encoding: "utf8",
        env: {
          ...process.env,
          VIZE_E2E_SERVER: resolveRealServerPath(),
          VIZE_E2E_WORKSPACE: scenario.workspace,
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    if (result.error !== undefined)
      throw new Error(`failed to run headless Neovim: ${result.error}`);
    if (result.status !== 0)
      throw new Error(`headless Neovim scenario failed with exit code ${result.status ?? 1}`);
    return 0;
  } finally {
    removeScenarioWorkspace(scenario);
  }
}

runEntrypoint(main);
