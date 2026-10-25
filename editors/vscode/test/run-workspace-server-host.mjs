import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runVSCodeCommand } from "@vscode/test-electron";
import { runPackagedExtensionHost } from "./packaged-host-contract.mjs";
import { retainHostOutputBeforeCleanup } from "./host-output.mjs";

const extensionPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceSha = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: extensionPath,
  encoding: "utf8",
}).trim();
if (process.env.VIZE_TEST_SOURCE_SHA) assert.equal(sourceSha, process.env.VIZE_TEST_SOURCE_SHA);
const evidence = path.join(extensionPath, ".vscode-test", "workspace-server");
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ws-"));
const userDataPath = path.join(profile, "user-data");
fs.mkdirSync(path.join(userDataPath, "User"), { recursive: true });
fs.writeFileSync(
  path.join(userDataPath, "User", "settings.json"),
  JSON.stringify({
    "extensions.autoUpdate": false,
    "extensions.autoCheckUpdates": false,
    "update.mode": "none",
  }),
);
const workspacePath = path.join(profile, "workspace 日本語 with spaces");
const version = "0.566.0";
fs.mkdirSync(evidence, { recursive: true });
fs.mkdirSync(path.join(workspacePath, ".vscode"), { recursive: true });
fs.writeFileSync(
  path.join(workspacePath, "package.json"),
  JSON.stringify({ private: true, dependencies: { vize: version, vue: "3.5.43" } }),
);
// npm's Windows command is a .cmd wrapper. Invoke its JS entry with Node so
// spaces and Unicode paths exercise the same shell-free contract as the server.
const npmPath = [
  process.env.npm_execpath,
  path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js"),
  path.resolve(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
].find((candidate) => candidate && fs.existsSync(candidate));
assert.ok(npmPath, "Node distribution must provide the npm CLI");
execFileSync(
  process.execPath,
  [npmPath, "install", "--ignore-scripts", "--no-audit", "--no-fund"],
  { cwd: workspacePath, stdio: "inherit" },
);
const packageRoot = path.join(workspacePath, "node_modules", "vize");
const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
assert.equal(manifest.version, version);
const serverPath = fs.realpathSync(
  path.join(packageRoot, typeof manifest.bin === "string" ? manifest.bin : manifest.bin.vize),
);
assert.match(
  execFileSync(process.execPath, [serverPath, "--version"], {
    encoding: "utf8",
    cwd: workspacePath,
  }),
  /vize 0\.566\.0/,
);
const source =
  '<script setup lang="ts">\nconst message = "workspace engine"\n</script>\n<template>{{ message }}</template>\n';
fs.writeFileSync(path.join(workspacePath, "App.vue"), source);
fs.writeFileSync(
  path.join(workspacePath, ".vscode", "settings.json"),
  JSON.stringify({
    "vize.enable": true,
    "vize.typecheck.enable": false,
    "vize.lint.enable": false,
    "vize.ecosystem.enable": false,
    "vize.server.preferWorkspace": true,
  }),
);
const extensionManifest = JSON.parse(
  fs.readFileSync(path.join(extensionPath, "package.json"), "utf8"),
);
assert.notEqual(extensionManifest.version, version, "acceptance must exercise release skew");
const resultPath = path.join(evidence, "host-result.json");
const vsixPath = path.join(extensionPath, "dist", "vize.vsix");
const bundlePath = path.join(extensionPath, "dist", "extension.cjs");
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const packageIdentityPath = path.join(extensionPath, "dist", "workspace-server-package.json");
const producer = fs.existsSync(packageIdentityPath)
  ? JSON.parse(fs.readFileSync(packageIdentityPath, "utf8"))
  : undefined;
if (producer) {
  assert.equal(producer.sourceSha, sourceSha, "the VSIX producer must match the tested source");
  assert.equal(
    producer.vsixSha256,
    hash(vsixPath),
    "the consumer must install the producer's VSIX",
  );
  assert.equal(producer.bundleSha256, hash(bundlePath), "the producer's bundle bytes must match");
}
fs.copyFileSync(
  path.join(workspacePath, "package-lock.json"),
  path.join(evidence, "workspace-package-lock.json"),
);
try {
  const installed = await runPackagedExtensionHost(runVSCodeCommand, {
    extensionId: "ubugeeei.vize",
    extensionsPath: path.join(profile, "extensions"),
    extensionTestsPath: path.join(extensionPath, "test", "suite", "workspace-server-host.cjs"),
    hostEnvironment: {
      ...process.env,
      VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
      VIZE_TEST_WORKSPACE_SERVER_PATH: serverPath,
      VIZE_TEST_WORKSPACE_SERVER_VERSION: version,
      VIZE_TEST_WORKSPACE_RESULT: resultPath,
      VIZE_TEST_EXTENSION_BUNDLE_SHA256: hash(bundlePath),
    },
    hostTimeoutMs: 120_000,
    installEnvironment: process.env,
    installTimeoutMs: 60_000,
    onOutput: (output) => {
      if (output) process.stdout.write(String(output));
    },
    userDataPath,
    vscodeVersion: "1.107.1",
    vsixPath,
    workspacePath,
  });
  const result = JSON.parse(fs.readFileSync(resultPath, "utf8"));
  assert.equal(path.relative(result.installedExtensionPath, installed), "");
  assert.equal(result.workspace.source, "workspace");
  assert.equal(result.workspace.version, version);
  assert.equal(result.configured.source, "configured");
  const receipt = {
    sourceSha,
    vsixSha256: hash(vsixPath),
    bundleSha256: hash(bundlePath),
    npmVersion: version,
    serverPath,
    result,
  };
  const contents = JSON.stringify(receipt, null, 2) + "\n";
  fs.writeFileSync(path.join(evidence, "artifact.json"), contents);
  if (!producer) fs.writeFileSync(packageIdentityPath, contents);
} finally {
  retainHostOutputBeforeCleanup(profile, path.join(evidence, "output-channel"), true);
}
