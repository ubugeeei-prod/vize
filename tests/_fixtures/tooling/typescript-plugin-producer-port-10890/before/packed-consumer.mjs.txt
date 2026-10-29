import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { startTsserver, position } from "./tsserver-client.mjs";
import { createNavigationFixtures, checkNavigation } from "./authored-navigation.ts";
import { checkProjectMetadata } from "./project-metadata.ts";
import {
  createInheritedConfig,
  checkInheritedConfig,
  checkConfigurationLog,
  checkAutoTargetMetadata,
} from "./inherited-config.ts";

const typescriptVersion = process.env.VIZE_TYPESCRIPT_PLUGIN_TYPESCRIPT || "6.0.3";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
// Scenario directories are created exclusively, so local reruns need a fresh capture.
const output = process.env.VIZE_TYPESCRIPT_PLUGIN_CAPTURE
  ? path.resolve(process.env.VIZE_TYPESCRIPT_PLUGIN_CAPTURE)
  : fs.mkdtempSync(path.join(os.tmpdir(), "vize-typescript-plugin-capture-"));
fs.mkdirSync(output, { recursive: true });
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-tsserver-consumer-"));
const digest = (data) => createHash("sha256").update(data).digest("hex");
const run = (command, args, cwd = temporary) =>
  execFileSync(command, args, { cwd, encoding: "utf8", timeout: 90_000, maxBuffer: 8_000_000 });
const manifest = (name) =>
  JSON.parse(fs.readFileSync(path.join(root, name, "package.json"), "utf8"));
const nativeManifest = manifest("npm/native");
const pluginManifest = manifest("npm/typescript-plugin");
function pack(name, packageJson, files) {
  const stage = path.join(temporary, name);
  fs.mkdirSync(stage);
  for (const [source, target] of files) fs.copyFileSync(source, path.join(stage, target));
  fs.writeFileSync(path.join(stage, "package.json"), JSON.stringify(packageJson, null, 2));
  const report = JSON.parse(
    run("npm", ["pack", "--ignore-scripts", "--json", "--pack-destination", output], stage),
  )[0];
  fs.writeFileSync(path.join(output, `${name}-pack.json`), JSON.stringify(report, null, 2));
  return path.join(output, report.filename);
}
const nativeFiles = fs
  .readdirSync(path.join(root, "npm/native"))
  .filter((name) => nativeManifest.files.includes(name) || name.endsWith(".node"));
assert(
  nativeFiles.some((name) => name.endsWith(".node")),
  "Build the exact-source native binary before this test",
);
const native = pack(
  "native",
  {
    ...nativeManifest,
    scripts: {},
    optionalDependencies: {},
    devDependencies: {},
    files: nativeFiles,
  },
  nativeFiles.map((name) => [path.join(root, "npm/native", name), name]),
);
const pluginFiles = fs
  .readdirSync(path.join(root, "npm/typescript-plugin"))
  .filter((name) => /\.cjs$|\.d\.cts$|^README\.md$/.test(name));
const plugin = pack(
  "plugin",
  { ...pluginManifest, dependencies: { "@vizejs/native": nativeManifest.version } },
  pluginFiles.map((name) => [path.join(root, "npm/typescript-plugin", name), name]),
);
const consumer = path.join(temporary, "consumer");
fs.mkdirSync(consumer);
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify({ private: true, name: "vize-tsserver-consumer", version: "1.0.0" }),
);
fs.writeFileSync(
  path.join(output, "install.log"),
  run(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      native,
      plugin,
      `typescript@${typescriptVersion}`,
      "vue@3.5.35",
    ],
    consumer,
  ),
);
const pluginPath = path.join(consumer, "node_modules/@vizejs/typescript-vue-plugin");
for (const name of pluginFiles)
  assert.equal(
    digest(fs.readFileSync(path.join(pluginPath, name))),
    digest(fs.readFileSync(path.join(root, "npm/typescript-plugin", name))),
  );
const config = {
  compilerOptions: {
    strict: true,
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "Bundler",
    noEmit: true,
    skipLibCheck: true,
    baseUrl: ".",
    paths: { "@/*": ["src/*"] },
    ...(typescriptVersion.startsWith("6.") ? { ignoreDeprecations: "6.0" } : {}),
    plugins: [{ name: pluginManifest.name }],
  },
  include: ["src"],
};
fs.writeFileSync(path.join(consumer, "tsconfig.json"), JSON.stringify(config));
const declarationsConfig = path.join(consumer, "declarations.json");
fs.writeFileSync(
  declarationsConfig,
  JSON.stringify({
    compilerOptions: {
      strict: true,
      target: "ES2022",
      module: "Node16",
      moduleResolution: "Node16",
      noEmit: true,
      types: [],
    },
    files: ["node_modules/@vizejs/typescript-vue-plugin/index.d.cts"],
  }),
);
fs.writeFileSync(
  path.join(output, "declarations.stdout"),
  run(
    process.execPath,
    [path.join(consumer, "node_modules/typescript/bin/tsc"), "-p", declarationsConfig],
    consumer,
  ),
);
const src = path.join(consumer, "src");
fs.mkdirSync(src);
const main = path.join(src, "main.ts"),
  vue = path.join(src, "App.vue"),
  art = path.join(src, "Sample.art");
const mainText =
  'import App from "@/App.vue";\nimport Art from "./Sample.art";\nimport Missing from "./Missing.vue";\nconst props: InstanceType<typeof App>["$props"] = {};\nApp; Art; Missing;\n';
const vueText =
  '<script setup lang="ts">\nimport { word } from "./util";\nconst emoji = "😀 日本語";\nconst state = { count: 1, caption: word };\nconst answer: number = "wrong";\nfunction double(value: number) { return value * 2; }\nconst result = double(state.count);\ndefineProps<{ title: string }>();\n</script>\n<template>{{ state.count }} {{ double(2) }} {{ emoji }} {{ result }}</template>\n';
const artText =
  '<script setup lang="ts">\nconst artCount: number = "wrong";\n</script>\n<template>{{ artCount }}</template>\n';
fs.writeFileSync(main, mainText);
fs.writeFileSync(vue, vueText);
fs.writeFileSync(art, artText);
fs.writeFileSync(path.join(src, "util.ts"), 'export const word = "hello";\n');
const consumerRequire = createRequire(path.join(consumer, "package.json"));
const nativeApi = consumerRequire("@vizejs/native");
const projectionHelper = consumerRequire("@vizejs/native/language-projection.js");
const loadedImages = Object.values(consumerRequire.cache).filter(
  (entry) =>
    entry.filename.endsWith(".node") &&
    entry.exports.generateVueLanguageCode === nativeApi.generateVueLanguageCode,
);
assert.equal(loadedImages.length, 1, "Capture the actual native image serving this projection API");
const loadedImage = {
  file: path.basename(loadedImages[0].filename),
  sha256: digest(fs.readFileSync(loadedImages[0].filename)),
  nativeBuildToken: nativeApi.getNativeBuildToken(),
};
assert.match(loadedImage.nativeBuildToken, /^[0-9a-f]{32}$/);
assert.equal(
  loadedImage.sha256,
  digest(fs.readFileSync(path.join(root, "npm/native", loadedImage.file))),
);
fs.writeFileSync(path.join(output, "native-image.json"), JSON.stringify(loadedImage, null, 2));
checkProjectMetadata(nativeApi, consumer, output);
checkAutoTargetMetadata(nativeApi, consumer, output);
for (const [file, source] of [
  [vue, vueText],
  [art, artText],
]) {
  const raw = nativeApi.generateVueLanguageCode(source, {
    filename: file,
    configFileName: path.join(consumer, "tsconfig.json"),
    preserveUnusedDiagnostics: true,
  });
  const normalized = projectionHelper.toUtf16LanguageProjection(source, raw);
  fs.writeFileSync(
    path.join(output, `${path.basename(file)}.projection.json`),
    JSON.stringify({ source, raw: JSON.parse(raw), normalized }, null, 2),
  );
}
const navigation = createNavigationFixtures(src, output);
const inherited = createInheritedConfig(consumer, src, output, typescriptVersion);
const tsserver = path.join(consumer, "node_modules/typescript/lib/tsserver.js");
const client = startTsserver(tsserver, consumer, output);
const results = {};
async function request(name, command, file, text, marker, extra = {}) {
  const response = await client.send(command, {
    file,
    ...(marker ? position(text, marker[0], marker[1]) : {}),
    ...extra,
  });
  results[name] = response;
  assert.equal(response.success, true, JSON.stringify(response));
  return response;
}
try {
  await client.send("configure", {
    preferences: { includeCompletionsForModuleExports: false },
    extraFileExtensions: [
      { extension: ".vue", isMixedContent: true, scriptKind: 7 },
      { extension: ".art", isMixedContent: true, scriptKind: 7 },
    ],
  });
  await client.send("open", { file: main, fileContent: mainText }, false);
  await client.send("open", { file: vue, fileContent: vueText, scriptKindName: "TS" }, false);
  await client.send("open", { file: art, fileContent: artText, scriptKindName: "TS" }, false);
  const hover = await request("hover", "quickinfo", vue, vueText, ["state.count", 7]);
  assert(hover.success, JSON.stringify(hover));
  assert.match(hover.body.displayString, /number/);
  const completion = await request("completion", "completionInfo", vue, vueText, [
    "state.count",
    6,
  ]);
  assert(
    completion.body.entries.some((entry) => entry.name === "count"),
    JSON.stringify(completion),
  );
  const definition = await request("definition", "definition", vue, vueText, ["double(state", 2]);
  assert(
    definition.body.some((entry) => entry.file === vue && entry.start.line === 6),
    JSON.stringify(definition),
  );
  const references = await request("references", "references", vue, vueText, ["double(value", 2]);
  assert(
    references.body.refs.filter((entry) => entry.file === vue).length >= 2,
    JSON.stringify(references),
  );
  const rename = await request("rename", "rename", vue, vueText, ["double(value", 2], {
    findInStrings: false,
    findInComments: false,
  });
  assert.equal(rename.body.info.canRename, true, JSON.stringify(rename));
  assert(rename.body.locs.some((entry) => entry.file === vue && entry.locs.length >= 2));
  for (const group of rename.body.locs)
    for (const edit of group.locs) {
      const text = fs.readFileSync(group.file, "utf8");
      const lines = text.split("\n");
      assert.equal(edit.start.line, edit.end.line);
      assert.equal(
        lines[edit.start.line - 1].slice(edit.start.offset - 1, edit.end.offset - 1),
        "double",
      );
    }
  const brokenFile = path.join(src, "Broken.vue"),
    brokenSource = '<script setup lang="ts">\nconst n = 1;\n';
  fs.writeFileSync(brokenFile, brokenSource);
  await client.send(
    "open",
    { file: brokenFile, fileContent: brokenSource, scriptKindName: "TS" },
    false,
  );
  const syntaxPacket = await request(
    "native-syntax-diagnostics",
    "syntacticDiagnosticsSync",
    brokenFile,
    brokenSource,
  );
  assert(
    syntaxPacket.body.some((entry) => entry.code === 100001),
    JSON.stringify(syntaxPacket),
  );
  const vueDiagnostics = await request("vue-diagnostics", "semanticDiagnosticsSync", vue, vueText);
  assert(
    vueDiagnostics.body.some((entry) => entry.code === 2322 && entry.start.line === 5),
    JSON.stringify(vueDiagnostics),
  );
  const artDiagnostics = await request("art-diagnostics", "semanticDiagnosticsSync", art, artText);
  assert(
    artDiagnostics.body.some((entry) => entry.code === 2322 && entry.start.line === 2),
    JSON.stringify(artDiagnostics),
  );
  const tsDiagnostics = await request("ts-diagnostics", "semanticDiagnosticsSync", main, mainText);
  assert(
    tsDiagnostics.body.some((entry) => entry.code === 2307 && entry.text.includes("Missing.vue")),
    JSON.stringify(tsDiagnostics),
  );
  assert(
    !tsDiagnostics.body.some((entry) => entry.code === 2307 && entry.text.includes("App.vue")),
    JSON.stringify(tsDiagnostics),
  );
  assert(
    tsDiagnostics.body.some((entry) => [2322, 2741].includes(entry.code)),
    JSON.stringify(tsDiagnostics),
  );
  await checkNavigation(client, navigation, results, vue, vueText, path.join(src, "util.ts"));
  const inheritedQualification = await checkInheritedConfig(
    client,
    inherited,
    nativeApi,
    tsserver,
    consumer,
    output,
    results,
  );
  let renamed = vueText;
  const lineOffsets = [0];
  for (let index = 0; index < vueText.length; index++)
    if (vueText[index] === "\n") lineOffsets.push(index + 1);
  const renameEdits = rename.body.locs
    .find((group) => group.file === vue)
    .locs.map((edit) => ({
      start: lineOffsets[edit.start.line - 1] + edit.start.offset - 1,
      end: lineOffsets[edit.end.line - 1] + edit.end.offset - 1,
    }))
    .sort((left, right) => right.start - left.start);
  for (const edit of renameEdits)
    renamed = renamed.slice(0, edit.start) + "twice" + renamed.slice(edit.end);
  await client.send("change", {
    file: vue,
    line: 1,
    offset: 1,
    endLine: vueText.split("\n").length,
    endOffset: 1,
    insertString: renamed,
  });
  const renamedDefinition = await request(
    "definition-after-applied-rename",
    "definition",
    vue,
    renamed,
    ["twice(state", 2],
  );
  assert(renamedDefinition.body.some((entry) => entry.file === vue && entry.start.line === 6));
  const edited = renamed.replace("count: 1", 'count: "changed"');
  await client.send("change", {
    file: vue,
    line: 1,
    offset: 1,
    endLine: vueText.split("\n").length,
    endOffset: 1,
    insertString: edited,
  });
  const afterEdit = await request("hover-after-unsaved-edit", "quickinfo", vue, edited, [
    "state.count",
    7,
  ]);
  assert.match(afterEdit.body.displayString, /string/);
  await client.send("change", {
    file: vue,
    line: 1,
    offset: 1,
    endLine: edited.split("\n").length,
    endOffset: 1,
    insertString: "",
  });
  const cleared = await request(
    "diagnostics-after-unsaved-clear",
    "semanticDiagnosticsSync",
    main,
    mainText,
  );
  assert(!cleared.body.some((entry) => [2322, 2741].includes(entry.code)), JSON.stringify(cleared));
  const tsc = path.join(consumer, "node_modules/typescript/bin/tsc");
  try {
    run(process.execPath, [tsc, "-p", consumer, "--pretty", "false"], consumer);
    throw new Error("Ordinary tsc unexpectedly accepted Vue imports");
  } catch (error) {
    if (!error.stdout) throw error;
    fs.writeFileSync(path.join(output, "ordinary-tsc.stdout"), error.stdout);
    assert.match(String(error.stdout), /TS2307.*App\.vue/);
  }
  await client.stop();
  async function isolatedScenario(name, file, source, expectedCode, predicate = () => true) {
    const directory = path.join(output, name);
    fs.mkdirSync(directory);
    const negative = startTsserver(tsserver, consumer, directory);
    try {
      await negative.send("configure", {
        extraFileExtensions: [
          { extension: ".vue", isMixedContent: true, scriptKind: 7 },
          { extension: ".art", isMixedContent: true, scriptKind: 7 },
        ],
      });
      await negative.send("open", { file: main, fileContent: mainText }, false);
      await negative.send("open", { file, fileContent: source }, false);
      const response = await negative.send("semanticDiagnosticsSync", { file });
      results[name] = response;
      assert(
        response.body.some(
          (entry) =>
            (Array.isArray(expectedCode) ? expectedCode : [expectedCode]).includes(entry.code) &&
            predicate(entry),
        ),
        JSON.stringify(response),
      );
    } finally {
      await negative.stop();
    }
  }
  config.compilerOptions.module = "NodeNext";
  config.compilerOptions.moduleResolution = "NodeNext";
  fs.writeFileSync(path.join(consumer, "tsconfig.json"), JSON.stringify(config));
  await isolatedScenario("native-nodenext", main, mainText, [2322, 2741]);
  assert(
    !results["native-nodenext"].body.some(
      (entry) => entry.code === 2307 && entry.text.includes("App.vue"),
    ),
  );
  const closedReferencesDirectory = path.join(output, "closed-component-references");
  fs.mkdirSync(closedReferencesDirectory);
  const closedReferencesClient = startTsserver(tsserver, consumer, closedReferencesDirectory);
  try {
    await closedReferencesClient.send("configure", {
      extraFileExtensions: [
        { extension: ".vue", isMixedContent: true, scriptKind: 7 },
        { extension: ".art", isMixedContent: true, scriptKind: 7 },
      ],
    });
    await closedReferencesClient.send("open", { file: main, fileContent: mainText }, false);
    const utilFile = path.join(src, "util.ts");
    await closedReferencesClient.send(
      "open",
      { file: utilFile, fileContent: fs.readFileSync(utilFile, "utf8") },
      false,
    );
    const response = await closedReferencesClient.send("references", {
      file: utilFile,
      line: 1,
      offset: 15,
    });
    results["closed-component-references"] = response;
    assert(
      response.body.refs.some(
        (entry) =>
          entry.file === vue &&
          entry.start.line === 2 &&
          entry.lineText.includes("import { word }"),
      ),
      JSON.stringify(response),
    );
  } finally {
    await closedReferencesClient.stop();
  }
  const helperPath = path.join(consumer, "node_modules/@vizejs/native/language-projection.js");
  fs.renameSync(helperPath, `${helperPath}.disabled`);
  try {
    await isolatedScenario("missing-native-helper", vue, vueText, 100003);
  } finally {
    fs.renameSync(`${helperPath}.disabled`, helperPath);
  }
  config.compilerOptions.plugins[0].enabled = false;
  fs.writeFileSync(path.join(consumer, "tsconfig.json"), JSON.stringify(config));
  await isolatedScenario("disabled-plugin", main, mainText, 2307, (entry) =>
    entry.text.includes("App.vue"),
  );
  await client.stop();
  checkConfigurationLog(output);
  fs.writeFileSync(
    path.join(output, "receipt.json"),
    JSON.stringify(
      {
        source: run("git", ["rev-parse", "HEAD"], root).trim(),
        node: process.version,
        loadedImage,
        tsserverVersion: JSON.parse(
          fs.readFileSync(path.join(consumer, "node_modules/typescript/package.json")),
        ).version,
        nativeBinary: nativeFiles
          .filter((name) => name.endsWith(".node"))
          .map((name) => ({
            name,
            sha256: digest(fs.readFileSync(path.join(root, "npm/native", name))),
          })),
        pluginFiles: pluginFiles.map((name) => ({
          name,
          sha256: digest(fs.readFileSync(path.join(pluginPath, name))),
        })),
        tarballs: [native, plugin].map((file) => ({
          file: path.basename(file),
          sha256: digest(fs.readFileSync(file)),
        })),
        results: Object.keys(results),
        qualificationRole: "typescript-vue-projection",
        inheritedQualification,
        passed: true,
      },
      null,
      2,
    ),
  );
} finally {
  fs.writeFileSync(path.join(output, "results.json"), JSON.stringify(results, null, 2));
  await client.stop();
  fs.rmSync(temporary, { recursive: true, force: true });
}
