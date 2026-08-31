import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);
// Prefer the TypeScript the VS Code extension bundles, but fall back to the
// workspace copy: editors/vscode is excluded from the pnpm workspace, so its
// node_modules is empty in CI while tests/node_modules pins the same version.
const ts = require(
  require.resolve("typescript", {
    paths: [path.join(root, "editors/vscode"), path.join(root, "tests")],
  }),
) as typeof import("typescript");
const initVuePlugin = require(
  path.join(root, "editors/vscode/typescript-vue-plugin/index.cjs"),
) as (modules: { typescript: typeof ts }) => {
  create(info: {
    languageService: import("typescript").LanguageService;
    languageServiceHost: import("typescript").LanguageServiceHost;
    serverHost: typeof ts.sys;
  }): import("typescript").LanguageService;
};

test("VS Code extension installs the guarded TypeScript Vue plugin", () => {
  const manifest = readJson<{
    contributes?: {
      typescriptServerPlugins?: Array<{
        enableForWorkspaceTypeScriptVersions?: boolean;
        name?: string;
      }>;
    };
    scripts?: Record<string, string>;
  }>("editors/vscode/package.json");

  assert.deepEqual(manifest.contributes?.typescriptServerPlugins, [
    {
      enableForWorkspaceTypeScriptVersions: true,
      name: "@vizejs/typescript-vue-plugin",
    },
  ]);
  assert.equal(manifest.scripts?.["vscode:prepublish"], "vp pack");
  assert.equal(manifest.scripts?.build, "vp pack");
  assert.equal(manifest.scripts?.watch, "vp pack --watch");
  assert.equal(
    manifest.scripts?.package,
    "rust-script ../../tools/commands/editors/vscode/sync-typescript-plugin.rs stage && vsce package --no-dependencies --out dist/vize.vsix && rust-script ../../tools/commands/editors/vscode/sync-typescript-plugin.rs inject dist/vize.vsix",
  );
  assert.equal(
    manifest.scripts?.["test:host"],
    "rust-script ../../tools/commands/editors/vscode/sync-typescript-plugin.rs stage && node test/run-extension-host.mjs",
  );

  assert.match(
    readFile("tools/config/vite-plus/tasks/build.ts"),
    /rustToolFromVscodeExtension\(\s*"editors\/vscode\/sync-typescript-plugin"/,
  );
  assert.match(
    readFile("tools/config/vite-plus/tasks/build.ts"),
    /rustToolFromVscodeExtension\(\s*"editors\/vscode\/assert-vsix-package"/,
  );
  assert.match(
    readFile("tools/config/vite-plus/tasks/test-benchmark.ts"),
    /rustToolFromVscodeExtension\(\s*"editors\/vscode\/sync-typescript-plugin"/,
  );
  assert.match(
    readFile("tools/config/vite-plus/tasks/test-benchmark.ts"),
    /rustToolFromVscodeExtension\(\s*"editors\/vscode\/assert-vsix-package"/,
  );

  assert.match(readFile("editors/vscode/.vscodeignore"), /^typescript-vue-plugin\/$/m);
  assert.ok(
    fs.existsSync(path.join(root, "editors/vscode/typescript-vue-plugin/index.cjs")),
    "plugin source is synchronized into node_modules/@vizejs/typescript-vue-plugin for VS Code",
  );
});

test("VSIX plugin injection resolves relative archive paths before changing directory", () => {
  const source = readFile("tools/commands/editors/vscode/sync-typescript-plugin.rs");

  assert.match(source, /let vsix_path = absolute_from_cwd\(vsix_path\)\?/);
  assert.match(source, /fn absolute_from_cwd\(path: &Path\) -> Result<PathBuf, String>/);
  assert.match(source, /\.current_dir\(&temp_dir\)/);
  assert.match(source, /\.arg\(&vsix_path\)/);
});

test("TypeScript Vue plugin resolves existing relative .vue imports", () => {
  const project = createVueProject('import App from "./app.vue";\nApp;\n', {
    "app.vue": "<template />\n",
  });
  try {
    const withoutPlugin = collectDiagnostics(project.mainTs, false);
    assert.ok(hasCannotFindVueModule(withoutPlugin, "./app.vue"), formatDiagnostics(withoutPlugin));

    const withPlugin = collectDiagnostics(project.mainTs, true);
    assert.equal(
      hasCannotFindVueModule(withPlugin, "./app.vue"),
      false,
      formatDiagnostics(withPlugin),
    );
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin maps .ts definition results back to real .vue files", () => {
  const project = createVueProject('import App from "./app.vue";\nApp;\n', {
    "app.vue": "<template />\n",
  });
  try {
    const service = createLanguageService(project.mainTs, true);
    const source = fs.readFileSync(project.mainTs, "utf8");
    const vuePath = path.join(path.dirname(project.mainTs), "app.vue");

    for (const position of [
      source.indexOf("App from") + 1,
      source.indexOf("./app.vue") + 2,
      source.indexOf("App;") + 1,
    ]) {
      const definitions = service.getDefinitionAtPosition(project.mainTs, position);
      assert.equal(definitions?.[0]?.fileName, vuePath);
      assert.deepEqual(definitions?.[0]?.textSpan, { start: 0, length: 0 });

      const bound = service.getDefinitionAndBoundSpan(project.mainTs, position);
      assert.equal(bound?.definitions?.[0]?.fileName, vuePath);
      assert.deepEqual(bound?.definitions?.[0]?.textSpan, { start: 0, length: 0 });
    }
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin filters existing .vue import diagnostics without patching host resolution", () => {
  const project = createVueProject('import App from "./app.vue";\nApp;\n', {
    "app.vue": "<template />\n",
  });
  try {
    const host = createHost(path.dirname(path.dirname(project.mainTs)), project.mainTs);
    Object.preventExtensions(host);
    const service = ts.createLanguageService(host);
    const plugin = initVuePlugin({ typescript: ts });
    const wrapped = plugin.create({
      languageService: service,
      languageServiceHost: host,
      serverHost: ts.sys,
    });

    assert.equal(Reflect.get(host, "resolveModuleNameLiterals"), undefined);
    assert.equal(Reflect.get(host, "resolveModuleNames"), undefined);

    const diagnostics = wrapped.getSemanticDiagnostics(project.mainTs);
    assert.equal(
      hasCannotFindVueModule(diagnostics, "./app.vue"),
      false,
      formatDiagnostics(diagnostics),
    );
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin annotates .ts hover info for .vue imports", () => {
  const project = createVueProject('import App from "./app.vue";\nApp;\n', {
    "app.vue": "<template />\n",
  });
  try {
    const service = createLanguageService(project.mainTs, true);
    const source = fs.readFileSync(project.mainTs, "utf8");

    for (const position of [source.indexOf("App from") + 1, source.indexOf("App;") + 1]) {
      const quickInfo = service.getQuickInfoAtPosition(project.mainTs, position);
      assert.equal(displayPartsText(quickInfo?.documentation), "Vue component: app.vue");
    }
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin keeps missing .vue imports diagnostic", () => {
  const project = createVueProject('import Missing from "./missing.vue";\nMissing;\n', {});
  try {
    const diagnostics = collectDiagnostics(project.mainTs, true);
    assert.ok(hasCannotFindVueModule(diagnostics, "./missing.vue"), formatDiagnostics(diagnostics));
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin keeps plain .ts projects crash-free without host resolver patches", () => {
  const project = createVueProject('import value from "./missing";\nvalue;\n', {});
  try {
    const service = createLanguageService(project.mainTs, true);

    assert.doesNotThrow(() => service.getSemanticDiagnostics(project.mainTs));
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin does not require a patchable tsserver host", () => {
  const project = createVueProject("const value = 1;\nvalue;\n", {});
  try {
    const host = createHost(path.dirname(path.dirname(project.mainTs)), project.mainTs);
    Object.freeze(host);
    const service = ts.createLanguageService(host);
    const plugin = initVuePlugin({ typescript: ts });
    let wrapped: import("typescript").LanguageService | undefined;

    assert.doesNotThrow(() => {
      wrapped = plugin.create({
        languageService: service,
        languageServiceHost: host,
        serverHost: ts.sys,
      });
    });
    assert.notEqual(wrapped, service);
    assert.doesNotThrow(() => wrapped?.getSemanticDiagnostics(project.mainTs));
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin leaves host resolver calls untouched", () => {
  const project = createVueProject('import App from "./app.vue";\nApp;\n', {
    "app.vue": "<template />\n",
  });
  try {
    const host = createHost(path.dirname(path.dirname(project.mainTs)), project.mainTs);
    Object.preventExtensions(host);
    const service = ts.createLanguageService(host);
    const plugin = initVuePlugin({ typescript: ts });

    plugin.create({
      languageService: service,
      languageServiceHost: host,
      serverHost: ts.sys,
    });

    assert.doesNotThrow(() => {
      assert.equal(Reflect.get(host, "resolveModuleNameLiterals"), undefined);
      assert.equal(Reflect.get(host, "resolveModuleNames"), undefined);
    });
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

function createVueProject(mainSource: string, vueFiles: Record<string, string>) {
  const rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vscode-vue-plugin-"));
  const srcDir = path.join(rootDir, "src");
  fs.mkdirSync(srcDir, { recursive: true });
  const mainTs = path.join(srcDir, "main.ts");
  fs.writeFileSync(mainTs, mainSource);
  for (const [fileName, content] of Object.entries(vueFiles)) {
    fs.writeFileSync(path.join(srcDir, fileName), content);
  }
  return { root: rootDir, mainTs };
}

function collectDiagnostics(mainTs: string, enablePlugin: boolean) {
  return createLanguageService(mainTs, enablePlugin).getSemanticDiagnostics(mainTs);
}

function createLanguageService(
  mainTs: string,
  enablePlugin: boolean,
  configureHost?: (host: import("typescript").LanguageServiceHost) => void,
) {
  const host = createHost(path.dirname(path.dirname(mainTs)), mainTs);
  configureHost?.(host);
  const service = ts.createLanguageService(host);
  return enablePlugin
    ? initVuePlugin({ typescript: ts }).create({
        languageService: service,
        languageServiceHost: host,
        serverHost: ts.sys,
      })
    : service;
}

function createHost(rootDir: string, mainTs: string): import("typescript").LanguageServiceHost {
  const options: import("typescript").CompilerOptions = {
    allowSyntheticDefaultImports: true,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    noEmit: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
  };

  return {
    directoryExists: (directoryName) => ts.sys.directoryExists(directoryName),
    fileExists: (fileName) => ts.sys.fileExists(fileName),
    getCompilationSettings: () => options,
    getCurrentDirectory: () => rootDir,
    getDefaultLibFileName: (compilerOptions) => ts.getDefaultLibFilePath(compilerOptions),
    getDirectories: (directoryName) => ts.sys.getDirectories(directoryName),
    getScriptFileNames: () => [mainTs],
    getScriptSnapshot(fileName) {
      if (!fs.existsSync(fileName)) return undefined;
      return ts.ScriptSnapshot.fromString(fs.readFileSync(fileName, "utf8"));
    },
    getScriptVersion: () => "0",
    readDirectory: (rootDir, extensions, excludes, includes, depth) =>
      ts.sys.readDirectory(rootDir, extensions, excludes, includes, depth),
    readFile: (fileName, encoding) => ts.sys.readFile(fileName, encoding),
    useCaseSensitiveFileNames: () => ts.sys.useCaseSensitiveFileNames,
  };
}

function hasCannotFindVueModule(
  diagnostics: readonly import("typescript").Diagnostic[],
  specifier: string,
) {
  return diagnostics.some((diagnostic) => {
    const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n");
    return message.includes(`Cannot find module '${specifier}'`);
  });
}

function formatDiagnostics(diagnostics: readonly import("typescript").Diagnostic[]) {
  return diagnostics
    .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"))
    .join("\n");
}

function displayPartsText(parts: readonly import("typescript").SymbolDisplayPart[] | undefined) {
  return parts?.map((part) => part.text).join("");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(readFile(relativePath)) as T;
}

function readFile(relativePath: string) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}
