// Actual Vim + pinned vim-lsp config integration; every request reaches vize lsp.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { repositoryRoot, resolveRealServerPath } from "../editor-e2e/real-vue-workspace.mjs";

const pinnedClient = "474c656659b5fb51eec6770309e0211c8aa49b5b";
const client = process.env.VIZE_TEST_VIM_LSP_PATH;
assert.ok(client, "VIZE_TEST_VIM_LSP_PATH must identify the pinned real vim-lsp client");
assert.equal(
  execFileSync("git", ["-C", client, "rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  pinnedClient,
);
const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: repositoryRoot,
  encoding: "utf8",
}).trim();
if (process.env.GITHUB_SHA) assert.equal(sourceSha, process.env.GITHUB_SHA);
const binary = fs.realpathSync(resolveRealServerPath());
const digest = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const binarySha = digest(binary);
const vim = process.env.VIZE_TEST_VIM_PATH || "vim";
const vimVersion = execFileSync(vim, ["--version"], { encoding: "utf8" });
assert.match(vimVersion, /^VIM - Vi IMproved /, "this acceptance lane requires actual Vim");
const output = process.env.VIZE_VIM_ARTIFACT_DIR
  ? path.resolve(process.env.VIZE_VIM_ARTIFACT_DIR)
  : fs.mkdtempSync(path.join(os.tmpdir(), "vize-vim-config-"));
fs.mkdirSync(output, { recursive: true });
const transcript = path.join(output, "transcript.jsonl");
assert.ok(!fs.existsSync(transcript), "configuration acceptance needs a fresh transcript");
const workspace = path.join(output, "workspace");
fs.mkdirSync(workspace);
const pidFile = path.join(output, "native.pid");
const wrapper = path.join(output, "native-server.sh");
const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;
fs.writeFileSync(
  wrapper,
  `#!/bin/sh\nexec 3<&0\n${quote(binary)} "$@" <&3 &\nnative=$!\nexec 3<&-\nprintf '%s\\n' "$native" > ${quote(pidFile)}\nwait "$native"\n`,
  { mode: 0o755 },
);
const result = spawnSync(
  vim,
  [
    "-Nu",
    "NONE",
    "-n",
    "-es",
    "-i",
    "NONE",
    `-V1${path.join(output, "vim-verbose.log")}`,
    "-S",
    path.join(repositoryRoot, "editors/vim/test/config_project_spec.vim"),
  ],
  {
    cwd: workspace,
    encoding: "utf8",
    timeout: 180_000,
    killSignal: "SIGKILL",
    env: {
      ...process.env,
      VIZE_E2E_PLUGIN_ROOT: path.join(repositoryRoot, "editors/vim"),
      VIZE_E2E_WORKSPACE: workspace,
      VIZE_E2E_SERVER: wrapper,
      VIZE_E2E_NODE: process.execPath,
      VIZE_E2E_TAP: path.join(repositoryRoot, "tests/editor-conformance/lsp-tap.mjs"),
      VIZE_E2E_ERROR_PATH: path.join(output, "vim-errors.log"),
      VIZE_CONFORMANCE_TRANSCRIPT: transcript,
    },
  },
);
fs.writeFileSync(path.join(output, "stdout.log"), result.stdout ?? "");
fs.writeFileSync(path.join(output, "stderr.log"), result.stderr ?? "");
const nativePid = fs.existsSync(pidFile) ? Number(fs.readFileSync(pidFile, "utf8").trim()) : null;
const receipt = {
  passed:
    result.status === 0 &&
    !result.error &&
    Number.isSafeInteger(nativePid) &&
    nativePid! > 0 &&
    digest(binary) === binarySha,
  sourceSha,
  client: { path: fs.realpathSync(client), commit: pinnedClient },
  binary: {
    path: binary,
    sha256: binarySha,
    nativePid,
  },
  host: {
    pid: result.pid,
    exitCode: result.status,
    signal: result.signal,
    error: result.error?.message,
    version: vimVersion,
  },
  transcript: { path: transcript, sha256: fs.existsSync(transcript) ? digest(transcript) : null },
  cases: [
    "canonical-vite-features",
    "same-directory-tsconfig-references",
    "saved-helper-last-valid-recovery",
    "unaffected-package",
    "inactive-and-outside-workspace",
  ],
  watchBoundary:
    "Vim buffer saves notify Vize; this vim-lsp client has no dynamic external-file watching.",
};
fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(`Vim configuration evidence: ${output}`);
if (!receipt.passed) {
  for (const name of ["stdout.log", "stderr.log", "vim-errors.log", "vim-verbose.log"]) {
    const file = path.join(output, name);
    if (fs.existsSync(file)) process.stderr.write(fs.readFileSync(file));
  }
  throw result.error ?? new Error(`Vim configuration scenario exited ${result.status}`);
}
assert.ok(Number.isSafeInteger(receipt.binary.nativePid) && receipt.binary.nativePid! > 0);
console.log("Vim configuration scenario passed");
