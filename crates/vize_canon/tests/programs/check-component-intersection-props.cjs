const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");
const fromTests = createRequire(path.join(process.argv[2], "tests/package.json"));
const ts = fromTests("typescript");
const directory = process.argv[3];
const modules = JSON.parse(fs.readFileSync(path.join(directory, "modules.json"), "utf8"));
const options = {
  strict: true,
  exactOptionalPropertyTypes: true,
  noEmit: true,
  skipLibCheck: true,
  types: [],
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.Preserve,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  jsx: ts.JsxEmit.Preserve,
};
const host = ts.createCompilerHost(options);
host.resolveModuleNames = (names, containing) =>
  names.map((name) => {
    const virtual = modules[path.resolve(path.dirname(containing), name)];
    return virtual
      ? { resolvedFileName: virtual, extension: path.extname(virtual) }
      : ts.resolveModuleName(name, containing, options, host).resolvedModule;
  });
const program = ts.createProgram(
  fs
    .readdirSync(directory)
    .filter((file) => /\.tsx?$/.test(file))
    .map((file) => path.join(directory, file)),
  options,
  host,
);
process.stdout.write(
  JSON.stringify({
    typescript: ts.version,
    diagnostics: ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
      file: diagnostic.file && path.basename(diagnostic.file.fileName),
      code: diagnostic.code,
      start: diagnostic.start,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    })),
  }),
);
