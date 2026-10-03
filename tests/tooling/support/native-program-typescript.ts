// Actual Rust-generated checker inputs arrive on stdin. This helper is valid JS
// so the same source runs inside the Rust law without another transpilation.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const fromUi = createRequire(path.join(process.cwd(), "npm/ui/package.json"));
const ts = process.env.VIZE_TYPESCRIPT_PACKAGE
  ? createRequire(process.env.VIZE_TYPESCRIPT_PACKAGE)("./lib/typescript.js")
  : fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const inputs = JSON.parse(fs.readFileSync(0, "utf8"));
const cases = [];
for (const input of inputs) {
  const fixture = input.case;
  const js = fixture.kind === "js";
  assert.equal(input.extension, js ? "mjs" : "ts");
  const filename = path.resolve(`native-projection-${fixture.id}.${input.extension}`);
  const options = {
    noEmit: true,
    strict: true,
    allowJs: js,
    checkJs: js,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    moduleDetection: ts.ModuleDetectionKind.Force,
    skipLibCheck: true,
  };
  const check = (text) => {
    const host = ts.createCompilerHost(options);
    const read = host.readFile.bind(host);
    host.readFile = (name) => (name === filename ? text : read(name));
    // The real CompilerHost applies ModuleDetection and infers ScriptKind from
    // the sealed extension. A detached createSourceFile drops that option.
    const program = ts.createProgram([filename], options, host);
    assert.equal(
      program.getSourceFile(filename).scriptKind,
      js ? ts.ScriptKind.JS : ts.ScriptKind.TS,
    );
    return ts.getPreEmitDiagnostics(program).map((diagnostic) => {
      assert.equal(diagnostic.file?.fileName, filename, "unexpected project/global diagnostic");
      assert.equal(typeof diagnostic.start, "number");
      assert.equal(typeof diagnostic.length, "number");
      return {
        code: diagnostic.code,
        category: diagnostic.category,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
        start: diagnostic.start,
        length: diagnostic.length,
        authored: text.slice(diagnostic.start, diagnostic.start + diagnostic.length),
      };
    });
  };
  const original = check(fixture.source);
  const diagnostics = check(input.projected);
  assert.deepEqual(diagnostics, original, `${fixture.id}: complete checker diagnostics`);
  assert.deepEqual(
    diagnostics.map((value) => value.code),
    fixture.codes,
    fixture.id,
  );
  assert.deepEqual(diagnostics, fixture.diagnostics, `${fixture.id}: independent full reference`);
  cases.push({ id: fixture.id, kind: fixture.kind, extension: input.extension, diagnostics });
}
process.stdout.write(JSON.stringify({ version: ts.version, cases }));
