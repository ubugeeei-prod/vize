import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { nativeLoaderProgram } from "../../../tools/support/release/human_reports_11252_11253_public/native-loader-program.ts";

const addedExports =
  "module.exports.typeProjectInputs = nativeBinding.typeProjectInputs;\n" +
  "module.exports.typeProjectOwners = nativeBinding.typeProjectOwners;\n";
const typeResolutionBatchWrapper = `const compileSfcNative = nativeBinding.compileSfc;
const beginTypeResolutionBatchNative = nativeBinding.beginTypeResolutionBatch;
let activeTypeResolutionTurn = null;
nativeBinding.compileSfc = function compileSfc(source, options) {
  if (typeof beginTypeResolutionBatchNative !== "function") {
    return compileSfcNative(source, options);
  }
  if (activeTypeResolutionTurn === null) {
    const turn = beginTypeResolutionBatchNative();
    activeTypeResolutionTurn = turn;
    queueMicrotask(() => {
      if (activeTypeResolutionTurn !== turn) return;
      activeTypeResolutionTurn = null;
      turn.close();
    });
  }
  return compileSfcNative(source, options);
};

`;
const typeResolutionBatchClassExport =
  "module.exports.TypeResolutionBatch = nativeBinding.TypeResolutionBatch;\n";
const typeResolutionBatchFunctionExport =
  "module.exports.beginTypeResolutionBatch = nativeBinding.beginTypeResolutionBatch;\n";
const sha256 = (body: Buffer) => createHash("sha256").update(body).digest("hex");

/** Keep the original authenticated loader law through the exact public additions. */
export function originalNativeLoaderModule(name: string, body: Buffer): Buffer {
  const reference = nativeLoaderProgram.modules.find((module) => module.name === name);
  assert.ok(reference, "unknown native loader module");
  if (body.length === reference.bytes && sha256(body) === reference.sha256) return body;
  assert.equal(name, "index.js", "only the public project API facade changed");
  const text = body.toString("utf8");
  assert.deepEqual(Buffer.from(text), body);
  assert.equal(
    text.split(
      'const nativeBinding = require("./native-binding");\n\n' +
        typeResolutionBatchWrapper +
        "module.exports = nativeBinding;\n",
    ).length,
    2,
    "type-resolution batch wrapper retains its canonical initialization position",
  );
  assert.equal(
    text.split(
      "module.exports.PluginDocumentHandle = nativeBinding.PluginDocumentHandle;\n" +
        typeResolutionBatchClassExport +
        "module.exports.applyOutputPlugins = nativeBinding.applyOutputPlugins;\n",
    ).length,
    2,
    "type-resolution batch class export retains its canonical initialization position",
  );
  assert.equal(
    text.split(
      "module.exports.artToCsf = nativeBinding.artToCsf;\n" +
        typeResolutionBatchFunctionExport +
        "module.exports.buildDesignTokenMap = nativeBinding.buildDesignTokenMap;\n",
    ).length,
    2,
    "type-resolution batch function export retains its canonical initialization position",
  );
  assert.equal(text.split(addedExports).length, 2, "exact project API export addition");
  const before =
    "module.exports.typeCheckWithVirtualTsVisitors = nativeBinding.typeCheckWithVirtualTsVisitors;\n";
  const after =
    "module.exports.validateDesignTokenReference = nativeBinding.validateDesignTokenReference;\n";
  assert.equal(
    text.split(before + addedExports + after).length,
    2,
    "project API exports retain their canonical initialization position",
  );
  const original = Buffer.from(
    text
      .replace(typeResolutionBatchWrapper, "")
      .replace(typeResolutionBatchClassExport, "")
      .replace(typeResolutionBatchFunctionExport, "")
      .replace(addedExports, ""),
  );
  assert.equal(original.length, reference.bytes, "complete original native loader length");
  assert.equal(sha256(original), reference.sha256, "complete original native loader program");
  return original;
}
