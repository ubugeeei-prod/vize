#!/usr/bin/env node
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runVSCodeCommand } from "@vscode/test-electron";

import {
  prepareRealVueWorkspace,
  resolveRealServerPath,
} from "../../../tools/editor-e2e/real-vue-workspace.mjs";
import { runPackagedExtensionHost } from "./packaged-host-contract.mjs";

const sourceExtensionPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const testDataPath = path.join(sourceExtensionPath, ".vscode-test", "host-smoke-real");
const workspacePath = path.join(testDataPath, "workspaces", "real-vue");
const extensionTestsPath = path.join(
  sourceExtensionPath,
  "test",
  "suite",
  "extension-host-real.cjs",
);
const vsixPath = path.join(sourceExtensionPath, "dist", "vize.vsix");

const serverPath = resolveRealServerPath();

fs.rmSync(testDataPath, { force: true, recursive: true });
prepareRealVueWorkspace(workspacePath);

// macOS caps AF_UNIX socket paths at 104 bytes, and VS Code creates its
// singleton socket inside the user-data directory. Deep checkouts overflow the
// default `.vscode-test/user-data` location, so keep user data in a short
// temporary directory instead.
const profilePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-real-"));
const extensionsPath = path.join(profilePath, "extensions");
const userDataPath = path.join(profilePath, "user-data");

try {
  await runPackagedExtensionHost(runVSCodeCommand, {
    extensionId: "ubugeeei.vize",
    extensionsPath,
    extensionTestsPath,
    hostEnvironment: {
      ...process.env,
      VIZE_TEST_PACKAGED_EXTENSIONS_DIR: extensionsPath,
      VIZE_TEST_SERVER_PATH: serverPath,
      VIZE_TEST_SOURCE_EXTENSION_PATH: sourceExtensionPath,
    },
    hostTimeoutMs: 300_000,
    installEnvironment: process.env,
    installTimeoutMs: 120_000,
    onOutput: writeCommandOutput,
    userDataPath,
    vsixPath,
    workspacePath,
  });
} finally {
  fs.rmSync(profilePath, { force: true, recursive: true });
}

function writeCommandOutput({ stderr, stdout }) {
  if (stdout) process.stdout.write(stdout);
  if (stderr) process.stderr.write(stderr);
}
