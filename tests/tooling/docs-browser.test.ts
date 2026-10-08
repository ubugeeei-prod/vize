import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { writeFakeCommand } from "./support/fake-command.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("docs build prepares its browser before previews and checks the rendered site", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "docs", "package.json"), "utf8"),
  ) as {
    scripts: Record<string, string>;
  };

  assert.deepEqual(packageJson.scripts.build.split(" && "), [
    "node ./scripts/ensure-browser.ts",
    "pnpm generate:ui-previews",
    "pnpm generate:reference",
    "vp build",
    "pnpm check:ui-docs",
  ]);
  assert.equal(packageJson.scripts["generate:ui-previews"], "node ./scripts/build-ui-previews.ts");
  assert.equal(
    packageJson.scripts["generate:reference"],
    "node ../npm/ui/scripts/generate-reference-docs.ts",
  );
  assert.equal(packageJson.scripts["check:ui-docs"], "node ./previews/ui/check-site.ts");
});

test("docs browser helper reuses an existing browser path without invoking Playwright install", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "docs-browser-helper-"));
  const binDir = path.join(tempDir, "bin");
  const fakeBrowserPath = path.join(tempDir, "chromium");
  const playwrightLogPath = path.join(tempDir, "playwright.log");
  const helperPath = path.join(repoRoot, "docs", "scripts", "ensure-browser.ts");

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
