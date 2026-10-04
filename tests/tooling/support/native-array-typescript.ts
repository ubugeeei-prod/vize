// Only complete Rust-produced original-owner projections arrive on stdin.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const fromUi = createRequire(path.join(process.cwd(), "npm/ui/package.json"));
const ts = process.env.VIZE_TYPESCRIPT_PACKAGE
  ? createRequire(process.env.VIZE_TYPESCRIPT_PACKAGE)("./lib/typescript.js")
  : fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const cases = [];
for (const input of JSON.parse(fs.readFileSync(0, "utf8"))) {
  const fixture = input.case;
  assert.ok(["ts", "tsx"].includes(input.extension));
  const filename = path.resolve(`original-array-${fixture.id}.${input.extension}`);
  const options = {
    noEmit: true,
    strict: true,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    moduleDetection: ts.ModuleDetectionKind.Force,
    skipLibCheck: true,
    jsx: ts.JsxEmit.Preserve,
  };
  const check = (text) => {
    const host = ts.createCompilerHost(options);
    const read = host.readFile.bind(host);
    host.readFile = (name) => (name === filename ? text : read(name));
    const program = ts.createProgram([filename], options, host);
    assert.equal(
      program.getSourceFile(filename).scriptKind,
      input.extension === "tsx" ? ts.ScriptKind.TSX : ts.ScriptKind.TS,
    );
    const record = (diagnostic) => {
      assert.equal(diagnostic.file?.fileName, filename);
      assert.equal(typeof diagnostic.start, "number");
      assert.equal(typeof diagnostic.length, "number");
      return {
        code: diagnostic.code,
        category: diagnostic.category,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
        start: diagnostic.start,
        length: diagnostic.length,
        authored: text.slice(diagnostic.start, diagnostic.start + diagnostic.length),
        ...(diagnostic.relatedInformation
          ? { related: diagnostic.relatedInformation.map(record) }
          : {}),
      };
    };
    return ts.getPreEmitDiagnostics(program).map(record);
  };
  const original = check(fixture.source);
  const projected = check(input.projected);
  assert.deepEqual(projected, original, `${fixture.id}: complete original/projected vector`);
  assert.deepEqual(original, fixture.diagnostics, `${fixture.id}: independent primary vector`);
  cases.push({ id: fixture.id, extension: input.extension, diagnostics: projected });
}
process.stdout.write(JSON.stringify({ version: ts.version, cases }));
