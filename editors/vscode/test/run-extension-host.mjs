#!/usr/bin/env node
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-electron";
import { retainHostOutputBeforeCleanup } from "./host-output.mjs";

const extensionDevelopmentPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixtureWorkspacePath = path.join(
  extensionDevelopmentPath,
  "test-fixtures",
  "extension-host",
  "basic-vue",
);
const testDataPath = path.join(extensionDevelopmentPath, ".vscode-test", "host-smoke");
const workspacePath = path.join(testDataPath, "workspaces", "basic-vue");
const fakeServerDir = path.join(testDataPath, "fake-server");
const fakeServerPath = path.join(fakeServerDir, process.platform === "win32" ? "vize.cmd" : "vize");
const fakeServerLogPath = path.join(fakeServerDir, "events.jsonl");
const fakeServerScriptPath = path.join(
  extensionDevelopmentPath,
  "test",
  "fixtures",
  "fake-vize-server.cjs",
);
const extensionTestsPath = path.join(
  extensionDevelopmentPath,
  "test",
  "suite",
  "extension-host.cjs",
);
const packageJson = JSON.parse(
  fs.readFileSync(path.join(extensionDevelopmentPath, "package.json"), "utf-8"),
);

fs.rmSync(testDataPath, { force: true, recursive: true });
fs.mkdirSync(fakeServerDir, { recursive: true });
fs.cpSync(fixtureWorkspacePath, workspacePath, { recursive: true });
// Existing-project config cases create their files after activation. Prepare
// only the empty parent before VS Code's recursive watcher takes its snapshot.
fs.mkdirSync(path.join(workspacePath, "packages", "app"), { recursive: true });
fs.writeFileSync(fakeServerLogPath, "");

if (process.platform === "win32") {
  fs.writeFileSync(
    fakeServerPath,
    `@echo off\r\n"${process.execPath}" "${fakeServerScriptPath}" %*\r\n`,
  );
} else {
  fs.writeFileSync(
    fakeServerPath,
    `#!/bin/sh\nexec ${JSON.stringify(process.execPath)} ${JSON.stringify(fakeServerScriptPath)} "$@"\n`,
    { mode: 0o755 },
  );
}

const profilePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-"));
try {
  await runTests({
    version: process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1",
    extensionDevelopmentPath,
    extensionTestsPath,
    extensionTestsEnv: {
      VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
      VIZE_TEST_SERVER_LOG: fakeServerLogPath,
      VIZE_TEST_SERVER_PATH: fakeServerPath,
      VIZE_TEST_SERVER_VERSION: packageJson.version,
    },
    launchArgs: [
      `--extensions-dir=${path.join(profilePath, "extensions")}`,
      `--user-data-dir=${path.join(profilePath, "user-data")}`,
      "--disable-extensions",
      "--disable-workspace-trust",
      "--skip-welcome",
      "--skip-release-notes",
      workspacePath,
    ],
  });
} catch (error) {
  const destination = path.join(testDataPath, "failure");
  fs.mkdirSync(destination, { recursive: true });
  fs.copyFileSync(fakeServerLogPath, path.join(destination, "events.jsonl"));
  fs.writeFileSync(
    path.join(destination, "error.json"),
    JSON.stringify({ message: String(error), stack: error?.stack }, null, 2) + "\n",
  );
  fs.cpSync(workspacePath, path.join(destination, "workspace"), {
    recursive: true,
    filter: (input) => !input.split(path.sep).includes("node_modules"),
  });
  throw error;
} finally {
  retainHostOutputBeforeCleanup(profilePath, path.join(testDataPath, "output-channel"));
}
