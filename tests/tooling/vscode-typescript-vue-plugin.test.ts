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

test("VS Code extension ships the TypeScript Vue plugin", () => {
  const manifest = readJson<{
    contributes?: {
      typescriptServerPlugins?: Array<{
        enableForWorkspaceTypeScriptVersions?: boolean;
        name?: string;
      }>;
    };
    scripts?: Record<string, string>;
  }>("editors/vscode/package.json");
  const plugin = manifest.contributes?.typescriptServerPlugins?.find(
    (entry) => entry.name === "@vizejs/typescript-vue-plugin",
  );
  const stageScript = "node ../../tools/vscode-vize/sync-typescript-plugin.mjs stage && vp pack";

  assert.ok(plugin, "manifest should contribute the Vue TypeScript plugin");
  assert.equal(plugin.enableForWorkspaceTypeScriptVersions, true);
  assert.equal(manifest.scripts?.["vscode:prepublish"], stageScript);
  assert.equal(manifest.scripts?.build, stageScript);
  assert.equal(manifest.scripts?.watch, `${stageScript} --watch`);
  assert.equal(
    manifest.scripts?.package,
    "vsce package --no-dependencies --out dist/vize.vsix && node ../../tools/vscode-vize/sync-typescript-plugin.mjs inject dist/vize.vsix",
  );
  assert.ok(
    fs.existsSync(path.join(root, "editors/vscode/typescript-vue-plugin/index.cjs")),
    "plugin package source must exist for the local file dependency",
  );
  assert.match(readFile("editors/vscode/.vscodeignore"), /^typescript-vue-plugin\/$/m);
  assert.match(
    readFile("tools/vite-plus/tasks/build.ts"),
    /package:vscode-extension[\s\S]*vscode-typescript-vue-plugin\.test\.ts/,
  );
  assert.match(
    readFile("tools/vite-plus/tasks/build.ts"),
    /package:editor-extensions[\s\S]*vscode-typescript-vue-plugin\.test\.ts/,
  );
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

test("TypeScript Vue plugin keeps missing .vue imports diagnostic", () => {
  const project = createVueProject('import Missing from "./missing.vue";\nMissing;\n', {});
  try {
    const diagnostics = collectDiagnostics(project.mainTs, true);
    assert.ok(hasCannotFindVueModule(diagnostics, "./missing.vue"), formatDiagnostics(diagnostics));
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
  const host = createHost(path.dirname(path.dirname(mainTs)), mainTs);
  const service = ts.createLanguageService(host);
  const activeService = enablePlugin
    ? initVuePlugin({ typescript: ts }).create({
        languageService: service,
        languageServiceHost: host,
        serverHost: ts.sys,
      })
    : service;

  return activeService.getSemanticDiagnostics(mainTs);
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

function readJson<T>(relativePath: string): T {
  return JSON.parse(readFile(relativePath)) as T;
}

function readFile(relativePath: string) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}
