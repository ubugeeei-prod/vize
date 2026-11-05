const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const ts = require("../node_modules/typescript");

const root = path.resolve(__dirname, "../..");
const pluginSource = path.join(root, "npm/typescript-plugin");

function write(directory, files) {
  for (const [name, text] of Object.entries(files)) {
    const file = path.join(directory, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, text);
  }
}

function fakeNative(directory) {
  write(directory, {
    "package.json": JSON.stringify({ name: "@vizejs/native", version: "0.0.0", main: "index.js" }),
    "index.js": "exports.generateVueLanguageCode = () => { throw new Error('unset'); };\n",
  });
  fs.copyFileSync(
    path.join(root, "npm/native/language-projection.js"),
    path.join(directory, "language-projection.js"),
  );
  return require(path.join(directory, "index.js"));
}

void test("unrequested SFC snapshots stay stubs until a query names them", (t) => {
  const project = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-lazy-sfc-")));
  t.after(() => fs.rmSync(project, { recursive: true, force: true }));
  const files = {
    "src/App.vue": "<template>app</template>\n",
    "src/Child.vue": "<template>child</template>\n",
    "src/Parent.vue": '<script setup lang="ts">\nimport Child from "./Child.vue";\n</script>\n',
    "src/components.d.ts":
      'export interface Globals { app: import("./App.vue").default; child: import("./Child.vue").default }\n',
    "src/main.ts": "export const ready = true;\n",
  };
  write(project, files);
  const abs = (name) => path.join(project, name);
  const native = fakeNative(path.join(project, "native"));
  const generated = [];
  native.generateVueLanguageCode = (_source, request) => {
    generated.push(request.filename);
    return JSON.stringify({
      // Carry the parent's real dependency in its canonical generated module.
      text:
        request.filename === abs("src/Parent.vue")
          ? 'import Child from "./Child.vue";\nexport default Child;\n'
          : "export default 1;\n",
      extension: ".tsx",
      mappings: [],
      diagnostics: [],
    });
  };
  const { installVueVirtualModules } = require(path.join(pluginSource, "virtual-modules.cjs"));
  const { createMappedLanguageService } = require(path.join(pluginSource, "language-service.cjs"));
  let updates = 0;
  const host = {
    getCompilationSettings: () => ({ noLib: true, module: ts.ModuleKind.ESNext }),
    getScriptFileNames: () => Object.keys(files).map(abs),
    getScriptVersion: () => "1",
    getScriptSnapshot: (name) =>
      fs.existsSync(name) ? ts.ScriptSnapshot.fromString(fs.readFileSync(name, "utf8")) : undefined,
    fileExists: (name) => fs.existsSync(name),
    readFile: (name) => (fs.existsSync(name) ? fs.readFileSync(name, "utf8") : undefined),
    getCurrentDirectory: () => project,
    getDefaultLibFileName: () => "/lib.d.ts",
  };
  const support = installVueVirtualModules(ts, {
    languageServiceHost: host,
    serverHost: ts.sys,
    config: { nativePath: path.join(project, "native") },
    project: {
      updateGraph() {
        updates++;
      },
    },
  });
  t.after(() => support.dispose());
  const text = (name) => host.getScriptSnapshot(abs(name))?.getText(0, 12);
  assert.equal(text("src/App.vue"), "export {};\n");
  assert.equal(text("src/Child.vue"), "export {};\n");
  assert.deepEqual(generated, []);
  support.ensure(abs("src/components.d.ts"));
  assert.deepEqual(generated, []);
  const service = createMappedLanguageService(ts, ts.createLanguageService(host), support);
  service.getSemanticDiagnostics(abs("src/components.d.ts"));
  assert.deepEqual(generated, [], "declaration imports do not project every component");
  assert.equal(updates, 0);
  service.getSemanticDiagnostics(abs("src/Parent.vue"));
  assert.equal(updates, 1);
  assert.deepEqual(
    generated.slice().sort((a, b) => a.localeCompare(b)),
    [abs("src/Child.vue"), abs("src/Parent.vue")].sort((a, b) => a.localeCompare(b)),
  );
  assert.equal(text("src/App.vue"), "export {};\n");
  assert.equal(support.get(abs("src/App.vue")).projection.text, "export default 1;\n");
  assert.deepEqual(
    generated.slice().sort((a, b) => a.localeCompare(b)),
    [abs("src/App.vue"), abs("src/Child.vue"), abs("src/Parent.vue")].sort((a, b) =>
      a.localeCompare(b),
    ),
  );
});

void test("cold TypeScript diagnostics promote only resolved SFC dependencies and refresh cached stubs", (t) => {
  const project = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-demand-types-")));
  t.after(() => fs.rmSync(project, { recursive: true, force: true }));
  const mainText = [
    'import App from "@/App.vue";',
    'import Parent from "./Parent.vue";',
    'import { BarrelApp } from "./barrel";',
    'import { PackageApp } from "ui";',
    'import Declared from "./Declared.vue";',
    'import Missing from "./Missing.vue";',
    'const bait = "./Unused.vue";',
    "const direct: string = App;",
    "const parent: string = Parent;",
    "const barrel: string = BarrelApp;",
    "const packaged: string = PackageApp;",
    "const positive: number = App;",
    "const declared: string = Declared;",
  ].join("\n");
  const files = {
    "src/main.ts": mainText,
    "src/barrel.ts":
      'export { default as BarrelApp } from "@/App.vue";\nexport * from "./cycle";\n',
    "src/cycle.ts": 'export * from "./barrel";\n',
    "src/globals.d.ts": 'export type Globals = typeof import("./Unused.vue");\n',
    "src/App.vue": '<template>import Bait from "./Unused.vue";</template>\n',
    "src/Parent.vue": '<script setup>import Child from "./Child.vue";</script>\n',
    "src/Child.vue": "<template />\n",
    "src/Unused.vue": "<template />\n",
    "src/Declared.vue": "<template />\n",
    "src/Declared.d.vue.ts": "declare const declared: string;\nexport default declared;\n",
    "node_modules/ui/package.json": JSON.stringify({
      name: "ui",
      version: "1.0.0",
      types: "index.ts",
    }),
    "node_modules/ui/index.ts": 'export { default as PackageApp } from "./Package.vue";\n',
    "node_modules/ui/Package.vue": "<template />\n",
    "src/import.mts": 'import Selected from "conditional";\nconst selected: string = Selected;\n',
    "src/require.cts":
      'import Selected = require("conditional");\nconst selected: string = Selected.default;\n',
    "node_modules/conditional/package.json": JSON.stringify({
      name: "conditional",
      version: "1.0.0",
      exports: { ".": { import: "./import.mts", require: "./require.cts" } },
    }),
    "node_modules/conditional/import.mts": 'export { default } from "./Import.vue";\n',
    "node_modules/conditional/require.cts": 'export { default } from "./Require.vue";\n',
    "node_modules/conditional/Import.vue": "<template />\n",
    "node_modules/conditional/Require.vue": "<template />\n",
  };
  write(project, files);
  const abs = (name) => path.join(project, name);
  const native = fakeNative(path.join(project, "native"));
  const generated = [];
  native.generateVueLanguageCode = (_source, request) => {
    generated.push(request.filename);
    return JSON.stringify({
      text:
        request.filename === abs("src/Parent.vue")
          ? 'export { default } from "./Child.vue";\n'
          : "export default 1;\n",
      extension: ".tsx",
      mappings: [],
      diagnostics: [],
    });
  };
  const options = {
    noLib: true,
    strict: true,
    types: [],
    allowArbitraryExtensions: true,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    baseUrl: project,
    paths: { "@/*": ["src/*"] },
  };
  let main = mainText,
    mainVersion = 1,
    projectVersion = 1;
  const host = {
    getCompilationSettings: () => options,
    getScriptFileNames: () =>
      Object.keys(files)
        .filter((name) => !name.startsWith("node_modules/"))
        .map(abs),
    getProjectVersion: () => String(projectVersion),
    getScriptVersion: (name) => (name === abs("src/main.ts") ? String(mainVersion) : "1"),
    getScriptSnapshot: (name) => {
      const source = name === abs("src/main.ts") ? main : ts.sys.readFile(name);
      return source === undefined ? undefined : ts.ScriptSnapshot.fromString(source);
    },
    fileExists: ts.sys.fileExists.bind(ts.sys),
    readFile: ts.sys.readFile.bind(ts.sys),
    directoryExists: ts.sys.directoryExists.bind(ts.sys),
    getCurrentDirectory: () => project,
    getDefaultLibFileName: () => "/lib.d.ts",
  };
  const { installVueVirtualModules } = require(path.join(pluginSource, "virtual-modules.cjs"));
  const { createMappedLanguageService } = require(path.join(pluginSource, "language-service.cjs"));
  const support = installVueVirtualModules(ts, {
    languageServiceHost: host,
    serverHost: ts.sys,
    config: { nativePath: path.join(project, "native") },
  });
  const raw = ts.createLanguageService(host);
  const service = createMappedLanguageService(ts, raw, support);
  t.after(() => service.dispose());
  raw.getProgram();
  assert.deepEqual(generated, [], "loading all external roots never generates a component");
  const initial = host.getProjectVersion();
  const diagnostics = () =>
    service.getSemanticDiagnostics(abs("src/main.ts")).map((value) => ({
      code: value.code,
      category: value.category,
      start: value.start,
      length: value.length,
      message: ts.flattenDiagnosticMessageText(value.messageText, "\n"),
      related: value.relatedInformation,
    }));
  const assignment = (name) => ({
    code: 2322,
    category: ts.DiagnosticCategory.Error,
    start: mainText.indexOf(`const ${name}:`) + 6,
    length: name.length,
    message: "Type 'number' is not assignable to type 'string'.",
    related: undefined,
  });
  assert.deepEqual(diagnostics(), [
    {
      code: 2307,
      category: ts.DiagnosticCategory.Error,
      start: mainText.indexOf('"./Missing.vue"'),
      length: '"./Missing.vue"'.length,
      message: "Cannot find module './Missing.vue' or its corresponding type declarations.",
      related: undefined,
    },
    ...["direct", "parent", "barrel", "packaged"].map(assignment),
  ]);
  assert.notEqual(host.getProjectVersion(), initial);
  const expected = ["src/App.vue", "src/Parent.vue", "src/Child.vue", "node_modules/ui/Package.vue"]
    .map(abs)
    .sort((a, b) => a.localeCompare(b));
  assert.deepEqual(
    generated.slice().sort((a, b) => a.localeCompare(b)),
    expected,
  );
  assert.equal(host.getScriptSnapshot(abs("src/Unused.vue")).getText(0, 12), "export {};\n");
  const prepared = host.getProjectVersion();
  diagnostics();
  assert.equal(host.getProjectVersion(), prepared);
  assert.deepEqual(
    generated.slice().sort((a, b) => a.localeCompare(b)),
    expected,
  );
  main = 'import Added from "./Unused.vue";\nconst added: string = Added;\n';
  mainVersion++;
  const updated = diagnostics();
  assert.deepEqual(updated, [{ ...assignment("added"), start: main.indexOf("const added:") + 6 }]);
  assert.deepEqual(
    generated.slice().sort((a, b) => a.localeCompare(b)),
    [...expected, abs("src/Unused.vue")].sort((a, b) => a.localeCompare(b)),
  );
  options.module = ts.ModuleKind.NodeNext;
  options.moduleResolution = ts.ModuleResolutionKind.NodeNext;
  projectVersion++;
  for (const [file, component] of [
    ["src/import.mts", "Import.vue"],
    ["src/require.cts", "Require.vue"],
  ]) {
    const text = files[file];
    const actual = service.getSemanticDiagnostics(abs(file)).map((value) => ({
      code: value.code,
      category: value.category,
      start: value.start,
      length: value.length,
      message: ts.flattenDiagnosticMessageText(value.messageText, "\n"),
      related: value.relatedInformation,
    }));
    assert.deepEqual(actual, [
      { ...assignment("selected"), start: text.indexOf("const selected:") + 6 },
    ]);
    expected.push(abs(`node_modules/conditional/${component}`));
    assert.deepEqual(
      generated.slice().sort((a, b) => a.localeCompare(b)),
      [...expected, abs("src/Unused.vue")].sort((a, b) => a.localeCompare(b)),
    );
  }
});

void test("external file discovery skips dependency and build directories", () => {
  const project = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-prune-sfc-")));
  try {
    write(project, {
      "tsconfig.json": JSON.stringify({ include: ["**/*"] }),
      "src/App.vue": "<template />\n",
      "src/kept/Ok.vue": "<template />\n",
      "node_modules/pkg/Hidden.vue": "<template />\n",
      "src/node_modules/Nested.vue": "<template />\n",
      ".output/Server.vue": "<template />\n",
      "dist/Built.vue": "<template />\n",
      ".git/secret.vue": "<template />\n",
    });
    const plugin = require(path.join(pluginSource, "index.cjs"))({ typescript: ts });
    const found = plugin.getExternalFiles(
      { getCompilerOptions: () => ({ configFilePath: path.join(project, "tsconfig.json") }) },
      2,
    );
    assert.deepEqual(
      found.slice().sort((a, b) => a.localeCompare(b)),
      [path.join(project, "src/App.vue"), path.join(project, "src/kept/Ok.vue")].sort((a, b) =>
        a.localeCompare(b),
      ),
    );
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
});
