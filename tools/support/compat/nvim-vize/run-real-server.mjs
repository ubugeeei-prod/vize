// Drive the packaged Neovim integration against a real `vize lsp` process
// (#3457). Neovim is the only non-VS Code editor in the packaged set with a
// scriptable headless LSP client, so it carries the second end-to-end scenario
// for the #3224 parity scorecard.
//
// This runner only prepares the workspace and launches headless Neovim; every
// assertion lives in `editors/nvim/test/vize_e2e_spec.lua`, which ships inside
// the Neovim tarball so a packaged checkout can run the same scenario.
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  prepareRealVueWorkspace,
  repositoryRoot,
  resolveRealServerPath,
} from "../editor-e2e/real-vue-workspace.mjs";

const nvimPath = process.env.VIZE_TEST_NVIM_PATH?.trim() || "nvim";
const pluginRoot = path.join(repositoryRoot, "editors", "nvim");
const specPath = path.join(pluginRoot, "test", "vize_e2e_spec.lua");
const serverPath = resolveRealServerPath();
const sessionPath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nvim-e2e-"));
const workspacePath = path.join(sessionPath, "real-vue");

prepareRealVueWorkspace(workspacePath);

try {
  const result = spawnSync(
    nvimPath,
    [
      "--headless",
      "-u",
      "NONE",
      "--noplugin",
      "-n",
      "-i",
      "NONE",
      // Paths can contain spaces, backslashes or `|`, which ex commands would
      // mangle, so hand them to Lua as literals instead. The spec prepends the
      // plugin root itself, but do it here too so `require("vize.…")` works even
      // if the spec is ever loaded differently.
      "-c",
      `lua vim.opt.runtimepath:prepend(${JSON.stringify(pluginRoot)})`,
      "-c",
      `lua dofile(${JSON.stringify(specPath)})`,
      "-c",
      "qall!",
    ],
    {
      cwd: workspacePath,
      encoding: "utf-8",
      env: {
        ...process.env,
        VIZE_E2E_SERVER: serverPath,
        VIZE_E2E_WORKSPACE: workspacePath,
      },
      // The spec's own waits already add up to 960s in the worst case (120s
      // initialize + 240s diagnostics + 120s for each of the five requests and
      // the synchronous format), so this outer kill switch has to sit above
      // that or it would pre-empt the assertions it is meant to bound.
      timeout: 1_200_000,
    },
  );

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`headless Neovim scenario failed with exit code ${result.status}`);
  }
} finally {
  fs.rmSync(sessionPath, { force: true, recursive: true });
}
