import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("publish_vscode_extension allows pnpm dlx builds for vsce signing dependencies", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-vsix-pnpm-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "pnpm-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "pnpm",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "if (args[0] === 'dlx' && args[7] === 'vsce' && args[8] === 'show') {",
        "  if (fs.existsSync(process.env.PNPM_ARGS_LOG)) {",
        "    process.stdout.write(JSON.stringify({ versions: [{ version: '0.57.0' }] }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "fs.writeFileSync(process.env.PNPM_ARGS_LOG, args.join('\\n'));",
        "process.exit(0);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        NPM_TAG: "rc",
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        PNPM_ARGS_LOG: argsLogPath,
        VSCE_DLX_BIN: "pnpm",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
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
