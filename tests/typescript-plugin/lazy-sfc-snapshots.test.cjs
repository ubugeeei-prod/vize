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
      text: "export default 1;\n",
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
  assert.deepEqual(generated.slice().sort(), [abs("src/Child.vue"), abs("src/Parent.vue")].sort());
  assert.equal(text("src/App.vue"), "export {};\n");
  assert.equal(support.get(abs("src/App.vue")).projection.text, "export default 1;\n");
  assert.deepEqual(
    generated.slice().sort(),
    [abs("src/App.vue"), abs("src/Child.vue"), abs("src/Parent.vue")].sort(),
  );
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
      found.slice().sort(),
      [path.join(project, "src/App.vue"), path.join(project, "src/kept/Ok.vue")].sort(),
    );
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
});
