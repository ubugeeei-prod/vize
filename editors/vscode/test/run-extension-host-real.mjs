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
import {
  createVueTypecheckAppPath,
  materializeCreateVueTypecheckSource,
} from "../../../tests/_helpers/create-vue-typecheck-patch.ts";
import { withPinnedFixtureWorkspace } from "../../../tests/_helpers/realworld-patch.ts";
import { createRealHostEnvironment, runPackagedExtensionHost } from "./packaged-host-contract.mjs";
import { readPinnedCreateVueHostResult } from "./pinned-create-vue-host-result.mjs";

const sourceExtensionPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const testDataPath = path.join(sourceExtensionPath, ".vscode-test", "host-smoke-real");
const extensionTestsPath = path.join(
  sourceExtensionPath,
  "test",
  "suite",
  "extension-host-real.cjs",
);
const hostTimeoutMs = 600_000;
const vsixPath = path.join(sourceExtensionPath, "dist", "vize.vsix");
const vscodeVersion = process.env.VIZE_TEST_VSCODE_VERSION ?? "1.107.1";

const serverPath = resolveRealServerPath();

fs.rmSync(testDataPath, { force: true, recursive: true });
await withPinnedFixtureWorkspace(
  {
    fixtureId: "create-vue",
    includePaths: [createVueTypecheckAppPath],
    outsideRepository: true,
  },
  async (fixture) => {
    materializeCreateVueTypecheckSource(fixture);
    prepareRealVueWorkspace(fixture.workspaceDir, { preserveExisting: true });
    fixture.write(
      "tsconfig.json",
      `${JSON.stringify(
        {
          compilerOptions: {
            allowImportingTsExtensions: true,
            module: "ESNext",
            moduleResolution: "bundler",
            noEmit: true,
            strict: true,
            target: "ES2022",
          },
          include: ["src/**/*", "template/bare/typescript/src/**/*.vue"],
        },
        null,
        2,
      )}\n`,
    );

    // macOS caps AF_UNIX socket paths at 104 bytes, and VS Code creates its
    // singleton socket inside the user-data directory. Deep checkouts overflow
    // the default location, so keep user data in a short temporary directory.
    const profilePath = fs.mkdtempSync(path.join(os.tmpdir(), "vize-host-real-"));
    const extensionsPath = path.join(profilePath, "extensions");
    const resultPath = path.join(profilePath, "pinned-create-vue-host-result.json");
    const userDataPath = path.join(profilePath, "user-data");

    try {
      await runPackagedExtensionHost(runVSCodeCommand, {
        extensionId: "ubugeeei.vize",
        extensionsPath,
        extensionTestsPath,
        hostEnvironment: createRealHostEnvironment({
          extensionsPath,
          processEnvironment: process.env,
          resultPath,
          serverPath,
          sourceExtensionPath,
        }),
        hostTimeoutMs,
        installEnvironment: process.env,
        installTimeoutMs: 120_000,
        onOutput: writeCommandOutput,
        userDataPath,
        vscodeVersion,
        vsixPath,
        workspacePath: fixture.workspaceDir,
      });
      readPinnedCreateVueHostResult(resultPath);
    } finally {
      fs.rmSync(profilePath, { force: true, recursive: true });
    }
  },
);

function writeCommandOutput({ stderr, stdout }) {
  if (stdout) process.stdout.write(stdout);
  if (stderr) process.stderr.write(stderr);
}
