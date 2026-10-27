const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const ts = require("../node_modules/typescript");
const { installVueVirtualModules } = require("../../npm/typescript-plugin/virtual-modules.cjs");

void test("configured hosts refresh package-selected options once without replacing roots or descriptors", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-package-options-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const write = (name, value) => {
    const file = path.join(root, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, typeof value === "string" ? value : JSON.stringify(value));
    return file;
  };
  const control = write(
    "src/control.ts",
    "export {};\nswitch (1 as number) {\n  case 1: { let value = 1; value++; }\n  case 2: break;\n}\n",
  );
  const config = write("tsconfig.json", {
    extends: "./inherited.json",
    compilerOptions: { noLib: true },
    include: ["src"],
  });
  const base = write("inherited.json", { extends: "vize-config-priority" });
  const pkg = write("node_modules/vize-config-priority/package.json", { tsconfig: "preferred" });
  const fallback = write("node_modules/vize-config-priority/tsconfig.json", {
    compilerOptions: { noFallthroughCasesInSwitch: false },
  });
  const preferred = path.join(root, "node_modules/vize-config-priority/preferred.json");
  const nativeRoot = path.join(root, "native");
  write("native/package.json", { name: "@vizejs/native", main: "index.cjs" });
  write("native/index.cjs", "module.exports = {};\n");
  fs.copyFileSync(
    require.resolve("../../npm/native/language-projection.js"),
    path.join(nativeRoot, "language-projection.js"),
  );
  const native = require(path.join(nativeRoot, "index.cjs"));
  native.generateVueLanguageCode = () =>
    JSON.stringify({ text: "export {};", extension: ".tsx", mappings: [], diagnostics: [] });
  native.getVueLanguageProjectMetadata = () => ({
    configIdentity: "unchanged-vue-options",
    watchedFiles: [config, base, pkg, fallback, preferred],
  });
  const parsed = ts.getParsedCommandLineOfConfigFile(config, undefined, {
    ...ts.sys,
    onUnRecoverableConfigFileDiagnostic: assert.fail,
  });
  const original = parsed.options;
  let currentOptions = original,
    version = 1;
  const configDescriptor = Object.getOwnPropertyDescriptor(original, "configFile");
  const prototype = { hostOption: true };
  Object.setPrototypeOf(original, prototype);
  const sentinel = Symbol("other plugin option");
  Object.defineProperty(original, sentinel, { value: "preserved", enumerable: false });
  const roots = parsed.fileNames;
  const references = [{ path: path.join(root, "referenced") }];
  const watches = new Map();
  const host = {
    ...ts.sys,
    useCaseSensitiveFileNames: () => ts.sys.useCaseSensitiveFileNames,
    getCompilationSettings: () => currentOptions,
    getScriptFileNames: () => roots,
    getProjectReferences: () => references,
    getProjectVersion: () => String(version),
    getScriptVersion: () => "1",
    getScriptSnapshot: (file) =>
      fs.existsSync(file) ? ts.ScriptSnapshot.fromString(fs.readFileSync(file, "utf8")) : undefined,
    getCurrentDirectory: () => root,
    getDefaultLibFileName: (options) => ts.getDefaultLibFilePath(options),
  };
  // References are owned by tsserver. This control records their identity without
  // loading a deliberately absent referenced project into the standalone service.
  const serviceHost = Object.create(host);
  serviceHost.getProjectReferences = () => undefined;
  const callbacks = {
    ...ts.sys,
    watchFile(file, callback) {
      watches.set(file, callback);
      return { close: () => watches.delete(file) };
    },
  };
  let parses = 0;
  const library = Object.create(ts, {
    getParsedCommandLineOfConfigFile: {
      value: (...args) => {
        parses++;
        return ts.getParsedCommandLineOfConfigFile(...args);
      },
    },
  });
  const support = installVueVirtualModules(library, {
    languageServiceHost: host,
    serverHost: callbacks,
    config: { nativePath: nativeRoot },
    project: {
      projectKind: ts.server.ProjectKind.Configured,
      getCurrentDirectory: () => root,
      setCompilerOptions(options) {
        currentOptions = options;
        version++;
      },
      refreshDiagnostics() {},
    },
  });
  const service = ts.createLanguageService(serviceHost);
  t.after(() => {
    service.dispose();
    support.dispose();
  });
  const body = () =>
    service
      .getSemanticDiagnostics(control)
      .map(({ code, start, length, messageText }) => ({ code, start, length, messageText }));
  const before = body();
  write("node_modules/vize-config-priority/preferred.json", {
    compilerOptions: { noFallthroughCasesInSwitch: true },
  });
  watches.get(preferred)(preferred, ts.FileWatcherEventKind.Created);
  const warm = body();
  const freshOptions = ts.getParsedCommandLineOfConfigFile(config, undefined, {
    ...ts.sys,
    onUnRecoverableConfigFileDiagnostic: assert.fail,
  }).options;
  const freshHost = Object.create(serviceHost);
  freshHost.getCompilationSettings = () => freshOptions;
  const fresh = ts.createLanguageService(freshHost);
  const expected = fresh
    .getSemanticDiagnostics(control)
    .map(({ code, start, length, messageText }) => ({ code, start, length, messageText }));
  fresh.dispose();
  assert.deepEqual(before, []);
  assert(expected.some(({ code }) => code === 7029));
  assert.deepEqual(warm, expected);
  for (let i = 0; i < 25; i++) {
    host.getCompilationSettings();
    host.getProjectVersion();
    body();
  }
  assert.equal(parses, 2, "one initial parse and one parse for the changed project closure");
  currentOptions = original;
  version++;
  assert.deepEqual(body(), expected, "reloadProjects cannot restore its stale cached options");
  assert.equal(parses, 2, "reapplying a fresh snapshot does not parse the project again");
  write("node_modules/vize-config-priority/preferred.json", { compilerOptions: {} });
  watches.get(preferred)(preferred, ts.FileWatcherEventKind.Changed);
  assert.deepEqual(body(), []);
  assert.equal(Object.hasOwn(host.getCompilationSettings(), "noFallthroughCasesInSwitch"), false);
  fs.unlinkSync(preferred);
  watches.get(preferred)(preferred, ts.FileWatcherEventKind.Deleted);
  assert.deepEqual(body(), []);
  assert.equal(host.getCompilationSettings().noFallthroughCasesInSwitch, false);
  assert.equal(parses, 4, "option removal and fallback selection each parse once");
  assert.equal(host.getScriptFileNames(), roots);
  assert.equal(host.getProjectReferences(), references);
  assert.equal(Object.getPrototypeOf(host.getCompilationSettings()), prototype);
  assert.deepEqual(
    Object.getOwnPropertyDescriptor(host.getCompilationSettings(), "configFile"),
    configDescriptor,
  );
  assert.equal(host.getCompilationSettings()[sentinel], "preserved");
  assert.equal(original.noFallthroughCasesInSwitch, false, "the original host object is untouched");
});
