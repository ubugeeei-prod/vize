import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { typecheckErrorsApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import {
  runVizeCheckJson,
  stringifyDiagnosticSnapshot,
  type VizeCheckJson,
} from "../../_helpers/vize-check.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = typecheckErrorsApp;
const qualifiedEmitProps = 'import("vize:virtual-ts").__EmitProps<';
const qualifiedWidenTemplateRef = 'import("vize:virtual-ts").$WidenTemplateRef';
const currentReceipt = {
  diagnosticSha256: "6b8cd236559e23b4795f404049af0ed5d6b3b9748b28fdd5867b74ebc3d43eef",
  emitHelperFiles: ["src/DefineEmitsMethodInterfaceError.vue", "src/EmitArgsMismatch.vue"],
  errorCount: 26,
  fileCount: 18,
  files: [
    "src/DefineEmitsMethodInterfaceError.vue",
    "src/DefineExposeError.vue",
    "src/DefinePropsRuntimeDeclError.vue",
    "src/DefineSlotsError.vue",
    "src/EmitArgsMismatch.vue",
    "src/InlineHandlerError.vue",
    "src/InvalidTemplateExpr.vue",
    "src/MissingRequiredProp.vue",
    "src/NoImplicitAnyError.vue",
    "src/PropsTypeMismatch.vue",
    "src/ProvideInjectSymbolError.vue",
    "src/TemplateInlineFunction.vue",
    "src/TypeMismatch.vue",
    "src/UndefinedVariable.vue",
    "src/UseTemplateRefError.vue",
    "src/VBindWrongType.vue",
    "src/WrongEventHandler.vue",
    "src/WrongFunctionArgs.vue",
  ],
  helperHoistProvenance: "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  provenance:
    "#10950 / 83e3ff318fb772340663b5688624dac2ad37e2b1, " +
    "#11064 / c2ee0878fec5d53edb7c30e10f374e37ab256f02, and " +
    "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  slotAliasProvenance: "#11064 / c2ee0878fec5d53edb7c30e10f374e37ab256f02",
  slotCallFiles: ["src/DefineSlotsError.vue"],
  // Exact source packet successor: #11153 / Actions 41097013703, artifact 12996064001.
  snapshotSha256: "bf2513812218645cc69333732bec578706d6a960870650dc8bbdcd4569821872",
  warningCount: 0,
  widenHelperFiles: [
    "src/DefineExposeError.vue",
    "src/EmitArgsMismatch.vue",
    "src/InlineHandlerError.vue",
    "src/InvalidTemplateExpr.vue",
    "src/PropsTypeMismatch.vue",
    "src/ProvideInjectSymbolError.vue",
    "src/TemplateInlineFunction.vue",
    "src/UseTemplateRefError.vue",
    "src/VBindWrongType.vue",
    "src/WrongEventHandler.vue",
    "src/WrongFunctionArgs.vue",
  ],
} as const;

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
    assert.deepEqual(
      [parsed.fileCount, parsed.errorCount, parsed.warningCount],
      [currentReceipt.fileCount, currentReceipt.errorCount, currentReceipt.warningCount],
      `diagnostic counts changed after ${currentReceipt.provenance}`,
    );
    assert.deepEqual(
      parsed.files.map(({ file }) => file),
      currentReceipt.files,
      `diagnostic files changed after ${currentReceipt.provenance}`,
    );
    assert.equal(
      sha256(diagnosticReceipt(parsed)),
      currentReceipt.diagnosticSha256,
      `diagnostics changed after ${currentReceipt.provenance}`,
    );
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedWidenTemplateRef) === true)
        .map(({ file }) => file),
      currentReceipt.widenHelperFiles,
      `qualified widening helpers changed after ${currentReceipt.helperHoistProvenance}`,
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
        count(virtualTs, "$WidenTemplateRef"),
        count(virtualTs, qualifiedWidenTemplateRef),
        `${file} must not restore a file-local widening helper`,
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
    for (const file of currentReceipt.widenHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(
        count(virtualTs, qualifiedWidenTemplateRef),
        1,
        `${file} must use one shared widening helper`,
      );
    }
    for (const file of currentReceipt.emitHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(count(virtualTs, qualifiedEmitProps), 5, `${file} must use five shared helpers`);
    }
    const slotFile = parsed.files.find(({ file }) => file === currentReceipt.slotCallFiles[0]);
    assert.ok(slotFile, "DefineSlotsError.vue is part of the current receipt");
    assert.equal(count(slotFile.virtualTs ?? "", "__vizeSlotOutlet("), 1);

    const templateRefFile = parsed.files.find(({ file }) => file === "src/UseTemplateRefError.vue");
    assert.ok(templateRefFile, "UseTemplateRefError.vue is part of the current receipt");
    const templateRefVirtualTs = templateRefFile.virtualTs ?? "";
    assert.match(
      templateRefVirtualTs,
      /^import \{ ref, onMounted, useTemplateRef \} from "vue";$/m,
    );
    assert.equal(templateRefVirtualTs.includes("Keep the authored Vue helper import"), false);
    assert.equal(templateRefVirtualTs.includes("\nvoid useTemplateRef;\n"), false);

    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
    assert.equal(
      sha256(prettyOutput),
      currentReceipt.snapshotSha256,
      `current snapshot needs an intentional successor after ${currentReceipt.provenance}`,
    );
  });
});

function count(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

function diagnosticReceipt(parsed: VizeCheckJson): string {
  return JSON.stringify(parsed.files.map(({ diagnostics, file }) => ({ file, diagnostics })));
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
