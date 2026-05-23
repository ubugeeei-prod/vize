#!/usr/bin/env node
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-electron";

const extensionDevelopmentPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspacePath = path.join(
  extensionDevelopmentPath,
  "test-fixtures",
  "extension-host",
  "basic-vue",
);
const extensionTestsPath = path.join(
  extensionDevelopmentPath,
  "test",
  "suite",
  "extension-host.cjs",
);

await runTests({
  extensionDevelopmentPath,
  extensionTestsPath,
  launchArgs: [
    "--disable-extensions",
    "--disable-workspace-trust",
    "--skip-welcome",
    "--skip-release-notes",
    workspacePath,
  ],
});
