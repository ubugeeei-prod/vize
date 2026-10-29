import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { stylePreprocessorsApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { runVizeCheckJson, stringifyDiagnosticSnapshot } from "../../_helpers/vize-check.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = stylePreprocessorsApp;
const qualifiedWidenTemplateRef = 'import("vize:virtual-ts").$WidenTemplateRef';
const currentReceipt = {
  helperHoistProvenance: "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  snapshotSha256: "8b116c0f795115e2e0f6648e79194119d61519d86601bc85b5b9fe3d32834450",
  widenHelperFiles: [
    "src/CssModuleScss.vue",
    "src/LessStyle.vue",
    "src/ScopedScss.vue",
    "src/ScssStyle.vue",
    "src/StylusStyle.vue",
  ],
} as const;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check passes with no errors for style preprocessors", () => {
    const checkConfig = app.check!;
    const parsed = runVizeCheckJson(checkConfig.cwd, checkConfig.patterns, {
      showVirtualTs: true,
    });
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");
    assert.equal(parsed.errorCount, 0, "errorCount should be 0 (style lang does not affect TS)");
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedWidenTemplateRef) === true)
        .map(({ file }) => file),
      currentReceipt.widenHelperFiles,
      `qualified widening helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    for (const { file, virtualTs = "" } of parsed.files) {
      assert.equal(
        count(virtualTs, "$WidenTemplateRef"),
        count(virtualTs, qualifiedWidenTemplateRef),
        `${file} must not restore a file-local widening helper`,
      );
    }
    for (const file of currentReceipt.widenHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(
        count(virtualTs, qualifiedWidenTemplateRef),
        1,
        `${file} must use one shared helper`,
      );
    }

    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
    assert.equal(
      sha256(prettyOutput),
      currentReceipt.snapshotSha256,
      `current snapshot needs an intentional successor after ${currentReceipt.helperHoistProvenance}`,
    );
  });
});

function count(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
