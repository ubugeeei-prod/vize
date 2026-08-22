import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { ecosystemProductsApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { runVizeCheckJson, stringifyDiagnosticSnapshot } from "../../_helpers/vize-check.ts";

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
    const parsed = normalizeEnvironmentDependentDiagnostics(
      runVizeCheckJson(checkConfig.cwd, checkConfig.patterns, {
        showVirtualTs: true,
        timeoutMs: 300_000,
      }),
    );
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");

    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
