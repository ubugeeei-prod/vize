import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { typecheckErrorsApp, CORSA_BIN, VIZE_BIN } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = typecheckErrorsApp;

describe(`${app.name} check (type checker)`, () => {
  before(() => {
    if (!fs.existsSync(VIZE_BIN) || !fs.existsSync(CORSA_BIN)) {
      console.log(`Skipping: vize=${fs.existsSync(VIZE_BIN)}, corsa=${fs.existsSync(CORSA_BIN)}`);
      process.exit(0);
    }
  });

  it("vize check detects type errors", () => {
    const checkConfig = app.check!;
    const patterns = checkConfig.patterns.map((p) => `'${p}'`).join(" ");
    const cmd = `${VIZE_BIN} check ${patterns} --format json --quiet --corsa-path '${CORSA_BIN}'`;
    console.log(`Running: ${cmd}`);

    let stdout: string;
    try {
      stdout = execSync(cmd, {
        cwd: checkConfig.cwd,
        timeout: 120_000,
        maxBuffer: 100 * 1024 * 1024,
      }).toString();
    } catch (e: any) {
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize check crashed (exit code ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    const parsed = JSON.parse(stdout);
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");
    assert.ok(parsed.errorCount > 0, "errorCount should be > 0 (intentional type errors)");

    const prettyOutput =
      JSON.stringify(parsed, null, 2).replaceAll(checkConfig.cwd, "<cwd>") + "\n";
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
