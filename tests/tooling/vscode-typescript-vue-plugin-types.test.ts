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
    project: { getCurrentDirectory(): string };
    serverHost: typeof ts.sys;
  }): import("typescript").LanguageService;
};

test("TypeScript Vue plugin exposes generated SFC component types", () => {
  const project = createProject();
  try {
    const { bumpProjectVersion, host } = createHost(project.root, [
      project.mainTs,
      project.ambientDts,
    ]);
    const service = initVuePlugin({ typescript: ts }).create({
      languageService: ts.createLanguageService(host),
      languageServiceHost: host,
      project: { getCurrentDirectory: () => project.root },
      serverHost: ts.sys,
    });
    const source = fs.readFileSync(project.mainTs, "utf8");
    const quickInfo = service.getQuickInfoAtPosition(
      project.mainTs,
      source.indexOf("App from") + 1,
    );

    const display = displayPartsText(quickInfo?.displayParts);
    assert.match(
      display,
      /props: \{ title: "hello   world"; count\?: number; format\?: \(value: \{ nested: string \}\) => string \}/,
    );
    assert.match(display, /emits: \{ save: \[id: string\] \}/);
    assert.match(display, /slots: \{ default\(props: \{ title: string \}\): unknown \}/);
    assert.match(display, /model: "modelValue": boolean/);
    assert.doesNotMatch(display, /ignored|notAnEmit/);
    assert.doesNotMatch(display, /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/);
    const sideEffectInfo = service.getQuickInfoAtPosition(
      project.mainTs,
      source.indexOf("SideEffect.vue") + 1,
    );
    assert.match(displayPartsText(sideEffectInfo?.displayParts), /^const component: VueComponent/);
    const barrelInfo = service.getQuickInfoAtPosition(
      project.mainTs,
      source.indexOf("BarrelApp }") + 1,
    );
    const barrelDisplay = displayPartsText(barrelInfo?.displayParts);
    assert.match(barrelDisplay, /^const BarrelApp: VueComponent/);
    assert.match(barrelDisplay, /props: \{ title: "hello   world"/);
    assert.doesNotMatch(
      barrelDisplay,
      /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/,
    );
    assert.equal(
      service.getDefinitionAtPosition(project.mainTs, source.indexOf("BarrelApp;") + 1)?.[0]
        ?.fileName,
      project.appVue,
    );
    assert.equal(
      service.getTypeDefinitionAtPosition(project.mainTs, source.indexOf("BarrelApp;") + 1)?.[0]
        ?.fileName,
      project.appVue,
    );
    const packageInfo = service.getQuickInfoAtPosition(
      project.mainTs,
      source.indexOf("PackageApp }") + 1,
    );
    const packageDisplay = displayPartsText(packageInfo?.displayParts);
    assert.match(packageDisplay, /^const PackageApp: VueComponent/);
    assert.match(packageDisplay, /props: \{ packageTitle: string \}/);
    assert.doesNotMatch(
      packageDisplay,
      /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/,
    );
    assert.equal(
      service.getDefinitionAtPosition(project.mainTs, source.indexOf("PackageApp;") + 1)?.[0]
        ?.fileName,
      project.packageVue,
    );
    assert.equal(
      service.getTypeDefinitionAtPosition(project.mainTs, source.indexOf("PackageApp;") + 1)?.[0]
        ?.fileName,
      project.packageVue,
    );
    assert.match(
      formatDiagnostics(service.getSemanticDiagnostics(project.mainTs)),
      /Property 'title' is missing/,
    );
    assert.equal(
      nativeGenerationCount(project.nativeCalls),
      2,
      "expected native generation for the imported component and side-effect specifier",
    );

    fs.writeFileSync(
      project.appVue,
      '<script setup lang="ts">\ndefineProps<{ count: number }>();\n</script>\n',
    );
    bumpProjectVersion();

    const refreshedInfo = service.getQuickInfoAtPosition(
      project.mainTs,
      source.indexOf("App from") + 1,
    );
    const refreshedDisplay = displayPartsText(refreshedInfo?.displayParts);
    assert.match(refreshedDisplay, /props: \{ count: number \}/);
    assert.doesNotMatch(refreshedDisplay, /title: string/);
    assert.doesNotMatch(
      refreshedDisplay,
      /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/,
    );
    assert.equal(
      nativeGenerationCount(project.nativeCalls),
      3,
      "expected one additional native generation after the SFC edit",
    );
  } finally {
    fs.rmSync(project.root, { force: true, recursive: true });
  }
});

function createProject() {
  const rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vue-import-types-"));
  const srcDir = path.join(rootDir, "src");
  fs.mkdirSync(srcDir, { recursive: true });

  const mainTs = path.join(srcDir, "main.ts");
  fs.writeFileSync(
    mainTs,
    [
      'import App from "./App.vue";',
      'import { BarrelApp } from "./components";',
      'import { PackageApp } from "@acme/ui";',
      'import "./SideEffect.vue";',
      'const props: InstanceType<typeof App>["$props"] = {};',
      "BarrelApp;",
      "PackageApp;",
      "props;",
      "",
    ].join("\n"),
  );
  const appVue = path.join(srcDir, "App.vue");
  fs.writeFileSync(
    appVue,
    [
      '<script setup lang="ts">',
      "// defineProps<{ ignored: string }>",
      'const stringLiteral = "defineEmits<{ notAnEmit: [] }>();";',
      'defineProps<{ title: "hello   world"; count?: number; format?: (value: { nested: string }) => string }>();',
      "defineEmits<{ save: [id: string] }>();",
      "defineSlots<{ default(props: { title: string }): unknown }>();",
      "defineModel<boolean>();",
      "</script>",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(path.join(srcDir, "SideEffect.vue"), "<template />\n");
  const componentsDir = path.join(srcDir, "components");
  fs.mkdirSync(componentsDir, { recursive: true });
  fs.writeFileSync(
    path.join(componentsDir, "index.ts"),
    'export { default as BarrelApp } from "../App.vue";\n',
  );
  const packageDir = path.join(rootDir, "node_modules/@acme/ui");
  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify({ name: "@acme/ui", types: "index.ts", version: "1.0.0" }),
  );
  fs.writeFileSync(
    path.join(packageDir, "index.ts"),
    'export { default as PackageApp } from "./PackageApp.vue";\n',
  );
  const packageVue = path.join(packageDir, "PackageApp.vue");
  fs.writeFileSync(
    packageVue,
    '<script setup lang="ts">\ndefineProps<{ packageTitle: string }>();\n</script>\n',
  );

  const ambientDts = path.join(srcDir, "vite-client.d.ts");
  fs.writeFileSync(
    ambientDts,
    'declare module "*.vue" {\n  const component: new () => { $props: {} };\n  export default component;\n}\n',
  );

  const vitePluginDir = path.join(rootDir, "node_modules/@vizejs/vite-plugin");
  fs.mkdirSync(vitePluginDir, { recursive: true });
  fs.writeFileSync(
    path.join(vitePluginDir, "package.json"),
    JSON.stringify({ main: "index.cjs", name: "@vizejs/vite-plugin", version: "0.0.0" }),
  );
  fs.writeFileSync(path.join(vitePluginDir, "index.cjs"), '"use strict";\n');

  const nativeDir = path.join(vitePluginDir, "node_modules/@vizejs/native");
  fs.mkdirSync(nativeDir, { recursive: true });
  fs.writeFileSync(
    path.join(nativeDir, "package.json"),
    JSON.stringify({ main: "index.cjs", name: "@vizejs/native", version: "0.0.0" }),
  );
  const nativeCalls = path.join(nativeDir, "calls.txt");
  fs.writeFileSync(nativeCalls, "0");
  fs.writeFileSync(
    path.join(nativeDir, "index.cjs"),
    [
      '"use strict";',
      'const fs = require("node:fs");',
      'const path = require("node:path");',
      "exports.typeCheck = (source) => {",
      '  const callsFile = path.join(__dirname, "calls.txt");',
      '  const calls = Number(fs.readFileSync(callsFile, "utf8")) + 1;',
      "  fs.writeFileSync(callsFile, String(calls));",
      "  const props = source.match(/^\\s*defineProps<([\\s\\S]*?)>\\s*\\(\\s*\\)/m)?.[1] || '{}';",
      "  return { virtualTs: `declare const component: { readonly __vizeComponentMarker: true; readonly __vizeRawProps?: ${props} } & (new () => { $props: ${props} });\\nexport default component;\\n` };",
      "};",
      "",
    ].join("\n"),
  );

  return { ambientDts, appVue, mainTs, nativeCalls, packageVue, root: rootDir };
}

function createHost(rootDir: string, rootFiles: string[]) {
  const options: import("typescript").CompilerOptions = {
    allowSyntheticDefaultImports: true,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    noEmit: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
  };

  let projectVersion = 0;
  const host: import("typescript").LanguageServiceHost = {
    directoryExists: (directoryName) => ts.sys.directoryExists(directoryName),
    fileExists: (fileName) => ts.sys.fileExists(fileName),
    getCompilationSettings: () => options,
    getCurrentDirectory: () => rootDir,
    getDefaultLibFileName: (compilerOptions) => ts.getDefaultLibFilePath(compilerOptions),
    getDirectories: (directoryName) => ts.sys.getDirectories(directoryName),
    getProjectVersion: () => String(projectVersion),
    getScriptFileNames: () => rootFiles,
    getScriptSnapshot(fileName) {
      const source = ts.sys.readFile(fileName);
      return source === undefined ? undefined : ts.ScriptSnapshot.fromString(source);
    },
    getScriptVersion: () => "0",
    readDirectory: (rootDir, extensions, excludes, includes, depth) =>
      ts.sys.readDirectory(rootDir, extensions, excludes, includes, depth),
    readFile: (fileName, encoding) => ts.sys.readFile(fileName, encoding),
    useCaseSensitiveFileNames: () => ts.sys.useCaseSensitiveFileNames,
  };
  return { bumpProjectVersion: () => projectVersion++, host };
}

function displayPartsText(parts: readonly import("typescript").SymbolDisplayPart[] | undefined) {
  return parts?.map((part) => part.text).join("") ?? "";
}

function nativeGenerationCount(callsFile: string) {
  return Number(fs.readFileSync(callsFile, "utf8"));
}

function formatDiagnostics(diagnostics: readonly import("typescript").Diagnostic[]) {
  return diagnostics
    .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"))
    .join("\n");
}
