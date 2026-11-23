const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");
const fromTests = createRequire(path.join(process.argv[2], "tests/package.json"));
const ts = fromTests("typescript");
const cases = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
const options = {
  strict: true,
  skipLibCheck: true,
  noEmit: true,
  types: [],
  target: ts.ScriptTarget.ESNext,
  jsx: ts.JsxEmit.Preserve,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
};
const rows = cases.map(({ name, directory, files }) => {
  const program = ts.createProgram(
    files.map((file) => path.join(directory, file)),
    options,
  );
  const diagnostics = ts.getPreEmitDiagnostics(program).map((d) => ({
    code: d.code,
    category: d.category,
    start: d.start,
    length: d.length,
    file: d.file && path.relative(directory, d.file.fileName),
    message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
  }));
  return { name, diagnostics };
});
process.stdout.write(JSON.stringify({ typescript: ts.version, rows }));
