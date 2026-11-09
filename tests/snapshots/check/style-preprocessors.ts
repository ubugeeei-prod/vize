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
// Ref shadows name the shared unwrap alias directly (#12431).
const qualifiedTemplateRefUnwrap = 'import("vize:virtual-ts").$TemplateRefUnwrap';
const currentReceipt = {
  helperHoistProvenance: "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  // Regenerated at the shared-unwrap head: every diagnostic field is unchanged; only
  // `virtualTs` moved (shared unwrap references, #12460, and the constructor shape since #12162).
  snapshotSha256: "a893eb11a886b33bac69d7cfbe4dad4011f816fd96a04b31209311d2bf5697ae",
  unwrapHelperFiles: [
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
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedTemplateRefUnwrap) === true)
        .map(({ file }) => file),
      currentReceipt.unwrapHelperFiles,
      `qualified unwrap helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    for (const { file, virtualTs = "" } of parsed.files) {
      assert.equal(
        count(virtualTs, "$TemplateRefUnwrap<"),
        count(virtualTs, `${qualifiedTemplateRefUnwrap}<`),
        `${file} must not restore a file-local unwrap helper`,
      );
    }
    for (const file of currentReceipt.unwrapHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.ok(
        count(virtualTs, qualifiedTemplateRefUnwrap) >= 1,
        `${file} must shadow its refs through the shared unwrap helper`,
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
