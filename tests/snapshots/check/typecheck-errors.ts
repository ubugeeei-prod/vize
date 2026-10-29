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
const currentReceipt = {
  diagnosticSha256: "6b8cd236559e23b4795f404049af0ed5d6b3b9748b28fdd5867b74ebc3d43eef",
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
  provenance: "#10950 / 83e3ff318fb772340663b5688624dac2ad37e2b1",
  snapshotSha256: "849b5fc7e9d28d6f1061f79c594e7723d228cadb5e65746bbc43fa6867b4a3ce",
  warningCount: 0,
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
    assert.equal(
      sha256(prettyOutput),
      currentReceipt.snapshotSha256,
      `current snapshot needs an intentional successor after ${currentReceipt.provenance}`,
    );
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});

function diagnosticReceipt(parsed: VizeCheckJson): string {
  return JSON.stringify(parsed.files.map(({ diagnostics, file }) => ({ file, diagnostics })));
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
