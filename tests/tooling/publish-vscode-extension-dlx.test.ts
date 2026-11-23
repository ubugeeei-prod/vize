import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("publish_vscode_extension allows corepack pnpm dlx builds for vsce signing dependencies", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-corepack-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "corepack-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    // The gallery probe never lists the version, so only publish reaches the runner.
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram("commands/release-command-diagnostics/curl.js"),
    );
    writeFakeCommand(
      binDir,
      "corepack",
      readToolingProgram("commands/publish-vscode-extension-dlx/corepack.js"),
    );

    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        NPM_TAG: "rc",
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        COREPACK_ARGS_LOG: argsLogPath,
        VSCE_DLX_BIN: "corepack",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
      "pnpm",
      "dlx",
      "--allow-build",
      "@vscode/vsce-sign",
      "--allow-build",
      "keytar",
      "--package",
      "@vscode/vsce@^3.3.2",
      "vsce",
      "publish",
      "--no-dependencies",
      "--packagePath",
      vsixPath,
      "--pre-release",
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_vscode_extension can publish with npx without pnpm hydration", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-npx-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "npx-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.58.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram("commands/release-command-diagnostics/curl.js"),
    );
    writeFakeCommand(
      binDir,
      "npx",
      readToolingProgram("commands/publish-vscode-extension-dlx/npx.js"),
    );

    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        NPM_TAG: "rc",
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        NPX_ARGS_LOG: argsLogPath,
        VSCE_DLX_BIN: "npx",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
      "-y",
      "-p",
      "@vscode/vsce@^3.3.2",
      "vsce",
      "publish",
      "--no-dependencies",
      "--packagePath",
      vsixPath,
      "--pre-release",
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_vscode_extension finishes on a successful publish without waiting for the listing", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-success-"));
  const binDir = path.join(tempDir, "bin");
  const logPath = path.join(tempDir, "calls.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");
  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.58.0" }),
    );
    // The Marketplace never lists the version; publish itself succeeds.
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram(
        "commands/publish-vscode-extension-dlx/curl-finishes-successful-without-waiting-listing.js",
      ),
    );
    writeFakeCommand(
      binDir,
      "vp",
      readToolingProgram("commands/publish-vscode-extension-dlx/vp.js"),
    );
    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CALLS_LOG: logPath,
        PUBLISH_RESOLUTION_RETRY_LIMIT: "3",
        PUBLISH_RESOLUTION_RETRY_DELAY: "0",
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(logPath, "utf8").trim().split("\n"), ["curl", "publish"]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
