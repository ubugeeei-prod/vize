import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { compilerMacrosApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { runVizeCheckJson, stringifyDiagnosticSnapshot } from "../../_helpers/vize-check.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = compilerMacrosApp;
const qualifiedEmitProps = 'import("vize:virtual-ts").__EmitProps<';
// Ref shadows name the shared unwrap alias directly (#12431).
const qualifiedTemplateRefUnwrap = 'import("vize:virtual-ts").$TemplateRefUnwrap';
const currentReceipt = {
  emitHelperFiles: [
    "src/DefineEmitsMethodInterface.vue",
    "src/DefineEmitsType.vue",
    "src/DefineModelType.vue",
  ],
  helperHoistProvenance: "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  slotAliasProvenance: "#11064 / c2ee0878fec5d53edb7c30e10f374e37ab256f02",
  slotCallFiles: ["src/DefineSlotsType.vue"],
  // Regenerated at the shared-unwrap head: every diagnostic field is unchanged; only
  // `virtualTs` moved (shared unwrap references and the constructor shape since #12162).
  snapshotSha256: "5d8ea46dbc67e3962600ef74a1bf45db4157d16af738ce6f1a22affef3f865b9",
  unwrapHelperFiles: [
    "src/DefineExposeType.vue",
    "src/DefineModelType.vue",
    "src/ProvideInjectSymbol.vue",
    "src/TopLevelAwait.vue",
  ],
} as const;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check does not crash and snapshot matches", () => {
    const checkConfig = app.check!;
    const parsed = runVizeCheckJson(checkConfig.cwd, checkConfig.patterns, {
      showVirtualTs: true,
    });
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");

    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedTemplateRefUnwrap) === true)
        .map(({ file }) => file),
      currentReceipt.unwrapHelperFiles,
      `qualified unwrap helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedEmitProps) === true)
        .map(({ file }) => file),
      currentReceipt.emitHelperFiles,
      `qualified emit helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes("__vizeSlotOutlet(") === true)
        .map(({ file }) => file),
      currentReceipt.slotCallFiles,
      `slot outlet owners changed after ${currentReceipt.slotAliasProvenance}`,
    );
    for (const { file, virtualTs = "" } of parsed.files) {
      assert.equal(
        count(virtualTs, "$TemplateRefUnwrap<"),
        count(virtualTs, `${qualifiedTemplateRefUnwrap}<`),
        `${file} must not restore a file-local unwrap helper`,
      );
      assert.equal(
        count(virtualTs, "__EmitProps<"),
        count(virtualTs, qualifiedEmitProps),
        `${file} must not restore a file-local emit helper`,
      );
      assert.equal(
        count(virtualTs, "type $SlotOutletTarget"),
        0,
        `${file} must not restore the unused slot target alias`,
      );
      assert.equal(
        count(virtualTs, "type $SlotOutletArgs"),
        0,
        `${file} must not restore the unused slot argument alias`,
      );
    }
    const slotFile = parsed.files.find(({ file }) => file === currentReceipt.slotCallFiles[0]);
    assert.ok(slotFile, "DefineSlotsType.vue is part of the compiler macro receipt");
    assert.equal(count(slotFile.virtualTs ?? "", "__vizeSlotOutlet("), 2);
    for (const file of currentReceipt.emitHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(count(virtualTs, qualifiedEmitProps), 5, `${file} must use five shared helpers`);
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
