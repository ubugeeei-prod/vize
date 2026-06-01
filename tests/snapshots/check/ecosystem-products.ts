import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import {
  ecosystemProductsApp,
  CORSA_BIN,
  VIZE_BIN,
  requireVizeAndCorsaBins,
} from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = ecosystemProductsApp;

function normalizeEnvironmentDependentDiagnostics(parsed: any): any {
  const files = parsed.files.map((file: any) => ({
    ...file,
    diagnostics: file.diagnostics.filter((diagnostic: string) => {
      if (/^hint:.* \[TS638[57]\] /.test(diagnostic)) {
        return false;
      }
      return !/^error:.* \[TS2882\] Cannot find module or type declarations for side-effect import/.test(
        diagnostic,
      );
    }),
  }));

  return {
    ...parsed,
    files,
    errorCount: files.reduce((count: number, file: any) => {
      return (
        count +
        file.diagnostics.filter((diagnostic: string) => diagnostic.startsWith("error:")).length
      );
    }, 0),
    warningCount: files.reduce((count: number, file: any) => {
      return (
        count +
        file.diagnostics.filter((diagnostic: string) => diagnostic.startsWith("warning:")).length
      );
    }, 0),
  };
}

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check does not crash and snapshot matches", () => {
    const checkConfig = app.check!;
    const patterns = checkConfig.patterns.map((p) => `'${p}'`).join(" ");
    const cmd = `${VIZE_BIN} check ${patterns} --format json --quiet --corsa-path '${CORSA_BIN}'`;
    console.log(`Running: ${cmd}`);

    let stdout: string;
    try {
      stdout = execSync(cmd, {
        cwd: checkConfig.cwd,
        timeout: 300_000,
        maxBuffer: 100 * 1024 * 1024,
      }).toString();
    } catch (e: any) {
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize check crashed (exit code ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    const parsed = normalizeEnvironmentDependentDiagnostics(JSON.parse(stdout));
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");

    const prettyOutput =
      JSON.stringify(parsed, null, 2).replaceAll(checkConfig.cwd, "<cwd>") + "\n";
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
