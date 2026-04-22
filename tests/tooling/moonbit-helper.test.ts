import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";

function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

test("runMoonScript falls back to the GitHub runner shim when MOON_BIN is unavailable", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-helper-"));
  const runnerTemp = path.join(tempDir, "runner-temp");
  const shimDir = path.join(runnerTemp, "moonbit-shims");
  const binDir = path.join(tempDir, "bin");
  const logPath = path.join(tempDir, "moon-command.log");
  const originalMoonBin = process.env.MOON_BIN;

  try {
    fs.mkdirSync(shimDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      shimDir,
      "moon",
      [
        "require('node:fs').writeFileSync(process.env.MOON_HELPER_LOG, 'runner-shim');",
        "process.exit(0);",
      ].join("\n"),
    );
    writeFakeCommand(binDir, "moon", "process.exit(99);");

    delete process.env.MOON_BIN;

    const result = runMoonScript("github/install_playwright_browsers", [], {
      cwd: tempDir,
      env: {
        MOON_HELPER_LOG: logPath,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        RUNNER_TEMP: runnerTemp,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(logPath, "utf8"), "runner-shim");
  } finally {
    if (originalMoonBin === undefined) {
      delete process.env.MOON_BIN;
    } else {
      process.env.MOON_BIN = originalMoonBin;
    }
    rmSync(tempDir, { recursive: true, force: true });
  }
});
