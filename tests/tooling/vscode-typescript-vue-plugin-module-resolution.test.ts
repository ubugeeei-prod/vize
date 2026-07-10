import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);
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

test("TypeScript Vue plugin resolves relative .vue imports from TSX under NodeNext", () => {
  const project = createVueProject(
    'import Card from "./Card.vue";\nconst render = () => <Card label="ok" />;\nrender;\n',
    {
      "Card.vue":
        '<script setup lang="ts">\ndefineProps<{ label: string }>();\n</script>\n<template />\n',
    },
    {
      jsx: ts.JsxEmit.Preserve,
      jsxImportSource: "vue",
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
    },
    "main.tsx",
  );
  try {
    const withoutPlugin = collectDiagnostics(project.mainTs, false, project.compilerOptions);
    assert.ok(
      hasCannotFindVueModule(withoutPlugin, "./Card.vue"),
      formatDiagnostics(withoutPlugin),
    );

    const withPlugin = collectDiagnostics(project.mainTs, true, project.compilerOptions);
    assert.equal(
      hasCannotFindVueModule(withPlugin, "./Card.vue"),
      false,
      formatDiagnostics(withPlugin),
    );

    const service = createLanguageService(project.mainTs, true, project.compilerOptions);
    const source = fs.readFileSync(project.mainTs, "utf8");
    const definition = service.getDefinitionAtPosition(
      project.mainTs,
      source.indexOf("./Card.vue") + 2,
    );
    assert.equal(definition?.[0]?.fileName, path.join(path.dirname(project.mainTs), "Card.vue"));
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin does not hide unresolved bare .vue subpath imports", () => {
  const project = createVueProject('import Remote from "design-system/Card.vue";\nRemote;\n', {});
  try {
    const diagnostics = collectDiagnostics(project.mainTs, true, project.compilerOptions);
    assert.ok(
      hasCannotFindVueModule(diagnostics, "design-system/Card.vue"),
      formatDiagnostics(diagnostics),
    );
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

function createVueProject(
  mainSource: string,
  vueFiles: Record<string, string>,
  compilerOptions: import("typescript").CompilerOptions = {},
  entryFileName = "main.ts",
) {
  const rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vscode-vue-plugin-"));
  const srcDir = path.join(rootDir, "src");
  fs.mkdirSync(srcDir, { recursive: true });
  const mainTs = path.join(srcDir, entryFileName);
  fs.writeFileSync(mainTs, mainSource);
  for (const [fileName, content] of Object.entries(vueFiles)) {
    fs.writeFileSync(path.join(srcDir, fileName), content);
  }
  return { compilerOptions, root: rootDir, mainTs };
}

function collectDiagnostics(
  mainTs: string,
  enablePlugin: boolean,
  compilerOptions: import("typescript").CompilerOptions,
) {
  return createLanguageService(mainTs, enablePlugin, compilerOptions).getSemanticDiagnostics(
    mainTs,
  );
}

function createLanguageService(
  mainTs: string,
  enablePlugin: boolean,
  compilerOptions: import("typescript").CompilerOptions,
) {
  const host = createHost(path.dirname(path.dirname(mainTs)), mainTs, compilerOptions);
  const service = ts.createLanguageService(host);
  return enablePlugin
    ? initVuePlugin({ typescript: ts }).create({
        languageService: service,
        languageServiceHost: host,
        serverHost: ts.sys,
      })
    : service;
}

function createHost(
  rootDir: string,
  mainTs: string,
  compilerOptions: import("typescript").CompilerOptions,
): import("typescript").LanguageServiceHost {
  const options: import("typescript").CompilerOptions = {
    allowSyntheticDefaultImports: true,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    noEmit: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
    ...compilerOptions,
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
