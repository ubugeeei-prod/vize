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
const pluginLintSessionExport =
  "module.exports.PluginLintSession = nativeBinding.PluginLintSession;\n";
const configStdinCapabilityExport =
  "module.exports.supportsCliConfigStdin = nativeBinding.supportsCliConfigStdin;\n";
// #13074: Vite hosts register a resolved `vue.plugins` plan once.
const registerPluginPlanExport =
  "module.exports.registerPluginPlan = nativeBinding.registerPluginPlan;\n";
const compileTsrxExport = "module.exports.compileTsrx = nativeBinding.compileTsrx;\n";
// Builds compile a batch for their client and SSR environments in one call.
const compileForTargetsExport =
  "module.exports.compileSfcBatchForTargets = nativeBinding.compileSfcBatchForTargets;\n";
const formatFileExports =
  "module.exports.formatFile = nativeBinding.formatFile;\n" +
  "module.exports.formatFileSync = nativeBinding.formatFileSync;\n";
const sha256 = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");

// native-binding.js gained load-failure diagnostics (the glibc floor, dlopen
// hints, absent platform packages). They replace only `buildLoadError`, and
// the loader records the targets it tried; loading itself is unchanged.
const attemptedTargetsDeclaration = "const loadErrors = [];\nlet attemptedTargets = [];\n";
const attemptedTargetsLoop =
  "  attemptedTargets = nativeTargets(loadErrors);\n  for (const target of attemptedTargets) {\n";
const loadDiagnosticsStart = "// Every Linux GNU binding is linked against this glibc";
const loadDiagnosticsEnd = "let nativeBinding = loadNativeBinding();\n";
const loadDiagnosticsSha256 = "339ebee50256785fa10b1ee1e06639f7ced67a98aa443ae1ccf2d69005b8fd39";
const originalBuildLoadError = `function buildLoadError() {
  if (loadErrors.length === 0) {
    return new Error("Failed to load native binding");
  }

  const loadFailure = loadErrors.find((error) => !error?.vizeMissingNativeSpecifier);
  if (loadFailure) {
    return new Error(\`Failed to load native binding: \${loadFailure.message}\`, {
      cause: loadErrors.reduce((error, current) => {
        current.cause = error;
        return current;
      }),
    });
  }

  return new Error(
    \`Cannot find native binding. \` +
      \`npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828). \` +
      "Please try \`npm i\` again after removing both package-lock.json and node_modules directory.",
    {
      cause: loadErrors.reduce((error, current) => {
        current.cause = error;
        return current;
      }),
    },
  );
}

`;

function originalNativeBinding(text: string): string {
  for (const [piece, label] of [
    [attemptedTargetsDeclaration, "attempted target declaration"],
    [attemptedTargetsLoop, "attempted target loop"],
    [loadDiagnosticsStart, "load diagnostics"],
    [loadDiagnosticsEnd, "native binding initialization"],
  ] as const) {
    assert.equal(text.split(piece).length, 2, `exactly one ${label}`);
  }
  const start = text.indexOf(loadDiagnosticsStart);
  const end = text.indexOf(loadDiagnosticsEnd);
  assert.ok(start < end, "load diagnostics precede initialization");
  assert.equal(sha256(text.slice(start, end)), loadDiagnosticsSha256, "exact load diagnostics");
  return (text.slice(0, start) + originalBuildLoadError + text.slice(end))
    .replace(attemptedTargetsDeclaration, "const loadErrors = [];\n")
    .replace(attemptedTargetsLoop, "  for (const target of nativeTargets(loadErrors)) {\n");
}

/** Keep the original authenticated loader law through the exact public additions. */
export function originalNativeLoaderModule(name: string, body: Buffer): Buffer {
  const reference = nativeLoaderProgram.modules.find((module) => module.name === name);
  assert.ok(reference, "unknown native loader module");
  if (body.length === reference.bytes && sha256(body) === reference.sha256) return body;
  if (name === "native-binding.js") {
    const original = Buffer.from(originalNativeBinding(body.toString("utf8")));
    assert.equal(original.length, reference.bytes, "complete original native binding length");
    assert.equal(sha256(original), reference.sha256, "complete original native binding program");
    return original;
  }
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
        pluginLintSessionExport +
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
  assert.equal(
    text.split(
      "module.exports.runCli = nativeBinding.runCli;\n" +
        configStdinCapabilityExport +
        "module.exports.scopeViteCssForPipeline = nativeBinding.scopeViteCssForPipeline;\n",
    ).length,
    2,
    "config stdin capability retains its canonical initialization position",
  );
  assert.equal(
    text.split(
      "module.exports.compileSfcBatchWithResults = nativeBinding.compileSfcBatchWithResults;\n" +
        compileTsrxExport +
        "module.exports.compileVapor = nativeBinding.compileVapor;\n",
    ).length,
    2,
    "TSRX compile export retains its canonical initialization position",
  );
  assert.equal(
    text.split(
      "module.exports.compileSfcBatch = nativeBinding.compileSfcBatch;\n" +
        compileForTargetsExport +
        "module.exports.compileSfcBatchWithResults = nativeBinding.compileSfcBatchWithResults;\n",
    ).length,
    2,
    "multi-target batch export retains its canonical initialization position",
  );
  assert.equal(
    text.split(
      "module.exports.printCssAst = nativeBinding.printCssAst;\n" +
        registerPluginPlanExport +
        "module.exports.resolveDesignTokenReferences = nativeBinding.resolveDesignTokenReferences;\n",
    ).length,
    2,
    "plugin plan registration retains its canonical initialization position",
  );
  // Earlier approved facades omit this addition; both must reduce to the
  // same complete historical program below.
  if (text.includes(formatFileExports)) {
    assert.equal(
      text.split(
        "module.exports.flattenDesignTokenCategories = nativeBinding.flattenDesignTokenCategories;\n" +
          formatFileExports +
          "module.exports.formatSfc = nativeBinding.formatSfc;\n",
      ).length,
      2,
      "source formatting exports retain their canonical initialization position",
    );
  }
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
      .replace(pluginLintSessionExport, "")
      .replace(typeResolutionBatchClassExport, "")
      .replace(typeResolutionBatchFunctionExport, "")
      .replace(configStdinCapabilityExport, "")
      .replace(compileTsrxExport, "")
      .replace(registerPluginPlanExport, "")
      .replace(compileForTargetsExport, "")
      .replace(formatFileExports, "")
      .replace(addedExports, ""),
  );
  assert.equal(original.length, reference.bytes, "complete original native loader length");
  assert.equal(sha256(original), reference.sha256, "complete original native loader program");
  return original;
}
