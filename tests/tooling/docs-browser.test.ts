import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

test("docs build uses the lazy browser bootstrap helper", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "docs", "package.json"), "utf8"),
  ) as {
    scripts: Record<string, string>;
  };

  assert.equal(packageJson.scripts.build, "node ./scripts/ensure-browser.mjs && vp build");
});

test("docs browser helper reuses an existing browser path without invoking Playwright install", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "docs-browser-helper-"));
  const binDir = path.join(tempDir, "bin");
  const fakeBrowserPath = path.join(tempDir, "chromium");
  const playwrightLogPath = path.join(tempDir, "playwright.log");
  const helperPath = path.join(repoRoot, "docs", "scripts", "ensure-browser.mjs");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(fakeBrowserPath, "");
    writeFakeCommand(
      binDir,
      "playwright",
      [
        "require('node:fs').appendFileSync(",
        `  ${JSON.stringify(playwrightLogPath)},`,
        "  process.argv.slice(2).join(' ') + '\\n',",
        ");",
        "process.exit(0);",
      ].join("\n"),
    );

    const result = spawnSync("node", [helperPath], {
      cwd: path.join(repoRoot, "docs"),
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        PUPPETEER_EXECUTABLE_PATH: fakeBrowserPath,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, new RegExp(`Using browser at ${fakeBrowserPath}`));
    assert.equal(fs.existsSync(playwrightLogPath), false);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
