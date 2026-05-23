#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-electron";

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

await runTests({
  extensionDevelopmentPath,
  extensionTestsPath,
  extensionTestsEnv: {
    VIZE_TEST_SERVER_LOG: fakeServerLogPath,
    VIZE_TEST_SERVER_PATH: fakeServerPath,
    VIZE_TEST_SERVER_VERSION: packageJson.version,
  },
  launchArgs: [
    "--disable-extensions",
    "--disable-workspace-trust",
    "--skip-welcome",
    "--skip-release-notes",
    workspacePath,
  ],
});
