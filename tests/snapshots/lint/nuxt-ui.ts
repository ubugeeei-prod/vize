import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { nuxtUiApp, VIZE_BIN } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = nuxtUiApp;

interface LintFileResult {
  file: string;
  errorCount: number;
  warningCount: number;
}

describe(`${app.name} lint (linter)`, () => {
  before(() => {
    if (!fs.existsSync(VIZE_BIN)) {
      console.log(`Skipping: vize binary not found at ${VIZE_BIN}`);
      process.exit(0);
    }
  });

  it("vize lint does not crash and snapshot matches", () => {
    const lintConfig = app.lint!;
    const patterns = lintConfig.patterns.map((p) => `'${p}'`).join(" ");
    const cmd = `${VIZE_BIN} lint ${patterns} --format json --quiet`;
    console.log(`Running: ${cmd}`);

    let stdout: string;
    try {
      stdout = execSync(cmd, {
        cwd: lintConfig.cwd,
        timeout: 120_000,
        maxBuffer: 100 * 1024 * 1024,
      }).toString();
    } catch (e: any) {
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize lint crashed (exit code ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    // Re-parse and pretty-print for stable snapshot
    const parsed = JSON.parse(stdout) as LintFileResult[];
    assert.ok(Array.isArray(parsed) && parsed.length > 0, "lint should produce results");
    const prettyOutput = JSON.stringify(parsed, null, 2).replaceAll(lintConfig.cwd, "<cwd>") + "\n";

    console.log(`fileCount=${parsed.length}`);
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-lint`, prettyOutput);
  });
});
