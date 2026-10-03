// Independent editor diagnostics; compiler pre-emit goldens remain separate.
// Valid JS also runs directly in the Rust fixture without transpilation.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const fromUi = createRequire(path.join(process.cwd(), "npm/ui/package.json"));
const ts = process.env.VIZE_TYPESCRIPT_PACKAGE
  ? createRequire(process.env.VIZE_TYPESCRIPT_PACKAGE)("./lib/typescript.js")
  : fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const options = {
  noEmit: true,
  strict: true,
  allowJs: true,
  checkJs: true,
  types: [],
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  moduleDetection: ts.ModuleDetectionKind.Force,
  skipLibCheck: true,
};
const capabilities = {
  textDocument: {
    diagnostic: { dynamicRegistration: false, relatedDocumentSupport: true },
  },
  positionEncoding: "utf-16",
};
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const sourceIdentity = (source) => ({
  utf8Base64: Buffer.from(source, "utf8").toString("base64"),
  sha256: sha256(Buffer.from(source, "utf8")),
  utf8Bytes: Buffer.byteLength(source, "utf8"),
  utf16Units: source.length,
  hasInitialBom: source.charCodeAt(0) === 0xfeff,
});

function rawDiagnostic(value, logicalSource, filename, active = new Set()) {
  if (value === undefined) return { __oracleValue: "undefined" };
  if (value === null || typeof value !== "object") {
    assert.ok(!["function", "symbol", "bigint"].includes(typeof value));
    return value;
  }
  // SourceFile contains cyclic AST links. Retain exact source identity instead
  // of serializing or traversing its syntax tree; other diagnostic keys survive.
  if (ts.isSourceFile(value)) {
    return {
      __oracleValue: "SourceFileSummary",
      fileName: value.fileName,
      logicalSource: path.resolve(value.fileName) === filename ? logicalSource : null,
      text: value.text,
      scriptKind: value.scriptKind,
      languageVersion: value.languageVersion,
      isDeclarationFile: value.isDeclarationFile,
      ...sourceIdentity(value.text),
    };
  }
  assert.ok(!active.has(value), "unexpected cycle in diagnostic fields");
  active.add(value);
  let result;
  if (Array.isArray(value)) {
    result = value.map((item) => rawDiagnostic(item, logicalSource, filename, active));
  } else {
    assert.equal(Object.getOwnPropertySymbols(value).length, 0);
    result = Object.fromEntries(
      Object.getOwnPropertyNames(value).map((key) => [
        key,
        rawDiagnostic(value[key], logicalSource, filename, active),
      ]),
    );
  }
  active.delete(value);
  return result;
}

function protocolDiagnostic(diagnostic) {
  const severity = new Map([
    [ts.DiagnosticCategory.Error, 1],
    [ts.DiagnosticCategory.Warning, 2],
    [ts.DiagnosticCategory.Suggestion, 4],
    [ts.DiagnosticCategory.Message, 3],
  ]).get(diagnostic.category);
  assert.notEqual(severity, undefined);
  let start = { line: 0, character: 0 };
  let end = start;
  if (diagnostic.file) {
    assert.ok(Number.isInteger(diagnostic.start) && diagnostic.start >= 0);
    assert.ok(Number.isInteger(diagnostic.length) && diagnostic.length >= 0);
    assert.ok(diagnostic.start + diagnostic.length <= diagnostic.file.text.length);
    start = diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start);
    end = diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start + diagnostic.length);
  }
  // Pinned TS7 pull converter and actual unadvertised related/tag capabilities:
  // every diagnostic survives, while those optional protocol fields are absent.
  return {
    range: { start, end },
    severity,
    code: diagnostic.code,
    source: "ts",
    message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
  };
}

function capture(source, filename, logicalSource, scriptKind) {
  const host = {
    getCompilationSettings: () => options,
    getScriptFileNames: () => [filename],
    getScriptVersion: () => "1",
    getScriptKind: (name) => (path.resolve(name) === filename ? scriptKind : undefined),
    getScriptSnapshot: (name) => {
      const text = path.resolve(name) === filename ? source : ts.sys.readFile(name);
      return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
    },
    getCurrentDirectory: () => path.dirname(filename),
    getDefaultLibFileName: (settings) => ts.getDefaultLibFilePath(settings),
    fileExists: ts.sys.fileExists,
    readFile: (name) => (path.resolve(name) === filename ? source : ts.sys.readFile(name)),
    readDirectory: ts.sys.readDirectory,
    useCaseSensitiveFileNames: () => ts.sys.useCaseSensitiveFileNames,
  };
  const service = ts.createLanguageService(host);
  try {
    const file = service.getProgram()?.getSourceFile(filename);
    assert.ok(file, "actual LanguageService owns the requested source");
    assert.equal(file.text, source);
    assert.equal(file.scriptKind, scriptKind);
    assert.ok(file.externalModuleIndicator, "actual Module Force is retained");
    const groups = [
      ["syntactic", service.getSyntacticDiagnostics(filename)],
      ["semantic", service.getSemanticDiagnostics(filename)],
      ["suggestion", service.getSuggestionDiagnostics(filename)],
    ];
    const diagnostics = groups.flatMap(([, values]) => values);
    return {
      source,
      identity: sourceIdentity(source),
      scriptKind: file.scriptKind,
      groups: groups.map(([api, values]) => ({ api, length: values.length })),
      diagnostics: diagnostics.map(protocolDiagnostic),
      rawDiagnostics: diagnostics.map((value) => rawDiagnostic(value, logicalSource, filename)),
    };
  } finally {
    service.dispose();
  }
}

const inputs = JSON.parse(fs.readFileSync(0, "utf8"));
assert.ok(Array.isArray(inputs));
const cases = inputs.map((input) => {
  const fixture = input.case;
  assert.ok(["js", "ts"].includes(fixture.kind));
  assert.equal(typeof fixture.source, "string");
  const extension = fixture.kind === "js" ? "mjs" : "ts";
  assert.equal(input.extension, extension);
  const filename = path.resolve(input.physicalPath);
  assert.equal(path.extname(filename), `.${extension}`);
  const bytes = fs.readFileSync(filename);
  assert.deepEqual(bytes, Buffer.from(fixture.source, "utf8"), "original physical bytes");
  const physical = ts.sys.readFile(filename);
  assert.equal(typeof physical, "string");
  const logicalSource = { family: input.family, id: fixture.id, extension };
  const scriptKind = fixture.kind === "js" ? ts.ScriptKind.JS : ts.ScriptKind.TS;
  let openedProjection = { unavailable: true, reason: "actual Rust output not supplied" };
  if (input.projected !== undefined) {
    assert.equal(typeof input.projected, "string");
    assert.ok(input.projected.startsWith(fixture.source), "authored whole-source prefix");
    openedProjection = capture(input.projected, filename, logicalSource, scriptKind);
  }
  return {
    family: input.family,
    id: fixture.id,
    kind: fixture.kind,
    extension,
    opened: capture(fixture.source, filename, logicalSource, scriptKind),
    physical: capture(physical, filename, logicalSource, scriptKind),
    openedProjection,
    provenance: {
      physicalPath: filename,
      physicalBytesSha256: sha256(bytes),
      physicalBytesBase64: bytes.toString("base64"),
      openedSnapshot: "ts.ScriptSnapshot.fromString(raw original fixture.source)",
      physicalSnapshot: "ts.ScriptSnapshot.fromString(ts.sys.readFile(actual original file))",
      projectedSnapshot:
        input.projected === undefined
          ? null
          : "ts.ScriptSnapshot.fromString(actual Rust projection.document() input)",
      diskBytesEqualOriginal: true,
      providerTextEqualsOpened: physical === fixture.source,
      offsetNormalization: false,
    },
  };
});
process.stdout.write(
  JSON.stringify({
    version: ts.version,
    options,
    capabilities,
    protocolConversion: {
      provider: "microsoft/typescript-go@2bd066d87f5bafd315be9f40889d0a60b9e58e0b",
      converterSha256: "54d5881b5596cf99074e168ed54aacbb582c62e3cc4294c5f1a3ed1d989fd3ac",
      order: ["syntactic", "semantic", "suggestion"],
      absentFields: ["relatedInformation", "tags", "codeDescription", "data"],
      relatedInformationAdvertised: false,
      tagSupportAdvertised: false,
      visualStudioExtensionsAdvertised: false,
      reportStyleChecksAsWarnings: false,
      rawUndefinedEncoding: { __oracleValue: "undefined" },
      rawSourceFileEncoding: "explicit exact source summary, no AST serialization",
      compilerPreEmitGoldensChanged: false,
      sdkResultIdAndRelatedDocuments:
        "not produced by TypeScript LanguageService; preserve actual SDK report separately",
    },
    cases,
  }),
);
