const fs = require("node:fs");
const path = require("node:path");
const { createRequire } = require("node:module");
const workspace = process.argv[2],
  directory = process.argv[3];
const fromTests = createRequire(path.join(workspace, "tests/package.json"));
const ts = fromTests("typescript");
const vuePackage = fromTests.resolve("vue/package.json");
const fromVue = createRequire(vuePackage);
const fromRuntimeDom = createRequire(fromVue.resolve("@vue/runtime-dom/package.json"));
const runtimeCorePackage = fromRuntimeDom.resolve("@vue/runtime-core/package.json");
// Bind the augmentation to this Vue's dependency, not a neighboring workspace
// runtime-core version or an unresolved ambient import hidden by skipLibCheck.
fs.mkdirSync(path.join(directory, "node_modules/@vue"), { recursive: true });
fs.symlinkSync(path.dirname(vuePackage), path.join(directory, "node_modules/vue"), "junction");
fs.symlinkSync(
  path.dirname(runtimeCorePackage),
  path.join(directory, "node_modules/@vue/runtime-core"),
  "junction",
);
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
const modules = JSON.parse(fs.readFileSync(path.join(directory, "modules.json"), "utf8"));
const host = ts.createCompilerHost(options);
// This checks complete native modules with the declared JS compiler, not the
// external mapper protocol. The official TS7 carrier checks that separately.
host.resolveModuleNames = (names, containing) =>
  names.map((name) => {
    const virtual = modules[path.resolve(path.dirname(containing), name)];
    // The authored specifier is .vue, not a TypeScript-extension import. Return
    // the registered virtual module instead of resolving a fabricated .tsx import.
    if (virtual)
      return {
        resolvedFileName: virtual,
        extension: path.extname(virtual),
        isExternalLibraryImport: false,
      };
    return ts.resolveModuleName(name, containing, options, host).resolvedModule;
  });
const files = fs.readdirSync(directory).filter((file) => /\.tsx?$/.test(file));
const program = ts.createProgram(
  files.map((file) => path.join(directory, file)),
  options,
  host,
);
const diagnostics = ts.getPreEmitDiagnostics(program).map((d) => ({
  code: d.code,
  category: d.category,
  start: d.start,
  length: d.length,
  file: d.file && path.relative(directory, d.file.fileName),
  message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
}));
process.stdout.write(
  JSON.stringify({
    typescript: ts.version,
    vue: fromTests("vue/package.json").version,
    runtimeCore: JSON.parse(fs.readFileSync(runtimeCorePackage, "utf8")).version,
    diagnostics,
  }),
);
