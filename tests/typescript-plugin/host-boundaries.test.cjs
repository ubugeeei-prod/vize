const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const ts = require("../node_modules/typescript");

const root = path.resolve(__dirname, "../..");
const pluginSource = path.join(root, "npm/typescript-plugin");

function temporary(t) {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-ts-host-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

function write(directory, files) {
  for (const [name, text] of Object.entries(files)) {
    const file = path.join(directory, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, typeof text === "string" ? text : JSON.stringify(text));
  }
}

// A provider with the real UTF-16 helper and a fixed generated module.
function fakeNative(directory) {
  write(directory, {
    "package.json": { name: "@vizejs/native", version: "0.0.0", main: "index.js" },
    "index.js": `exports.generateVueLanguageCode = () => JSON.stringify({ text: "export {};\\n", extension: ".tsx", mappings: [], diagnostics: [] });`,
    "language-projection.js": `module.exports = require(${JSON.stringify(path.join(root, "npm/native/language-projection.js"))});`,
  });
  return directory;
}

void test("an absent default provider leaves tsserver and its external files untouched", (t) => {
  // A copied plugin has no node_modules, like the universal VSIX.
  const plugin = path.join(temporary(t), "plugin");
  fs.cpSync(pluginSource, plugin, {
    recursive: true,
    filter: (name) => !/node_modules/.test(name),
  });
  const project = temporary(t);
  write(project, { "tsconfig.json": { include: ["src"] }, "src/App.vue": "<template />" });
  const languageService = { getSemanticDiagnostics: () => [] };
  const host = { getCompilationSettings: () => ({}), readFile: () => "", fileExists: () => true };
  host.getScriptSnapshot = () => undefined;
  const keys = Object.keys(host);
  const tsProject = {
    getCompilerOptions: () => ({ configFilePath: path.join(project, "tsconfig.json") }),
  };
  const instance = require(path.join(plugin, "index.cjs"))({ typescript: ts });
  const created = instance.create({
    languageService,
    languageServiceHost: host,
    serverHost: ts.sys,
    project: tsProject,
  });
  assert.equal(created, languageService);
  assert.deepEqual(Object.keys(host), keys);
  assert.deepEqual(instance.getExternalFiles(tsProject, 2), []);
});

void test("external files honor inherited opt-outs and reuse unchanged root names", (t) => {
  const project = temporary(t);
  write(project, {
    "base.json": {
      compilerOptions: { plugins: [{ name: "@vizejs/typescript-vue-plugin", enabled: false }] },
    },
    "disabled.json": { extends: "./base.json", include: ["src"] },
    "tsconfig.json": { include: ["src"] },
    "src/App.vue": "<template />",
  });
  const plugin = require(path.join(pluginSource, "index.cjs"))({ typescript: ts });
  const disabled = { getCompilerOptions: () => ({ configFilePath: `${project}/disabled.json` }) };
  assert.deepEqual(plugin.getExternalFiles(disabled, 2), []);
  const enabled = { getCompilerOptions: () => ({ configFilePath: `${project}/tsconfig.json` }) };
  assert.deepEqual(plugin.getExternalFiles(enabled, 2), [`${project}/src/App.vue`]);
  write(project, { "src/Next.vue": "<template />" });
  assert.deepEqual(plugin.getExternalFiles(enabled, 0), [`${project}/src/App.vue`]);
  assert.deepEqual(plugin.getExternalFiles(enabled, 1), [
    `${project}/src/App.vue`,
    `${project}/src/Next.vue`,
  ]);
});

void test("SFC projections read the project configuration once per project version", (t) => {
  const { installVueVirtualModules } = require(path.join(pluginSource, "virtual-modules.cjs"));
  const project = temporary(t);
  const config = `${project}/tsconfig.json`;
  write(project, { "tsconfig.json": {}, "A.vue": "<template />", "B.vue": "<template />" });
  let version = 1;
  const reads = [];
  const host = {
    getProjectVersion: () => String(version),
    getCompilationSettings: () => ({ configFilePath: config }),
    getScriptSnapshot: () => undefined,
    fileExists: (name) => fs.existsSync(name),
    readFile(name) {
      reads.push(name);
      return fs.readFileSync(name, "utf8");
    },
  };
  const support = installVueVirtualModules(ts, {
    languageServiceHost: host,
    serverHost: ts.sys,
    config: { nativePath: fakeNative(path.join(project, "native")) },
  });
  // Project configuration watchers must not outlive the test.
  t.after(() => support.dispose?.());
  for (const file of ["A.vue", "B.vue", "A.vue"]) assert(support.get(`${project}/${file}`));
  assert.equal(reads.filter((name) => name === config).length, 1);
  version++;
  support.get(`${project}/B.vue`);
  assert.equal(reads.filter((name) => name === config).length, 2);
});

void test("resolution keeps host results, project host methods and NodeNext import conditions", (t) => {
  const { installVueVirtualModules } = require(path.join(pluginSource, "virtual-modules.cjs"));
  const project = temporary(t);
  write(project, {
    "package.json": { type: "module" },
    "src/main.ts":
      'import Comp from "lib/Comp.vue";\nimport dep from "dep";\nimport App from "./App.vue";\n',
    "src/App.vue": "<template />",
    "node_modules/lib/package.json": {
      name: "lib",
      // Packages may publish declarations for their SFC subpaths per condition.
      exports: { "./Comp.vue": { import: "./esm/Comp.d.ts", require: "./cjs/Comp.d.ts" } },
    },
    "node_modules/lib/esm/Comp.d.ts": "export default 1;",
    "node_modules/lib/cjs/Comp.d.ts": "export default 1;",
    "node_modules/dep/package.json": { name: "dep", types: "index.d.ts" },
    "node_modules/dep/index.d.ts": "export default 1;",
  });
  const options = {
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    types: [],
  };
  const probes = [];
  const sentinel = { resolvedFileName: "/host/dep.d.ts", extension: ts.Extension.Dts };
  class ProjectHost {
    getCompilationSettings() {
      return options;
    }
    getCurrentDirectory() {
      return project;
    }
    directoryExists(name) {
      probes.push(name);
      return ts.sys.directoryExists(name);
    }
    getScriptSnapshot() {
      return undefined;
    }
    fileExists(name) {
      return ts.sys.fileExists(name);
    }
    readFile(name) {
      return ts.sys.readFile(name);
    }
    resolveModuleNameLiterals(literals) {
      return literals.map(() => ({ resolvedModule: sentinel }));
    }
  }
  const host = new ProjectHost();
  installVueVirtualModules(ts, {
    languageServiceHost: host,
    serverHost: { ...ts.sys, directoryExists: () => assert.fail("serverHost probe") },
    config: { nativePath: fakeNative(path.join(project, "native")) },
  });
  const main = `${project}/src/main.ts`;
  const sourceFile = ts.createSourceFile(
    main,
    ts.sys.readFile(main),
    {
      languageVersion: ts.ScriptTarget.Latest,
      impliedNodeFormat: ts.ModuleKind.ESNext,
    },
    true,
  );
  const literals = sourceFile.statements.map((statement) => statement.moduleSpecifier);
  const [component, dependency, local] = host.resolveModuleNameLiterals(
    literals,
    main,
    undefined,
    options,
    sourceFile,
    undefined,
  );
  assert.equal(
    component.resolvedModule.resolvedFileName,
    `${project}/node_modules/lib/esm/Comp.d.ts`,
  );
  assert.equal(dependency.resolvedModule, sentinel);
  assert.equal(local.resolvedModule.resolvedFileName, `${project}/src/App.vue`);
  assert(probes.length > 0);
});
