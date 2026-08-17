import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pinnedMoonbitVersion = fs
  .readFileSync(path.join(repoRoot, ".moonbit-version"), "utf8")
  .trim();

/** Stages a `moonc` that reports `version`, the way a restored cache would. */
function writeFakeMoonc(binDir: string, version: string): void {
  writeFakeCommand(
    binDir,
    process.platform === "win32" ? "moonc.exe" : "moonc",
    [`process.stdout.write("v${version} (2026-01-01)\\n");`, "process.exit(0);"].join("\n"),
  );
}

test("runMoonScript does not leak the parent Node test context", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-helper-node-context-"));
  const binDir = path.join(tempDir, "bin");
  const logPath = path.join(tempDir, "node-context.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "moon",
      `require('node:fs').writeFileSync(${JSON.stringify(logPath)}, process.env.NODE_TEST_CONTEXT ?? 'absent');`,
    );
    const moonCommand = path.join(binDir, process.platform === "win32" ? "moon.cmd" : "moon");

    const result = runMoonScript("release", [], {
      cwd: tempDir,
      env: { MOON_BIN: moonCommand, NODE_TEST_CONTEXT: "child-v8" },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.readFileSync(logPath, "utf8"), "absent");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

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

test("setup-moonbit installer reuses a cached toolchain without re-running the bootstrap script", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-installer-cache-"));
  const runnerTemp = path.join(tempDir, "runner-temp");
  const moonBinDir = path.join(runnerTemp, "moonbit", "bin");
  const binDir = path.join(tempDir, "bin");
  const logPath = path.join(tempDir, "moon-invocations.log");
  const bootstrapLogPath = path.join(tempDir, "bootstrap-invocations.log");
  const githubPath = path.join(tempDir, "github-path");
  const githubEnv = path.join(tempDir, "github-env");
  const installerPath = path.join(
    repoRoot,
    ".github",
    "actions",
    "setup-moonbit",
    "install-moonbit.mjs",
  );
  const moonCommandName = process.platform === "win32" ? "moon.exe" : "moon";
  const bootstrapCommandName = process.platform === "win32" ? "pwsh" : "bash";

  try {
    fs.mkdirSync(moonBinDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      moonBinDir,
      moonCommandName,
      [
        "require('node:fs').appendFileSync(",
        `  ${JSON.stringify(logPath)},`,
        "  process.argv.slice(2).join(' ') + '\\n',",
        ");",
        "process.exit(0);",
      ].join("\n"),
    );
    writeFakeMoonc(moonBinDir, pinnedMoonbitVersion);
    writeFakeCommand(
      binDir,
      bootstrapCommandName,
      [
        "require('node:fs').appendFileSync(",
        `  ${JSON.stringify(bootstrapLogPath)},`,
        "  process.argv.slice(2).join(' ') + '\\n',",
        ");",
        "process.exit(97);",
      ].join("\n"),
    );
    writeFileSync(githubPath, "");
    writeFileSync(githubEnv, "");

    const result = spawnSync("node", [installerPath], {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        RUNNER_TEMP: runnerTemp,
        GITHUB_PATH: githubPath,
        GITHUB_ENV: githubEnv,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(fs.existsSync(bootstrapLogPath), false);
    assert.equal(fs.readFileSync(logPath, "utf8").trim(), "run -q --target native - --");
    assert.match(fs.readFileSync(githubEnv, "utf8"), /MOON_HOME=/);
    assert.match(fs.readFileSync(githubEnv, "utf8"), /MOON_BIN=/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("setup-moonbit installer discards a cached toolchain that is not the pinned release", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-installer-stale-"));
  const runnerTemp = path.join(tempDir, "runner-temp");
  const moonBinDir = path.join(runnerTemp, "moonbit", "bin");
  const binDir = path.join(tempDir, "bin");
  const downloadLogPath = path.join(tempDir, "download-invocations.log");
  const githubPath = path.join(tempDir, "github-path");
  const githubEnv = path.join(tempDir, "github-env");
  const installerPath = path.join(
    repoRoot,
    ".github",
    "actions",
    "setup-moonbit",
    "install-moonbit.mjs",
  );
  const moonCommandName = process.platform === "win32" ? "moon.exe" : "moon";
  // Faking the download step keeps the test hermetic: reaching it already
  // proves the stale cache was rejected instead of reused.
  const downloadCommandName = process.platform === "win32" ? "pwsh" : "curl";

  try {
    fs.mkdirSync(moonBinDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(moonBinDir, moonCommandName, "process.exit(0);");
    writeFakeMoonc(moonBinDir, "0.0.0+stalecache");
    writeFakeCommand(
      binDir,
      downloadCommandName,
      [
        "require('node:fs').appendFileSync(",
        `  ${JSON.stringify(downloadLogPath)},`,
        "  process.argv.slice(2).join(' ') + '\\n',",
        ");",
        "process.exit(97);",
      ].join("\n"),
    );
    writeFileSync(githubPath, "");
    writeFileSync(githubEnv, "");

    const result = spawnSync("node", [installerPath], {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        RUNNER_TEMP: runnerTemp,
        GITHUB_PATH: githubPath,
        GITHUB_ENV: githubEnv,
      },
    });

    assert.equal(result.status, 97, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(
      result.stdout,
      new RegExp(
        `Discarding cached MoonBit 0\\.0\\.0\\+stalecache; expected ${pinnedMoonbitVersion.replace(/[.+]/g, "\\$&")}`,
      ),
    );
    assert.equal(fs.existsSync(path.join(moonBinDir, moonCommandName)), false);
    assert.equal(fs.existsSync(downloadLogPath), true);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
