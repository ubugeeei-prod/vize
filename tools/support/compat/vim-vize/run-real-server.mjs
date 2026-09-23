// Drive the packaged Vim integration through a pinned vim-lsp checkout and a
// real `vize lsp` process. All response assertions live in the Vim archive's
// own test directory so downstream users can run the same host scenario.
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  prepareRealVueWorkspace,
  repositoryRoot,
  resolveRealServerPath,
} from "../editor-e2e/real-vue-workspace.mjs";

const vimPath = process.env.VIZE_TEST_VIM_PATH?.trim() || "vim";
const vimLspPath = process.env.VIZE_TEST_VIM_LSP_PATH?.trim();
if (!vimLspPath) {
  throw new Error(
    "VIZE_TEST_VIM_LSP_PATH must point at the pinned vim-lsp checkout used by the host test",
  );
}

const vimLspPlugin = path.join(vimLspPath, "plugin", "lsp.vim");
if (!fs.existsSync(vimLspPlugin)) {
  throw new Error(`vim-lsp plugin not found: ${vimLspPlugin}`);
}

const pluginRoot = path.join(repositoryRoot, "editors", "vim");
const specPath = path.join(pluginRoot, "test", "vize_e2e_spec.vim");
const serverPath = resolveRealServerPath();
const sessionPath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vim-e2e-"));
const workspacePath = path.join(sessionPath, "real-vue");
const errorPath = path.join(sessionPath, "vim-errors.log");
const verbosePath = path.join(sessionPath, "vim-verbose.log");

prepareRealVueWorkspace(workspacePath);

try {
  const result = spawnSync(
    vimPath,
    ["-Nu", "NONE", "-n", "-es", "-i", "NONE", `-V1${verbosePath}`, "-S", specPath],
    {
      cwd: workspacePath,
      encoding: "utf-8",
      env: {
        ...process.env,
        VIZE_E2E_ERROR_PATH: errorPath,
        VIZE_E2E_PLUGIN_ROOT: pluginRoot,
        VIZE_E2E_SERVER: serverPath,
        VIZE_E2E_WORKSPACE: workspacePath,
      },
      timeout: 1_200_000,
    },
  );

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    for (const diagnosticPath of [errorPath, verbosePath]) {
      if (fs.existsSync(diagnosticPath)) {
        process.stderr.write(fs.readFileSync(diagnosticPath, "utf-8"));
      }
    }
    throw new Error(`headless Vim scenario failed with exit code ${result.status}`);
  }
  console.log("vim real-server scenario passed");
} finally {
  fs.rmSync(sessionPath, { force: true, recursive: true });
}
