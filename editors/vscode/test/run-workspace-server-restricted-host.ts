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

const evidence = path.join(extensionPath, ".vscode-test", "workspace-server-restricted");
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ws-restricted-"));
const userDataPath = path.join(profile, "user-data");
const workspacePath = path.join(profile, "untrusted workspace 日本語");
const safeServerPath = path.join(extensionPath, "test", "fixtures", "fake-vize-server.cjs");
const safeServerLog = path.join(evidence, "safe-server.jsonl");
const forbiddenMarker = path.join(evidence, "forbidden-workspace-server.jsonl");
const resultPath = path.join(evidence, "host-result.json");
const vsixPath = path.join(extensionPath, "dist", "vize.vsix");
const bundlePath = path.join(extensionPath, "dist", "extension.cjs");
const extensionManifest = JSON.parse(
  fs.readFileSync(path.join(extensionPath, "package.json"), "utf8"),
);
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");

fs.mkdirSync(evidence, { recursive: true });
for (const file of [safeServerLog, forbiddenMarker, resultPath]) fs.rmSync(file, { force: true });
fs.mkdirSync(path.join(userDataPath, "User"), { recursive: true });
fs.writeFileSync(
  path.join(userDataPath, "User", "settings.json"),
  JSON.stringify({
    "extensions.autoUpdate": false,
    "extensions.autoCheckUpdates": false,
    "security.workspace.trust.enabled": true,
    "security.workspace.trust.startupPrompt": "never",
    "update.mode": "none",
    "vize.serverPath": safeServerPath,
  }),
);

const packageRoot = path.join(workspacePath, "node_modules", "vize");
const forbiddenServerPath = path.join(packageRoot, "bin", "vize.cjs");
fs.mkdirSync(path.dirname(forbiddenServerPath), { recursive: true });
fs.mkdirSync(path.join(workspacePath, ".vscode"), { recursive: true });
fs.writeFileSync(path.join(workspacePath, "package.json"), JSON.stringify({ private: true }));
fs.writeFileSync(
  path.join(packageRoot, "package.json"),
  JSON.stringify({ name: "vize", version: "9.9.9", bin: { vize: "bin/vize.cjs" } }),
);
fs.writeFileSync(
  forbiddenServerPath,
  [
    "#!/usr/bin/env node",
    'const fs = require("node:fs");',
    'fs.appendFileSync(process.env.VIZE_TEST_FORBIDDEN_SERVER_MARKER, JSON.stringify({ argv: process.argv.slice(2) }) + "\\n");',
    'if (process.argv.includes("--version")) console.log("vize 9.9.9");',
  ].join("\n") + "\n",
  { mode: 0o755 },
);
fs.writeFileSync(
  path.join(workspacePath, ".vscode", "settings.json"),
  JSON.stringify({
    "vize.enable": true,
    "vize.typecheck.enable": false,
    "vize.lint.enable": false,
    "vize.ecosystem.enable": false,
    "vize.server.preferWorkspace": true,
    "vize.serverPath": forbiddenServerPath,
  }),
);
fs.writeFileSync(
  path.join(workspacePath, "App.vue"),
  '<script setup lang="ts">\nconst message = "restricted"\n</script>\n<template>{{ message }}</template>\n',
);

try {
  const installed = await runPackagedExtensionHost(runVSCodeCommand, {
    extensionId: "ubugeeei.vize",
    extensionsPath: path.join(profile, "extensions"),
    extensionTestsPath: path.join(
      extensionPath,
      "test",
      "suite",
      "workspace-server-restricted-host.cjs",
    ),
    hostEnvironment: {
      ...process.env,
      VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
      VIZE_TEST_EXTENSION_BUNDLE_SHA256: hash(bundlePath),
      VIZE_TEST_FORBIDDEN_SERVER_MARKER: forbiddenMarker,
      VIZE_TEST_RESTRICTED_RESULT: resultPath,
      VIZE_TEST_SAFE_SERVER_LOG: safeServerLog,
      VIZE_TEST_SAFE_SERVER_PATH: safeServerPath,
      VIZE_TEST_SERVER_LOG: safeServerLog,
      VIZE_TEST_SERVER_VERSION: extensionManifest.version,
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
    workspaceTrusted: false,
  });
  const result = JSON.parse(fs.readFileSync(resultPath, "utf8"));
  assert.equal(path.relative(result.installedExtensionPath, installed), "");
  assert.equal(result.server.source, "configured");
  assert.equal(
    result.server.workspaceFallbackReason,
    "Workspace servers skipped in Restricted Mode.",
  );
  assert.equal(fs.existsSync(forbiddenMarker), false);
  fs.writeFileSync(
    path.join(evidence, "artifact.json"),
    JSON.stringify(
      {
        sourceSha,
        vsixSha256: hash(vsixPath),
        bundleSha256: hash(bundlePath),
        forbiddenServerPath,
        safeServerPath,
        result,
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  retainHostOutputBeforeCleanup(profile, path.join(evidence, "output-channel"), true);
}
