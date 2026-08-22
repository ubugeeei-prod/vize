import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { typecheckErrorsApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { runVizeCheckJson, stringifyDiagnosticSnapshot } from "../../_helpers/vize-check.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = typecheckErrorsApp;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check detects type errors", () => {
    const checkConfig = app.check!;
    const parsed = runVizeCheckJson(checkConfig.cwd, checkConfig.patterns, {
      showVirtualTs: true,
    });
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");
    assert.ok(parsed.errorCount > 0, "errorCount should be > 0 (intentional type errors)");

    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
