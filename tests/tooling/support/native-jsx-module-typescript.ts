// Rust-projected JSX/TSX module documents arrive on stdin. This helper is
// valid JS so the same source runs inside the Rust law without transpiling.
// All documents are checked in one program with the published ambient set,
// under npm/ui so `import("vue")` resolves to the workspace Vue.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const fromUi = createRequire(path.join(process.cwd(), "npm/ui/package.json"));
const ts = process.env.VIZE_TYPESCRIPT_PACKAGE
  ? createRequire(process.env.VIZE_TYPESCRIPT_PACKAGE)("./lib/typescript.js")
  : fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const input = JSON.parse(fs.readFileSync(0, "utf8"));
const globals = path.resolve("npm/cli/virtual-ts-globals.d.ts");
const files = new Map(
  input.cases.map((entry) => [path.resolve(`npm/ui/l4-jsx-module-${entry.id}.ts`), entry]),
);
const options = {
  noEmit: true,
  strict: true,
  types: [],
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  moduleDetection: ts.ModuleDetectionKind.Force,
  skipLibCheck: true,
};
const host = ts.createCompilerHost(options);
const read = host.readFile.bind(host);
const exists = host.fileExists.bind(host);
host.readFile = (name) => files.get(name)?.projected ?? read(name);
host.fileExists = (name) => files.has(name) || exists(name);
const program = ts.createProgram([...files.keys(), globals], options, host);
const checker = program.getTypeChecker();
const cases = [];
for (const [filename, entry] of files) {
  const file = program.getSourceFile(filename);
  assert.equal(file.scriptKind, ts.ScriptKind.TS);
  const diagnostics = [
    ...program.getSyntacticDiagnostics(file),
    ...program.getSemanticDiagnostics(file),
  ].map((diagnostic) => {
    assert.equal(typeof diagnostic.start, "number");
    assert.equal(typeof diagnostic.length, "number");
    return {
      code: diagnostic.code,
      category: diagnostic.category,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      start: diagnostic.start,
      length: diagnostic.length,
    };
  });
  cases.push({ id: entry.id, diagnostics });
}
// Every diagnostic belongs to a projected document.
const projectedFiles = new Set(files.keys());
for (const diagnostic of ts.getPreEmitDiagnostics(program)) {
  assert.ok(projectedFiles.has(diagnostic.file?.fileName), "project or global diagnostic");
}
// Each helper the target names resolves, from every document, only to the
// published ambient set: no document declares one.
const helpers = input.helpers.map((name) => {
  const declarations = new Set();
  for (const filename of files.keys()) {
    const meaning = ts.SymbolFlags.Value | ts.SymbolFlags.Type;
    const symbol = checker.resolveName(name, program.getSourceFile(filename), meaning, false);
    for (const declaration of symbol?.declarations ?? []) {
      const relative = path.relative(process.cwd(), declaration.getSourceFile().fileName);
      declarations.add(relative.split(path.sep).join("/"));
    }
  }
  return { name, declarations: [...declarations] };
});
process.stdout.write(JSON.stringify({ version: ts.version, cases, helpers }));
